use crate::{
    math::{Vec2, Vec3},
    renderer::GPUVertexData,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::NoUninit)]
pub struct MeshVertex {
    pub position: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
}

impl GPUVertexData for MeshVertex {
    fn get_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 2,
                },
            ],
        }
    }
}
