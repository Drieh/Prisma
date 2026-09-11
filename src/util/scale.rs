#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}
impl Scale {
    pub fn new() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}
