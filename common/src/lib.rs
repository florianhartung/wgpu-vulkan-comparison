//! A library that provides basic code to get a window up and running and to manage the app state.
//! This library defines a [`Renderer`] trait that has to be implemented for some rendering backend.
//! Then one can call [`run_app`] with a specific [`Renderer`] implementor.

mod windowing;

pub mod camera;
use std::{f32::consts::PI, time::Instant};

use bezier_nd::Bezier;
pub use camera::*;
pub mod mesh;
pub use mesh::*;
pub mod has_window_and_display_handle;
pub use has_window_and_display_handle::*;
pub mod renderer;
pub use renderer::*;
use windowing::Event;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use geo_nd::{FArray, Vector};

struct Application<R> {
    init_time: Instant,
    frames: u64,
    renderer: R,
}

impl<R: Renderer> windowing::Application for Application<R> {
    fn new(
        window: impl HasWindowAndDisplayHandle + Send + Sync + 'static,
        initial_window_size: (u32, u32),
    ) -> Self {
        let renderer = R::new(window, initial_window_size);

        Self {
            init_time: Instant::now(),
            frames: 0,
            renderer,
        }
    }

    fn handle_event(&mut self, event: windowing::Event) {
        match event {
            Event::Render => {
                const DURATION_SECS: f32 = 5.0;
                let current_time = self.init_time.elapsed().as_secs_f32() / DURATION_SECS;

                if current_time > 1.0 {
                    let average_fps = self.frames as f32 / DURATION_SECS;
                    println!("Average FPS over {DURATION_SECS}s: {average_fps}");
                    self.init_time = Instant::now();
                    self.frames = 0;
                    return;
                }

                let dx = FArray::<f32, 5>::from_array([0.0, 0.0, 10.0, 0.0, PI*3.0/2.0]);
                let dy = FArray::<f32, 5>::from_array([10.0, 0.0, 0.0, 0.0, PI]);
                let line = Bezier::line(&dx, &dy);
                let x = line.point_at(current_time);


                self.frames += 1;
                self.renderer.render(Camera { xyz: (x[0], x[1], x[2]), pitch: x[3], yaw: x[4]});
            }
            Event::Resize { size, scale_factor } => self.renderer.resize(size, scale_factor),
            Event::KeyboardInput(KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::Space),
                state: ElementState::Pressed,
                ..
            }) => {
                let random_float = || rand::random::<f32>() * 2.0 - 1.0;

                let vertices = (0..3)
                    .map(|_| Vertex {
                        xyz: [random_float(), random_float(), random_float()],
                    })
                    .collect();

                let indices = vec![0, 1, 2];

                self.renderer.load_mesh(Mesh { indices, vertices });
            }
            _ => {}
        }
    }
}

pub fn run_app<R: Renderer>() {
    windowing::run_window_app::<Application<R>>();
}
