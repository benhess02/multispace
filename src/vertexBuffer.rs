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

fn create_wgpu_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: size as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl<V: GPUVertexData> VertexBuffer<V> {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, vertices: &[V]) -> Self {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        let buffer = create_wgpu_buffer(&device, bytes.len());
        queue.write_buffer(&buffer, 0, bytes);
        Self {
            device,
            queue,
            wgpu_buffer: buffer,
            vertex_count: vertices.len(),
            data: PhantomData {},
        }
    }

    pub fn set_vertices(&mut self, vertices: &[V]) {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        if bytes.len() != self.size() {
            self.wgpu_buffer = create_wgpu_buffer(&self.device, bytes.len());
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
