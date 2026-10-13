use std::{
    sync::{Arc, Mutex},
    thread,
};

use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

struct VisionState {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

struct AppState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    camera_data: Arc<Mutex<Option<VisionState>>>,
    camera_sampler: wgpu::Sampler,
    camera_texture: wgpu::Texture,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

fn run_camera_thread(camera_data: Arc<Mutex<Option<VisionState>>>) {
    let mut camera = nokhwa::Camera::new(
        nokhwa::utils::CameraIndex::Index(0),
        nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
            nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
        ),
    )
    .unwrap();

    camera.open_stream().unwrap();

    loop {
        let frame = camera.frame().unwrap();
        let buffer = frame
            .decode_image::<nokhwa::pixel_format::RgbFormat>()
            .unwrap();
        let layout = buffer.sample_layout();
        let buffer_bytes = buffer.into_raw();
        let mut buffer_bytes_with_alpha = Vec::with_capacity(buffer_bytes.len() / 3 * 4);
        for pixel in buffer_bytes.chunks(3) {
            buffer_bytes_with_alpha.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
        *camera_data.lock().unwrap() = Some(VisionState {
            width: layout.width,
            height: layout.height,
            buffer: buffer_bytes_with_alpha,
        });
    }
}

impl AppState {
    pub async fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Projector Computer"))
                .unwrap(),
        );

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

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
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

        let size = window.inner_size();

        let camera_data = Arc::new(Mutex::new(None));

        let camera_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let camera_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1920,
                height: 1080,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        let texture_view = camera_texture.create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&camera_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        let state = Self {
            window,
            surface,
            surface_format,
            adapter,
            device,
            queue,
            size,
            camera_data: camera_data.clone(),
            camera_texture,
            camera_sampler,
            render_pipeline,
            bind_group,
        };

        state.configure_surface();
        state.window.request_redraw();

        thread::spawn(move || run_camera_thread(camera_data));

        return state;
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
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![self.surface_format],
            },
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    fn update_camera_texture_data(&self, state: &VisionState, texture: &wgpu::Texture) {
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &state.buffer,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(state.width * 4 as u32),
                rows_per_image: Some(state.height),
            },
            texture.size(),
        );
    }

    fn set_camera_texture(&mut self, state: &VisionState) {
        self.update_camera_texture_data(state, &self.camera_texture);
    }

    fn render(&mut self) {
        self.window.request_redraw();

        let camera_data = self.camera_data.lock().unwrap().take();

        if let Some(state) = &camera_data {
            self.set_camera_texture(state);
        }

        let current_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            _ => panic!(),
        };

        let texture_view = current_texture
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.window.pre_present_notify();
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(current_texture);
    }
}

struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.state = Some(pollster::block_on(AppState::new(event_loop)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
