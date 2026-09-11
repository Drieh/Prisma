#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}
impl Size {
    pub fn new() -> Self {
        Self {
            width: 100,
            height: 100,
        }
    }
}
