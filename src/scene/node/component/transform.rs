use crate::util::{Position, Scale};
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct NodeTransform {
    pub position: Position,
    pub position_absolute: bool,
    pub rotation: f32,
    pub scale: Scale,
    pub layer: Option<usize>,
}
impl NodeTransform {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            position_absolute: false,
            rotation: 0.0,
            scale: Scale::new(),
            layer: None,
        }
    }
}
impl Add for NodeTransform {
    type Output = NodeTransform;
    fn add(self, rhs: Self) -> Self::Output {
        NodeTransform {
            position: self.position + rhs.position,
            position_absolute: self.position_absolute,
            rotation: self.rotation + rhs.rotation,
            scale: Scale {
                x: self.scale.x * rhs.scale.x,
                y: self.scale.y * rhs.scale.y,
            },
            layer: rhs.layer.or(self.layer),
        }
    }
}
impl Add for &NodeTransform {
    type Output = NodeTransform;
    fn add(self, rhs: &NodeTransform) -> NodeTransform {
        *self + *rhs
    }
}
