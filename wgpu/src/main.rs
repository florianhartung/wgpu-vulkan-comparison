use std::{num::NonZeroU32, sync::Arc};

use camera::Camera;
use common::{HasWindowAndDisplayHandle, Mesh};
use vertex::Vertex;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    Adapter, Backends, BindGroup, BindGroupLayout, Buffer, BufferAddress, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, Device, DeviceDescriptor, Features, FragmentState,
    Instance, InstanceDescriptor, InstanceFlags, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, VertexState,
};

pub mod vertex;
mod camera;

const MAX_VERTICES: usize = 100;
const MAX_INDICES: usize = 100;

struct WgpuRenderer {
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    num_vertices: usize,
    index_buffer: Buffer,
    num_indices: usize,
    camera: Camera,
}

impl common::Renderer for WgpuRenderer {
    fn new(
        window: impl HasWindowAndDisplayHandle + Send + Sync + 'static,
        initial_window_size: (u32, u32),
    ) -> Self {
        const DEVICE_FEATURES: Features = Features::empty();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: if cfg!(debug_assertions) {
                InstanceFlags::debugging()
            } else {
                InstanceFlags::empty()
            },
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                required_features: DEVICE_FEATURES,
                ..Default::default()
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_window_size.0,
            height: initial_window_size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (MAX_INDICES * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let aspect_ratio = surface_config.width as f32 / surface_config.height as f32;
        let camera = Camera::new(&device, (0.0, 0.0, 0.0), 0.0, 1.0, aspect_ratio);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[camera.bind_group_layout()],
            push_constant_ranges: &[],
        });

        let vertex_shader = include_wgsl!("shader.wgsl");
        let shader_module = device.create_shader_module(vertex_shader);

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[Vertex::layout()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            label: None,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            adapter,
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            vertex_buffer,
            num_vertices: 0,
            index_buffer,
            num_indices: 0,
            camera,
        }
    }

    fn render(&mut self, camera: common::Camera) {
        self.camera.update_with_camera(&self.queue, camera, self.surface_config.width as f32 / self.surface_config.height as f32);

        let frame = self.surface.get_current_texture().unwrap();

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {r: 0.4, g: 0.9, b: 1.0, a: 1.0}),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if self.num_indices > 0 && self.num_vertices > 0 {
                rpass.set_pipeline(&self.render_pipeline);

                rpass.set_vertex_buffer(
                    0,
                    self.vertex_buffer
                        .slice(..),
                );
                rpass.set_index_buffer(
                    self.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                rpass.set_bind_group(0, self.camera.bind_group(), &[]);

                rpass.draw_indexed(0..self.num_indices as u32, 0, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, size: (NonZeroU32, NonZeroU32), _scale_factor: f64) {
        // Reconfigure the surface with the new size
        self.surface_config.width = size.0.get();
        self.surface_config.height = size.1.get();
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn load_mesh(&mut self, mesh: Mesh) {
        let Mesh {
            vertices,
            mut indices,
        } = mesh;

        let vertices: Vec<Vertex> = vertices.into_iter().map(Into::into).collect();

        if self.num_vertices + vertices.len() > MAX_VERTICES {
            panic!("Reached maximum amount of vertices")
        }

        if self.num_indices + indices.len() > MAX_INDICES {
            panic!("Reached maximum amount of indices")
        }

        assert_eq!(indices.len() % 3, 0);

        // Offset indices
        for index in &mut indices {
            *index = *index + self.num_vertices as u32;
        }

        self.queue.write_buffer(
            &self.vertex_buffer,
            (self.num_vertices * std::mem::size_of::<Vertex>()) as u64,
            bytemuck::cast_slice(&vertices),
        );
        self.queue.write_buffer(
            &self.index_buffer,
            (self.num_indices * std::mem::size_of::<u32>()) as u64,
            bytemuck::cast_slice(&indices),
        );

        self.num_vertices += vertices.len();
        self.num_indices += indices.len();
    }
}

fn main() {
    common::run_app::<WgpuRenderer>();
}
