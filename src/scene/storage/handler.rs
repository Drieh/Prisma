use crate::error::PrismaError;
use crate::node::component::{NodeQueue, NodeState, NodeText, NodeVisual};
use crate::scene::node::{NodeID, component::NodeTree};
use crate::scene::storage::macros::define_handlers;
use std::collections::HashMap;

define_handlers!(
    { TreeHandler, NodeTree }
    {StateHandler, NodeState}
    {VisualStateHandler, NodeVisual}
    {QueueHandler, NodeQueue}
    {TextHandler, NodeText}
);

pub struct StorageHandler<'a> {
    pub tree: TreeHandler<'a>,
    pub state: StateHandler<'a>,
    pub queue: QueueHandler<'a>,
    pub text: TextHandler<'a>,
    pub visual: VisualStateHandler<'a>,
}
impl<'a> StorageHandler<'a> {
    pub fn has_node(&self, id: NodeID) -> bool {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
            text,
        } = self;
        tree.contains(id)
            || state.contains(id)
            || visual_state.contains(id)
            || queue.contains(id)
            || text.contains(id)
    }

    pub(crate) fn get_nodes(&self) -> Vec<NodeID> {
        self.tree.get_nodes()
    }

    pub(crate) fn insert_context(&mut self, id: NodeID) {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
            text,
        } = self;
        queue.insert(id);
        state.insert(id);
        visual_state.insert(id);
        tree.insert(id);
        text.insert(id);
    }

    pub(crate) fn remove_context(&mut self, id: NodeID) {
        let Self {
            tree,
            state,
            visual: visual_state,
            queue,
            text,
        } = self;
        queue.remove(id);
        state.remove(id);
        visual_state.remove(id);
        tree.remove(id);
        text.remove(id);
    }
}
