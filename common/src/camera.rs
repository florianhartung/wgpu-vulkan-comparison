// Pitch=0: horizontal
// Yaw=0: looking to positive x, Yaw=PI: looking to negative x
pub struct Camera {
    pub xyz: (f32, f32, f32),
    pub pitch: f32,
    pub yaw: f32,
}