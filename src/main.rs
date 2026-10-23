mod math;
mod mesh;
mod renderer;

use core::f32;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

use crate::{
    math::{Vec2, Vec3}, renderer::{Renderer, Texture},
};

struct VisionState {
    buffer: Vec<u8>,
    detections: Vec<[Vec2; 4]>,
}

struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
    camera_data: Arc<Mutex<Option<VisionState>>>,
    camera_texture: Texture,
}

fn run_camera_thread(camera_data: Arc<Mutex<Option<VisionState>>>) {
    let cameras = cameras::devices().unwrap();
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
        let frame = cameras::next_frame(&camera, Duration::from_secs(60)).unwrap();
        let buffer_bytes = cameras::to_rgba8(&frame).unwrap();
        let mut image_index: usize = 0;
        for pixel in buffer_bytes.chunks(4) {
            image.as_slice_mut()[image_index] =
                ((pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3) as u8;
            image_index += 1;
        }

        let raw_detections = detector.detect(&image);
        let mut detections = Vec::with_capacity(raw_detections.len());

        for detection in raw_detections {
            detections.push(detection.corners().map(|c| {
                Vec2::new(
                    (c[0] as f32 - frame.width as f32 / 2.) / (frame.height as f32 / 2.),
                    1. - c[1] as f32 / (frame.height as f32 / 2.),
                )
            }));
        }

        *camera_data.lock().unwrap() = Some(VisionState {
            buffer: buffer_bytes,
            detections,
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

        let camera_texture = renderer.create_texture(1920, 1080);

        let camera_data = Arc::new(Mutex::new(None));

        let state = Self {
            window,
            renderer,
            camera_data: camera_data.clone(),
            camera_texture,
        };

        state.window.request_redraw();

        thread::spawn(move || run_camera_thread(camera_data));

        return state;
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.renderer.resize(new_size);
    }

    fn update_camera_texture_data(&self, state: &VisionState) {
        self.camera_texture.set_data(&state.buffer);
    }

    fn render(&mut self) {
        self.window.request_redraw();

        let camera_data = self.camera_data.lock().unwrap();

        if let Some(state) = &camera_data.as_ref() {
            self.update_camera_texture_data(state);
        }

        self.renderer.reset_transform();
        self.renderer.set_color(Vec3::new(1., 1., 1.));
        self.renderer.set_texture(self.camera_texture.clone());
        let camera_aspect = self.camera_texture.width() as f32 / self.camera_texture.height() as f32;
        self.renderer.rectangle(Vec3::new(-camera_aspect, -1., 0.), Vec2::new(camera_aspect * 2., 2.));
        self.renderer.reset_texture();

        self.renderer.set_color(Vec3::new(1., 0., 0.));
        if let Some(state) = &camera_data.as_ref() {
            for detection in &state.detections {
                self.renderer.quad(
                    detection[0].into(),
                    detection[1].into(),
                    detection[2].into(),
                    detection[3].into(),
                );
            }
        }

        self.renderer.finish_frame();
    }
}

struct App {
    state: Option<AppState>,
    is_occluded: bool
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
                if !self.is_occluded {
                    state.render();
                }
            }
            WindowEvent::Occluded(occluded) => {
                self.is_occluded = occluded;
                if !occluded {
                    state.window.request_redraw();
                }
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
    let mut app = App { state: None, is_occluded: false };
    event_loop.run_app(&mut app).unwrap();
}
