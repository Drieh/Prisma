use crate::error::PrismaError;
use crate::node::component::{NodeQueue, NodeState, NodeVisual};
use crate::scene::node::{NodeID, component::NodeTree};
use crate::scene::storage::macros::define_handlers;
use std::collections::HashMap;

define_handlers!(
    { TreeHandler, NodeTree }
    {StateHandler, NodeState}
    {VisualStateHandler, NodeVisual}
    {QueueHandler, NodeQueue}
);

pub struct StorageHandler<'a> {
    pub tree: TreeHandler<'a>,
    pub state: StateHandler<'a>,
    pub queue: QueueHandler<'a>,
    pub visual: VisualStateHandler<'a>,
}
impl<'a> StorageHandler<'a> {
    /// Returns `true` if the node exists.
    pub fn has_node(&self, id: NodeID) -> bool {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
        } = self;
        tree.contains(id) && state.contains(id) && visual_state.contains(id) && queue.contains(id)
    }

    /// Returns a list of all existing nodes.
    pub fn get_nodes(&self) -> Vec<NodeID> {
        self.tree.get_nodes()
    }

    pub(crate) fn insert_context(&mut self, id: NodeID) {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
        } = self;
        queue.insert(id);
        state.insert(id);
        visual_state.insert(id);
        tree.insert(id);
    }

    pub(crate) fn remove_context(&mut self, id: NodeID) {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
        } = self;
        queue.remove(id);
        state.remove(id);
        visual_state.remove(id);
        tree.remove(id);
    }
}
