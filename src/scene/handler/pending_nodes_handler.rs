use crate::{event::EventContext, node::NodeID};

pub struct PendingNodesHandler {
    created: Vec<NodeID>,
    destroyed: Vec<NodeID>,
}
impl PendingNodesHandler {
    pub fn new() -> Self {
        Self {
            created: Vec::new(),
            destroyed: Vec::new(),
        }
    }

    pub fn extend_pending(&mut self, context: &mut EventContext) {
        self.created.extend(context.take_created_nodes());
        self.destroyed.extend(context.take_destroyed_nodes());
    }

    pub fn get_created(&self) -> &Vec<NodeID> {
        &self.created
    }

    pub fn get_destroyed(&self) -> &Vec<NodeID> {
        &self.destroyed
    }

    pub fn take_created(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.created)
    }

    pub fn take_destroyed(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.destroyed)
    }

    pub fn push_created(&mut self, id: NodeID) {
        self.created.push(id);
    }

    pub fn push_destroyed(&mut self, id: NodeID) {
        self.destroyed.push(id);
    }

    pub fn destroyed_contains(&self, id: NodeID) -> bool {
        self.destroyed.contains(&id)
    }
}
