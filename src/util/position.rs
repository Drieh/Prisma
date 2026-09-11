use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
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
