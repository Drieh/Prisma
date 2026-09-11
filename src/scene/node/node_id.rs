use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

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
