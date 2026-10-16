use std::{borrow::Cow, sync::Arc};

use std::marker::PhantomData;

pub trait GPUVertexData: bytemuck::NoUninit {
    fn get_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub struct VertexBuffer<V: GPUVertexData> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub wgpu_buffer: wgpu::Buffer,
    vertex_count: usize,
    data: PhantomData<V>,
}

impl<V: GPUVertexData> VertexBuffer<V> {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, vertices: &[V]) -> Self {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        let buffer = Self::create_wgpu_buffer(&device, bytes.len());
        queue.write_buffer(&buffer, 0, bytes);
        Self {
            device,
            queue,
            wgpu_buffer: buffer,
            vertex_count: vertices.len(),
            data: PhantomData {},
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

    pub fn set_vertices(&mut self, vertices: &[V]) {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.len() != self.size() {
            self.wgpu_buffer = Self::create_wgpu_buffer(&self.device, bytes.len());
        }
        self.queue.write_buffer(&self.wgpu_buffer, 0, &bytes);
        self.vertex_count = vertices.len();
    }

    pub fn count(&self) -> usize {
        self.vertex_count
    }

    pub fn size(&self) -> usize {
        self.wgpu_buffer.size() as usize
    }
}

pub trait GPUResource {
    fn get_binding_type() -> wgpu::BindingType;
    fn get_binding_resource(&self) -> wgpu::BindingResource<'_>;
}

pub trait GPUResourceGroup {
    fn get_binding_types() -> Vec<wgpu::BindingType>;
    fn get_binding_resources(&self) -> Vec<wgpu::BindingResource<'_>>;
}

impl<R: GPUResource> GPUResourceGroup for R {
    fn get_binding_types() -> Vec<wgpu::BindingType> {
        vec![Self::get_binding_type()]
    }

    fn get_binding_resources(&self) -> Vec<wgpu::BindingResource<'_>> {
        vec![self.get_binding_resource()]
    }
}

pub struct GPUUniformBuffer<T: bytemuck::NoUninit> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    wgpu_buffer: wgpu::Buffer,
    data: PhantomData<T>,
}

impl<T: bytemuck::NoUninit> GPUUniformBuffer<T> {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, data: &T) -> Self {
        let bytes: &[u8] = bytemuck::bytes_of(data);
        let wgpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            device,
            queue,
            wgpu_buffer,
            data: PhantomData {},
        }
    }

    pub fn write(&self, data: &T) {
        let bytes: &[u8] = bytemuck::bytes_of(data);
        self.queue.write_buffer(&self.wgpu_buffer, 0, &bytes);
    }
}

impl<T: bytemuck::NoUninit> GPUResource for GPUUniformBuffer<T> {
    fn get_binding_type() -> wgpu::BindingType {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(size_of::<T>() as u64),
        }
    }

    fn get_binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &self.wgpu_buffer,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<T>() as u64),
        })
    }
}

pub struct GPUSampler {
    wgpu_sampler: wgpu::Sampler,
}

impl GPUSampler {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            wgpu_sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
        }
    }
}

impl GPUResource for GPUSampler {
    fn get_binding_type() -> wgpu::BindingType {
        wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    }

    fn get_binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::Sampler(&self.wgpu_sampler)
    }
}

pub struct GPUTexture {
    queue: wgpu::Queue,
    wgpu_texture: wgpu::Texture,
    wgpu_texture_view: wgpu::TextureView,
}

impl GPUTexture {
    pub fn new(device: &wgpu::Device, queue: wgpu::Queue, width: u32, height: u32) -> Self {
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
        Self {
            queue,
            wgpu_texture: texture,
            wgpu_texture_view: view,
        }
    }

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

impl GPUResource for GPUTexture {
    fn get_binding_type() -> wgpu::BindingType {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }

    fn get_binding_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.wgpu_texture_view)
    }
}

pub struct GPUBindGroup<G: GPUResourceGroup> {
    device: wgpu::Device,
    layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
    group: PhantomData<G>,
}

impl<G: GPUResourceGroup> GPUBindGroup<G> {
    pub fn uninit(device: wgpu::Device) -> Self {
        let types = G::get_binding_types();
        let mut entries = Vec::with_capacity(types.len());
        let mut index = 0;
        for ty in types {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: index,
                visibility: wgpu::ShaderStages::all(),
                ty,
                count: None,
            });
            index += 1;
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });

        Self {
            device,
            layout: bind_group_layout,
            bind_group: None,
            group: PhantomData,
        }
    }

    pub fn new(device: wgpu::Device, group: &G) -> Self {
        let mut bind_group = Self::uninit(device);
        bind_group.write(group);
        return bind_group;
    }

    pub fn write(&mut self, group: &G) {
        let resources = group.get_binding_resources();
        let mut entries = Vec::with_capacity(resources.len());
        let mut index = 0;
        for r in resources {
            entries.push(wgpu::BindGroupEntry {
                binding: index,
                resource: r,
            });
            index += 1;
        }
        self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &entries,
        }));
    }
}

impl<G: GPUResourceGroup> Clone for GPUBindGroup<G> {
    fn clone(&self) -> Self {
        Self {
            device: self.device.clone(),
            layout: self.layout.clone(),
            bind_group: self.bind_group.clone(),
            group: self.group.clone(),
        }
    }
}

pub struct GPURenderProgram<V: GPUVertexData, R: GPUResourceGroup> {
    pub pipeline: wgpu::RenderPipeline,
    vertex_data: PhantomData<V>,
    resource_data: PhantomData<R>,
}

impl<V: GPUVertexData, R: GPUResourceGroup> GPURenderProgram<V, R> {
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        vertex_main: &str,
        fragment_main: &str,
        output_format: wgpu::TextureFormat,
        bind_group: GPUBindGroup<R>,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group.layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(vertex_main),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(V::get_layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment_main),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            vertex_data: PhantomData,
            resource_data: PhantomData,
        }
    }
}

pub struct Renderer {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
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

        let renderer = Self {
            window,
            surface,
            surface_format,
            device,
            queue,
            size,
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
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn create_wgsl_shader(&self, source: &str) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
            })
    }

    pub fn begin_frame(&self) -> Option<RendererFrame> {
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
            command_encoder: encoder,
            surface_texture: current_texture,
            texture_view,
        });
    }

    pub fn end_frame(&self, frame: RendererFrame) {
        self.window.pre_present_notify();
        self.queue.submit(Some(frame.command_encoder.finish()));
        self.queue.present(frame.surface_texture);
    }
}

pub struct RendererFrame {
    command_encoder: wgpu::CommandEncoder,
    surface_texture: wgpu::SurfaceTexture,
    texture_view: wgpu::TextureView,
}

impl RendererFrame {
    pub fn begin_pass(&mut self) -> RenderingContext<'_> {
        let render_pass = self
            .command_encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
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
        RenderingContext { render_pass }
    }
}

pub struct RenderingContext<'f> {
    render_pass: wgpu::RenderPass<'f>,
}

impl<'f> RenderingContext<'f> {
    pub fn draw<V: GPUVertexData, G: GPUResourceGroup>(
        &mut self,
        program: &GPURenderProgram<V, G>,
        vertex_buffer: &VertexBuffer<V>,
        bind_group: &GPUBindGroup<G>,
    ) {
        self.render_pass.set_pipeline(&program.pipeline);
        self.render_pass
            .set_vertex_buffer(0, Some(vertex_buffer.wgpu_buffer.slice(..)));
        self.render_pass
            .set_bind_group(0, &bind_group.bind_group, &[]);
        self.render_pass.draw(0..vertex_buffer.count() as u32, 0..1);
    }
}
