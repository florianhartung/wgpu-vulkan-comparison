#[derive(Debug)]
pub struct Vertex {
    pub xyz: [f32; 3],
}

pub struct Mesh {
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
}