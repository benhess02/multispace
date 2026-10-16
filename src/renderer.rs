use std::{borrow::Cow, sync::Arc};

use crate::vertexBuffer::{
    GPUBindGroup, GPURenderProgram, GPUResourceGroup, GPUVertexData, VertexBuffer,
};

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
