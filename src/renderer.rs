use std::{borrow::Cow, ops::Range, rc::Rc, sync::Arc};

use crate::{math::{Mat4, Vec2, Vec3}, mesh::MeshVertex};

pub trait GPUVertexData: bytemuck::NoUninit {
    fn get_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub struct TextureInner {
    queue: wgpu::Queue,
    wgpu_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

#[derive(Clone)]
pub struct Texture {
    inner: Rc<TextureInner>
}

impl Texture {
    pub fn new(
        queue: wgpu::Queue,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        let view = texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            inner: Rc::new(TextureInner { queue, wgpu_texture: texture, bind_group })
        }
    }

    pub fn width(&self) -> u32 {
        self.inner.wgpu_texture.width()
    }

    pub fn height(&self) -> u32 {
        self.inner.wgpu_texture.height()
    }

    pub fn set_data(&self, data: &[u8]) {
        self.inner.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.inner.wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width() * 4 as u32),
                rows_per_image: Some(self.height()),
            },
            self.inner.wgpu_texture.size(),
        );
    }
}

enum DrawCommand {
    Draw { vertex_range: Range<u32> },
    SetTexture(Texture),
}

pub struct Renderer {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    default_texture: Texture,
    transform: Mat4,
    color: Vec3,
    vertex_buffer: wgpu::Buffer,
    vertices: Vec<MeshVertex>,
    last_flushed_vertex: usize,
    commands: Vec<DrawCommand>,
    default_pipeline: wgpu::RenderPipeline
}

impl Renderer {
    pub async fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        window: Arc<winit::window::Window>,
    ) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(event_loop.owned_display_handle()),
        ));

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_format = surface.get_capabilities(&adapter).formats[0];

        println!("GPU device: {}", device.adapter_info().name);
        println!("GPU backend: {}", device.adapter_info().backend);

        let size = window.inner_size();

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size_of::<Vec2>() as u64),
                    },
                    count: None,
                }],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<Vec2>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(size_of::<Vec2>() as u64),
                }),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::all(),
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let default_texture =
            Texture::new(queue.clone(), &device, &texture_bind_group_layout, 1, 1);
        default_texture.set_data(&[255, 255, 255, 255]);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader_module = device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let pipeline_layout = device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    Some(&uniform_bind_group_layout),
                    Some(&texture_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let default_pipeline = device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(MeshVertex::get_layout())],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });

        let renderer = Self {
            window,
            surface,
            surface_format,
            device,
            queue,
            size,
            uniform_bind_group,
            uniform_buffer,
            texture_bind_group_layout,
            default_texture,
            transform: Mat4::identity(),
            color: Vec3::new(1., 1., 1.),
            vertex_buffer,
            vertices: Vec::new(),
            last_flushed_vertex: 0,
            commands: Vec::new(),
            default_pipeline
        };

        renderer.configure_surface();

        return renderer;
    }

    fn configure_surface(&self) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: self.size.width,
                height: self.size.height,
                present_mode: wgpu::PresentMode::Immediate,
                desired_maximum_frame_latency: 1,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.surface_format],
            },
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
        let size_vec = Vec2::new(new_size.width as f32, new_size.height as f32);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&size_vec));
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn create_texture(&self, width: u32, height: u32) -> Texture {
        Texture::new(
            self.queue.clone(),
            &self.device,
            &self.texture_bind_group_layout,
            width,
            height,
        )
    }

    pub fn set_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }

    pub fn apply_matrix(&mut self, matrix: Mat4) {
        self.transform *= matrix;
    }

    pub fn reset_transform(&mut self) {
        self.set_transform(Mat4::identity());
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.apply_matrix(Mat4::translate(translation));
    }

    pub fn rotate(&mut self, radians: f32) {
        self.apply_matrix(Mat4::rotateZ(radians));
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.apply_matrix(Mat4::scale(scale));
    }

    pub fn flush_vertices(&mut self) {
        if self.vertices.len() > self.last_flushed_vertex {
            self.commands.push(DrawCommand::Draw {
                vertex_range: (self.last_flushed_vertex as u32)..(self.vertices.len() as u32)
            });
            self.last_flushed_vertex = self.vertices.len();
        }
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.flush_vertices();
        self.commands.push(DrawCommand::SetTexture(texture.clone()));
    }

    pub fn reset_texture(&mut self) {
        self.set_texture(self.default_texture.clone());
    }

    pub fn vertex(&mut self, position: Vec3, uv: Vec2) {
        self.vertices.push(MeshVertex {
            position: self.transform * position,
            color: self.color,
            uv,
        });
    }

    pub fn quad(
        &mut self,
        bottom_left: Vec3,
        bottom_right: Vec3,
        top_right: Vec3,
        top_left: Vec3,
    ) {
        self.vertex(bottom_left, Vec2::new(0., 1.));
        self.vertex(bottom_right, Vec2::new(1., 1.));
        self.vertex(top_right, Vec2::new(1., 0.));

        self.vertex(top_right, Vec2::new(1., 0.));
        self.vertex(top_left, Vec2::new(0., 0.));
        self.vertex(bottom_left, Vec2::new(0., 1.));
    }

    pub fn rectangle(&mut self, position: Vec3, size: Vec2) {
        self.quad(
            position, 
            position + Vec3::new(size.x, 0., 0.),
            position + Vec3::new(size.x, size.y, 0.),
            position + Vec3::new(0., size.y, 0.)
        );
    }

    pub fn finish_frame(&mut self) -> bool {
        let result = self.finish_frame_inner();
        self.vertices.clear();
        self.last_flushed_vertex = 0;
        self.commands.clear();
        return result;
    }

    fn finish_frame_inner(&mut self) -> bool {
        let current_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                self.configure_surface();
                return false;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return false,
            wgpu::CurrentSurfaceTexture::Occluded => return false,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return false;
            }
            _ => panic!(),
        };

        let texture_view = current_texture
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        self.flush_vertices();

        let vertex_bytes = bytemuck::cast_slice(&self.vertices);
        if vertex_bytes.len() as u64 > self.vertex_buffer.size() {
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: vertex_bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.queue.write_buffer(&self.vertex_buffer, 0,vertex_bytes);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            render_pass.set_pipeline(&self.default_pipeline);
            render_pass.set_bind_group(0, Some(&self.uniform_bind_group), &[]);
            render_pass.set_bind_group(1, Some(&self.default_texture.inner.bind_group), &[]);
            if vertex_bytes.len() > 0 {
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..(vertex_bytes.len() as u64)));
            }
            for command in &self.commands {
                match command {
                    DrawCommand::Draw { vertex_range } => {
                        render_pass.draw(vertex_range.clone(), 0..1);
                    },
                    DrawCommand::SetTexture(tex) => {
                        render_pass.set_bind_group(1, Some(&tex.inner.bind_group), &[]);
                    }
                }
            }
        }

        self.window.pre_present_notify();
        self.queue
            .submit(Some(encoder.finish()));
        self.queue.present(current_texture);

        return true;
    }
}
