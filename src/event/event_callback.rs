use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::event::context::EventContext;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct EventCallbackID(u32);
static NEXT_LISTENER_ID: AtomicU32 = AtomicU32::new(0);
impl EventCallbackID {
    pub fn id(id: u32) -> Self {
        Self(id)
    }

    pub(crate) fn next() -> Self {
        Self(NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for EventCallbackID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct EventCallback {
    pub callback: Box<dyn FnMut(&mut EventContext) + 'static>,
}
impl EventCallback {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }

    pub fn execute(&mut self, context: &mut EventContext) {
        (self.callback)(context);
    }
}
