mod math;
mod mesh;
mod renderer;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

use crate::{
    math::{Vec2, Vec3},
    mesh::MeshVertex,
    renderer::{
        GPUBindGroup, GPURenderProgram, GPUResource, GPUResourceGroup, GPUSampler, GPUTexture,
        GPUUniformBuffer, Renderer, VertexBuffer,
    },
};

const COLOR: Vec3 = Vec3::new(1., 0.2, 0.2);

const VERTICES: &[MeshVertex] = &[
    MeshVertex {
        position: Vec3::new(-1., -1., 0.),
        color: COLOR,
        uv: Vec2::new(0., 1.),
    },
    MeshVertex {
        position: Vec3::new(1., -1., 0.),
        color: COLOR,
        uv: Vec2::new(1., 1.),
    },
    MeshVertex {
        position: Vec3::new(1., 1., 0.),
        color: COLOR,
        uv: Vec2::new(1., 0.),
    },
    MeshVertex {
        position: Vec3::new(1., 1., 0.),
        color: COLOR,
        uv: Vec2::new(1., 0.),
    },
    MeshVertex {
        position: Vec3::new(-1., 1., 0.),
        color: COLOR,
        uv: Vec2::new(0., 0.),
    },
    MeshVertex {
        position: Vec3::new(-1., -1., 0.),
        color: COLOR,
        uv: Vec2::new(0., 1.),
    },
];

#[repr(C)]
#[derive(Copy, Clone, bytemuck::NoUninit)]
struct DisplayInfo {
    aspect: f32,
}

struct BasicBindGroup {
    camera_sampler: GPUSampler,
    camera_texture: GPUTexture,
    display_info_buffer: GPUUniformBuffer<DisplayInfo>,
}

impl GPUResourceGroup for BasicBindGroup {
    fn get_binding_types() -> Vec<wgpu::BindingType> {
        vec![
            GPUSampler::get_binding_type(),
            GPUTexture::get_binding_type(),
            GPUUniformBuffer::<DisplayInfo>::get_binding_type(),
        ]
    }

    fn get_binding_resources(&self) -> Vec<wgpu::BindingResource<'_>> {
        vec![
            self.camera_sampler.get_binding_resource(),
            self.camera_texture.get_binding_resource(),
            self.display_info_buffer.get_binding_resource(),
        ]
    }
}

struct VisionState {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
}

struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
    camera_data: Arc<Mutex<Option<VisionState>>>,
    shader_resources: BasicBindGroup,
    bind_group: GPUBindGroup<BasicBindGroup>,
    vertex_buffer: VertexBuffer<MeshVertex>,
    shader_program: GPURenderProgram<MeshVertex, BasicBindGroup>,
}

fn run_camera_thread(camera_data: Arc<Mutex<Option<VisionState>>>) {
    let cameras = cameras::devices().unwrap();
    for camera in &cameras {
        println!("{:?}: {}", camera.id, camera.name);
    }

    let device = &cameras[0];

    let caps = cameras::probe(device).unwrap();

    let mut format_index = 0;
    let mut max_resolution = 0;
    let mut max_framerate = 0.;
    for i in 0..caps.formats.len() {
        let format = &caps.formats[i];
        let resolution = format.resolution.width * format.resolution.height;
        let framerate = format.framerate_range.max;
        if framerate > max_framerate || (framerate == max_framerate && resolution > max_resolution)
        {
            format_index = i;
            max_resolution = resolution;
            max_framerate = framerate;
        }
    }

    let selected_format = &caps.formats[format_index];

    let camera = cameras::open(
        device,
        cameras::StreamConfig {
            resolution: selected_format.resolution,
            framerate: selected_format.framerate_range.max as u32,
            pixel_format: selected_format.pixel_format,
        },
    )
    .unwrap();

    println!("Using camera config: {:?}", camera.config);

    let mut image = unsafe { apriltag::Image::new_uinit(1920, 1080) }.unwrap();
    let mut detector = apriltag::Detector::builder()
        .add_family_bits(apriltag::Family::tag_standard_52h13(), 1)
        .build()
        .unwrap();

    loop {
        let frame = cameras::next_frame(&camera, Duration::from_secs(1)).unwrap();
        let buffer_bytes = cameras::to_rgba8(&frame).unwrap();
        let mut image_index: usize = 0;
        for pixel in buffer_bytes.chunks(4) {
            image.as_slice_mut()[image_index] =
                ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
            image_index += 1;
        }

        let detections = detector.detect(&image);

        *camera_data.lock().unwrap() = Some(VisionState {
            width: frame.width,
            height: frame.height,
            buffer: buffer_bytes,
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

        let renderer = Renderer::new(event_loop, window.clone()).await;

        let shader_module = &renderer.create_wgsl_shader(include_str!("shader.wgsl"));

        let camera_sampler = GPUSampler::new(&renderer.device);

        let camera_texture = GPUTexture::new(&renderer.device, renderer.queue.clone(), 1920, 1080);

        let display_info_buffer = GPUUniformBuffer::new(
            renderer.device.clone(),
            renderer.queue.clone(),
            &DisplayInfo { aspect: 1. },
        );

        let shader_resources = BasicBindGroup {
            camera_sampler,
            camera_texture,
            display_info_buffer,
        };

        let bind_group = GPUBindGroup::new(renderer.device.clone(), &shader_resources);

        let shader_program = GPURenderProgram::new(
            &renderer.device,
            shader_module,
            "vs_main",
            "fs_main",
            renderer.surface_format,
            bind_group.clone(),
        );

        let vertex_buffer =
            VertexBuffer::new(renderer.device.clone(), renderer.queue.clone(), VERTICES);

        let camera_data = Arc::new(Mutex::new(None));

        let state = Self {
            window,
            renderer,
            camera_data: camera_data.clone(),
            shader_resources,
            bind_group,
            vertex_buffer,
            shader_program,
        };

        state.window.request_redraw();

        thread::spawn(move || run_camera_thread(camera_data));

        return state;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    fn update_camera_texture_data(&self, state: &VisionState) {
        self.shader_resources.camera_texture.set_data(&state.buffer);
    }

    fn render(&mut self) {
        self.window.request_redraw();

        let camera_data = self.camera_data.lock().unwrap().take();

        if let Some(state) = &camera_data {
            self.update_camera_texture_data(state);
        }

        let camera_aspect = 1920. / 1080.;

        let window_aspect = self.renderer.size().width as f32 / self.renderer.size().height as f32;

        let aspect = window_aspect / camera_aspect;

        self.shader_resources
            .display_info_buffer
            .write(&DisplayInfo { aspect });

        let frame = self.renderer.begin_frame();
        if let Some(mut f) = frame {
            {
                let mut ctx = f.begin_pass();
                ctx.draw(&self.shader_program, &self.vertex_buffer, &self.bind_group);
            }
            self.renderer.end_frame(f);
        }
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
