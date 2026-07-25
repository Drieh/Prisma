use crate::{
    event::{EventContext, EventType, managers::event_manager::CallbackID},
    util::{Color, Position},
};
use std::{fmt::Display, sync::atomic::AtomicU32, sync::atomic::Ordering, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeActionType {
    BGColor,
    Layer,
    Position,
    BorderRadius,
    Scale,
    Size,
    Wait,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeAction {
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
impl NodeAction {
    pub fn get_type(&self) -> NodeActionType {
        match self {
            NodeAction::BGColor { .. } => NodeActionType::BGColor,
            NodeAction::Layer { .. } => NodeActionType::Layer,
            NodeAction::Position { .. } => NodeActionType::Position,
            NodeAction::BorderRadius { .. } => NodeActionType::BorderRadius,
            NodeAction::Scale { .. } => NodeActionType::Scale,
            NodeAction::Size { .. } => NodeActionType::Size,
            NodeAction::Wait { .. } => NodeActionType::Wait,
        }
    }
}
pub enum NodeListenerAction {
    Add {
        event_type: EventType,
        callback: Box<dyn FnMut(&mut EventContext) + 'static>,
    },
    Remove {
        target: CallbackID,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NodeID(u32);

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

impl NodeID {
    pub fn id(id: u32) -> Self {
        Self(id)
    }
    pub(crate) fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
