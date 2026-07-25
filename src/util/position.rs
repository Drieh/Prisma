use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}
impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
