use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_INDEX: AtomicUsize = AtomicUsize::new(0);

pub struct NodePosition {
    pub x: i32,
    pub y: i32,
    index: usize,
}

impl NodePosition {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            index: NodePosition::next_index(),
        }
    }

    fn next_index() -> usize {
        NEXT_INDEX.fetch_add(1, Ordering::Relaxed)
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
