use crate::common::Position;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Position,
    pub rotation: f32,
    pub scale: (f32, f32),
    pub layer: Option<usize>,
}
impl Transform {
    pub fn new() -> Self {
        Self {
            position: Position { x: 0.0, y: 0.0 },
            rotation: 0.0,
            scale: (1.0, 1.0),
            layer: None,
        }
    }
}
impl Add for Transform {
    type Output = Transform;
    fn add(self, rhs: Self) -> Self::Output {
        let self_layer = if let Some(layer) = self.layer {
            layer
        } else {
            0
        };
        let rhs_layer = if let Some(layer) = rhs.layer {
            layer
        } else {
            0
        };

        Transform {
            position: self.position + rhs.position,
            rotation: self.rotation + rhs.rotation,
            scale: (self.scale.0 * rhs.scale.0, self.scale.1 * rhs.scale.1),
            layer: if self_layer > rhs_layer {
                Some(self_layer)
            } else {
                Some(rhs_layer)
            },
        }
    }
}
impl<'a, 'b> Add<&'b Transform> for &'a Transform {
    type Output = Transform;

    fn add(self, rhs: &'b Transform) -> Transform {
        let self_layer = if let Some(layer) = self.layer {
            layer
        } else {
            0
        };
        let rhs_layer = if let Some(layer) = rhs.layer {
            layer
        } else {
            0
        };

        Transform {
            position: self.position + rhs.position,
            rotation: self.rotation + rhs.rotation,
            scale: (self.scale.0 * rhs.scale.0, self.scale.1 * rhs.scale.1),
            layer: if self_layer > rhs_layer {
                Some(self_layer)
            } else {
                Some(rhs_layer)
            },
        }
    }
}
