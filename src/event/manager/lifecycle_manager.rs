use crate::{
    event::{LifecycleEvent, NodeCreation, NodeDestruction, NodeUpdate},
    scene::NodeID,
};

pub struct LifecycleManager {
    queue: Vec<LifecycleEvent>,
}
impl LifecycleManager {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn handle_creation(&mut self, target: NodeID) {
        self.queue
            .push(LifecycleEvent::LifecycleCreation(NodeCreation { target }));
    }

    pub fn handle_update(&mut self, target: NodeID) {
        self.queue
            .push(LifecycleEvent::LifecycleUpdate(NodeUpdate { target }));
    }

    pub fn handle_destruction(&mut self, target: NodeID) {
        self.queue
            .push(LifecycleEvent::LifecycleDestruction(NodeDestruction {
                target,
            }));
    }

    pub fn take_events(&mut self) -> Vec<LifecycleEvent> {
        std::mem::take(&mut self.queue)
    }
}
