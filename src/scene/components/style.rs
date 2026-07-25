use crate::util::Color;

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub color: Color,
    pub border_radius: u32,
    pub size: (u32, u32),
}

impl Style {
    pub fn new() -> Self {
        let color = Color::rgba(0, 0, 0, 0);

        Self {
            color,
            border_radius: 0,
            size: (50, 50),
        }
    }
}
