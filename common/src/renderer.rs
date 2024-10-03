use std::num::NonZeroU32;

use crate::{Camera, HasWindowAndDisplayHandle, Mesh};

pub trait Renderer {
    fn new(window: impl HasWindowAndDisplayHandle + Send + Sync + 'static, initial_window_size: (u32, u32)) -> Self;
    fn render(&mut self, camera: Camera);
    fn resize(&mut self, size: (NonZeroU32, NonZeroU32), scale_factor: f64);
    fn load_mesh(&mut self, mesh: Mesh);
}