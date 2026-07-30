use crate::event::{EventType, context::EventContext, event_manager::CallbackID};
use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

// Types
pub type ListenerQueue = Vec<NodeListenerAction>;

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
/// An unique representation of a node.
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
