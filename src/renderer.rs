use std::{borrow::Cow, sync::Arc};

pub trait GPUVertexData: bytemuck::NoUninit {
    fn get_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub struct VertexBuffer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub wgpu_buffer: wgpu::Buffer,
    vertex_count: usize,
}

impl VertexBuffer {
    pub fn new<V: GPUVertexData>(device: wgpu::Device, queue: wgpu::Queue, vertices: &[V]) -> Self {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        let buffer = Self::create_wgpu_buffer(&device, bytes.len());
        queue.write_buffer(&buffer, 0, bytes);
        Self {
            device,
            queue,
            wgpu_buffer: buffer,
            vertex_count: vertices.len(),
        }
    }

    fn create_wgpu_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn set_vertices_raw(&mut self, vertex_data: &[u8], vertex_count: usize) {
        if vertex_data.len() != self.size() {
            self.wgpu_buffer = Self::create_wgpu_buffer(&self.device, vertex_data.len());
        }
        self.queue.write_buffer(&self.wgpu_buffer, 0, &vertex_data);
        self.vertex_count = vertex_count;
    }

    pub fn set_vertices<V: GPUVertexData>(&mut self, vertices: &[V]) {
        self.set_vertices_raw(bytemuck::cast_slice(vertices), vertices.len());
    }

    pub fn count(&self) -> usize {
        self.vertex_count
    }

    pub fn size(&self) -> usize {
        self.wgpu_buffer.size() as usize
    }
}

pub struct GPUTexture {
    queue: wgpu::Queue,
    wgpu_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl GPUTexture {
    pub fn width(&self) -> u32 {
        self.wgpu_texture.width()
    }

    pub fn height(&self) -> u32 {
        self.wgpu_texture.height()
    }

    pub fn set_data(&self, data: &[u8]) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.wgpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.wgpu_texture.width() * 4 as u32),
                rows_per_image: Some(self.wgpu_texture.height()),
            },
            self.wgpu_texture.size(),
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
struct UniformData {
    aspect: f32,
}

pub struct Renderer {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    uniform_data: UniformData,
    uniform_data_dirty: bool,
    texture_bind_group_layout: wgpu::BindGroupLayout,
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
                        min_binding_size: wgpu::BufferSize::new(size_of::<UniformData>() as u64),
                    },
                    count: None,
                }],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<UniformData>() as u64,
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
                    size: wgpu::BufferSize::new(size_of::<UniformData>() as u64),
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

        let renderer = Self {
            window,
            surface,
            surface_format,
            device,
            queue,
            size,
            uniform_bind_group_layout,
            uniform_bind_group,
            uniform_buffer,
            uniform_data: UniformData { aspect: 0. },
            uniform_data_dirty: true,
            texture_bind_group_layout,
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
        self.uniform_data.aspect = (new_size.width as f32) / (new_size.height as f32);
        self.uniform_data_dirty = true;
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn create_wgsl_shader_module(&self, source: &str) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            })
    }

    pub fn create_render_pipeline(
        &self,
        shader_module: &wgpu::ShaderModule,
        vertex_layout: wgpu::VertexBufferLayout<'_>,
        vertex_main: &str,
        fragment_main: &str,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    Some(&self.uniform_bind_group_layout),
                    Some(&self.texture_bind_group_layout),
                ],
                immediate_size: 0,
            });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader_module,
                    entry_point: Some(vertex_main),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(vertex_layout)],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: shader_module,
                    entry_point: Some(fragment_main),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
    }

    pub fn create_wgsl_render_pipeline(
        &self,
        source: &str,
        vertex_layout: wgpu::VertexBufferLayout<'_>,
        vertex_main: &str,
        fragment_main: &str,
    ) -> wgpu::RenderPipeline {
        self.create_render_pipeline(
            &self.create_wgsl_shader_module(source),
            vertex_layout,
            vertex_main,
            fragment_main,
        )
    }

    pub fn create_vertex_buffer<V: GPUVertexData>(&self, vertices: &[V]) -> VertexBuffer {
        VertexBuffer::new(self.device.clone(), self.queue.clone(), vertices)
    }

    pub fn create_texture(&self, width: u32, height: u32) -> GPUTexture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
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

        let sampler = self
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
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

        GPUTexture {
            queue: self.queue.clone(),
            wgpu_texture: texture,
            bind_group,
        }
    }

    pub fn begin_frame(&mut self) -> Option<RendererFrame<'_>> {
        let current_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                self.configure_surface();
                return None;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return None,
            wgpu::CurrentSurfaceTexture::Occluded => return None,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return None;
            }
            _ => panic!(),
        };

        let texture_view = current_texture
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        return Some(RendererFrame {
            renderer: self,
            command_encoder: encoder,
            surface_texture: current_texture,
            texture_view,
        });
    }
}

pub struct RendererFrame<'r> {
    renderer: &'r mut Renderer,
    command_encoder: wgpu::CommandEncoder,
    surface_texture: wgpu::SurfaceTexture,
    texture_view: wgpu::TextureView,
}

impl<'r> RendererFrame<'r> {
    pub fn begin_pass(&mut self) -> RenderingContext<'_> {
        let mut render_pass = self
            .command_encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        render_pass.set_bind_group(0, Some(&self.renderer.uniform_bind_group), &[]);
        RenderingContext {
            renderer: self.renderer,
            render_pass,
        }
    }

    pub fn end_frame(self) {
        self.renderer.window.pre_present_notify();
        self.renderer
            .queue
            .submit(Some(self.command_encoder.finish()));
        self.renderer.queue.present(self.surface_texture);
    }
}

pub struct RenderingContext<'f> {
    renderer: &'f mut Renderer,
    render_pass: wgpu::RenderPass<'f>,
}

impl<'f> RenderingContext<'f> {
    pub fn set_texture(&mut self, texture: &GPUTexture) {
        self.render_pass
            .set_bind_group(1, Some(&texture.bind_group), &[]);
    }

    pub fn draw(&mut self, pipeline: &wgpu::RenderPipeline, vertex_buffer: &VertexBuffer) {
        self.render_pass.set_pipeline(&pipeline);

        if self.renderer.uniform_data_dirty {
            self.renderer.queue.write_buffer(
                &self.renderer.uniform_buffer,
                0,
                bytemuck::bytes_of(&self.renderer.uniform_data),
            );
            self.renderer.uniform_data_dirty = false;
        }

        self.render_pass
            .set_vertex_buffer(0, Some(vertex_buffer.wgpu_buffer.slice(..)));
        self.render_pass.draw(0..vertex_buffer.count() as u32, 0..1);
    }
}
