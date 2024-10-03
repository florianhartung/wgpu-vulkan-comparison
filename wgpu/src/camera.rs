use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Perspective, Point3, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferUsages, Device, Queue, ShaderStages
};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraRaw {
    view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Camera {
    pub fn new(device: &Device, initial_position: (f32, f32, f32), pitch: f32, yaw: f32, aspect_ratio: f32) -> Self {
        let camera_raw = make_raw_camera(initial_position.into(), pitch_yaw_to_dir(pitch, yaw), aspect_ratio);

        let buffer: Buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[camera_raw]),
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&mut self, queue: &Queue, position: (f32, f32, f32), pitch: f32, yaw: f32, aspect_ratio: f32) {
        let camera_raw = make_raw_camera(position.into(), pitch_yaw_to_dir(pitch, yaw), aspect_ratio);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[camera_raw]));
    }

    pub fn update_with_camera(&mut self, queue: &Queue, camera: common::Camera, aspect_ratio: f32) {
        self.update(queue, camera.xyz, camera.pitch, camera.yaw, aspect_ratio);
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}


fn pitch_yaw_to_dir(pitch: f32, yaw: f32) -> Vector3<f32> {
    Vector3::new(yaw.cos() * pitch.cos(), pitch.sin(), pitch.cos() * yaw.sin())
}

fn make_raw_camera(eye: Point3<f32>, dir: Vector3<f32>, aspect_ratio: f32) -> CameraRaw {
    let view = Matrix4::look_to_rh(eye, dir, Vector3::unit_y());
    let proj = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 100.0);

    let view_proj = OPENGL_TO_WGPU_MATRIX * proj * view;

    CameraRaw {
        view_proj: view_proj.into(),
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
