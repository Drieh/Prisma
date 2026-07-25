use crate::util::Position;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Position,
    pub position_absolute: bool,
    pub rotation: f32,
    pub scale: (f32, f32),
    pub layer: Option<usize>,
}
impl Transform {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            position_absolute: false,
            rotation: 0.0,
            scale: (1.0, 1.0),
            layer: None,
        }
    }
}
impl Add for Transform {
    type Output = Transform;
    fn add(self, rhs: Self) -> Self::Output {
        Transform {
            position: self.position + rhs.position,
            position_absolute: self.position_absolute,
            rotation: self.rotation + rhs.rotation,
            scale: (self.scale.0 * rhs.scale.0, self.scale.1 * rhs.scale.1),
            layer: rhs.layer.or(self.layer),
        }
    }
}
impl<'a, 'b> Add<&'b Transform> for &'a Transform {
    type Output = Transform;
    fn add(self, rhs: &'b Transform) -> Transform {
        *self + *rhs
    }
}
