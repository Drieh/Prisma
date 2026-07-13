use sdl3::pixels::Color;

#[derive(Debug, Clone)]
pub struct Style {
    pub color: Color,
}

impl Style {
    pub fn new() -> Self {
        let color = Color::RGBA(0, 0, 0, 0);

        Self { color }
    }
}
