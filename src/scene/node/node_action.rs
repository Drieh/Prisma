use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::{
    node::components::{NodeState, Style, Transform},
    util::{Color, Position},
};

pub type ActionQueue = VecDeque<NodeAction>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    BGColor,
    Layer,
    Position,
    BorderRadius,
    Scale,
    Size,
    Wait,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents visual changes of a node.
pub enum Action {
    BGColor {
        color: Color,
    },
    Layer {
        layer: usize,
    },
    BorderRadius {
        radius: u32,
    },
    Scale {
        x: f32,
        y: f32,
    },
    Position {
        position: Option<Position>,
        absolute: Option<bool>,
    },
    Size {
        width: u32,
        height: u32,
    },
    Wait {
        duration: Duration,
    },
}
impl Action {
    fn get_type(&self) -> ActionType {
        match self {
            Action::BGColor { .. } => ActionType::BGColor,
            Action::Layer { .. } => ActionType::Layer,
            Action::Position { .. } => ActionType::Position,
            Action::BorderRadius { .. } => ActionType::BorderRadius,
            Action::Scale { .. } => ActionType::Scale,
            Action::Size { .. } => ActionType::Size,
            Action::Wait { .. } => ActionType::Wait,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeAction {
    persistent: bool,
    action: Action,
}
impl NodeAction {
    pub fn new(action: Action, persistent: bool) -> Self {
        Self { action, persistent }
    }

    pub fn is_persistent(&self) -> bool {
        self.persistent
    }

    pub fn get_action(&self) -> Action {
        self.action
    }

    pub fn get_type(&self) -> ActionType {
        self.action.get_type()
    }

    pub fn apply_action(
        &self,
        style: &mut Style,
        transform: &mut Transform,
        state: &mut NodeState,
    ) {
        match self.action {
            Action::Position { position, absolute } => {
                if let Some(value) = position {
                    transform.position = value;
                }
                if let Some(value) = absolute {
                    transform.position_absolute = value;
                }
            }
            Action::Size { width, height } => {
                style.size = (width, height);
            }
            Action::Scale { x, y } => {
                transform.scale = (x, y);
            }
            Action::BGColor { color } => {
                style.color = color;
            }
            Action::Layer { layer } => {
                transform.layer = Some(layer);
            }
            Action::BorderRadius { radius } => {
                style.border_radius = radius;
            }
            Action::Wait { duration } => {
                state.waiting_until = Some(Instant::now() + duration);
            }
        }
    }
}
