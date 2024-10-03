use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
}

impl Vertex {
    pub fn new(position: impl Into<[f32; 3]>) -> Self {
        let position = position.into();
        Self {
            position: [position[0], position[1], position[2], 0.0],
        }
    }

    pub fn layout() -> VertexBufferLayout<'static> {
        const ATTRIBUTES: &'static [VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRIBUTES,
        }
    }
}

impl From<common::Vertex> for Vertex {
    fn from(value: common::Vertex) -> Self {
        Self {
            position: [value.xyz[0], value.xyz[1], value.xyz[2], 0.0],
        }
    }
}