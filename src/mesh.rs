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

pub struct MeshBuilder {
    vertices: Vec<MeshVertex>,
    color: Vec3,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            color: Vec3::new(1., 1., 1.),
        }
    }

    pub fn add_vertex_with_uv(&mut self, position: Vec3, uv: Vec2) {
        self.vertices.push(MeshVertex {
            position,
            color: self.color,
            uv,
        });
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn add_vertex(&mut self, position: Vec3) {
        self.add_vertex_with_uv(position, Vec2::new(0., 0.));
    }

    pub fn set_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn add_quad(
        &mut self,
        top_left: Vec3,
        top_right: Vec3,
        bottom_left: Vec3,
        bottom_right: Vec3,
    ) {
        self.add_vertex_with_uv(top_left, Vec2::new(0., 0.));
        self.add_vertex_with_uv(bottom_left, Vec2::new(0., 1.));
        self.add_vertex_with_uv(bottom_right, Vec2::new(1., 1.));

        self.add_vertex_with_uv(bottom_right, Vec2::new(1., 1.));
        self.add_vertex_with_uv(top_right, Vec2::new(1., 0.));
        self.add_vertex_with_uv(top_left, Vec2::new(0., 0.));
    }

    pub fn mesh(&self) -> &[MeshVertex] {
        &self.vertices
    }
}
