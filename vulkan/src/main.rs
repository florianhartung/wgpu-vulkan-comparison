use std::num::NonZeroU32;

use common::has_window_and_display_handle::HasWindowAndDisplayHandle;

struct VulkanRenderer {

}

impl common::Renderer for VulkanRenderer {
    fn new(_window: impl HasWindowAndDisplayHandle + 'static, initial_window_size: (u32, u32)) -> Self {
        todo!("init renderer")
    }

    fn render(&mut self, camera: common::Camera) {
        todo!("render")
    }

    fn resize(&mut self, size: (NonZeroU32, NonZeroU32), scale_factor: f64) {
        todo!("handle resize")
    }
    
    fn load_mesh(&mut self, mesh: common::Mesh) {
        todo!()
    }
}

fn main() {
    common::run_app::<VulkanRenderer>();
}