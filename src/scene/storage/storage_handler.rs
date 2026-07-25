use crate::scene::{
    NodeID,
    storage::{
        ActionQueueHandler, ListenerQueueHandler, StateHandler, StyleHandler, TransformHandler,
        TreeHandler,
    },
};

pub struct StorageHandler<'a> {
    pub tree: TreeHandler<'a>,
    pub state: StateHandler<'a>,
    pub transform: TransformHandler<'a>,
    pub style: StyleHandler<'a>,
    pub action_queue: ActionQueueHandler<'a>,
    pub listener_queue: ListenerQueueHandler<'a>,
}
impl<'a> StorageHandler<'a> {
    pub fn has_node(&self, id: NodeID) -> bool {
        if self.tree.contains(id)
            && self.state.contains(id)
            && self.transform.contains(id)
            && self.style.contains(id)
            && self.action_queue.contains(id)
            && self.listener_queue.contains(id)
        {
            true
        } else {
            false
        }
    }

    pub fn get_nodes_id(&self) -> Vec<NodeID> {
        self.tree.get_nodes_id()
    }

    pub(crate) fn context_insert(&mut self, id: NodeID) {
        self.action_queue.context_insert(id);
        self.listener_queue.context_insert(id);
        self.state.context_insert(id);
        self.style.context_insert(id);
        self.transform.context_insert(id);
        self.tree.context_insert(id);
    }

    pub(crate) fn context_remove(&mut self, id: NodeID) {
        self.action_queue.context_remove(id);
        self.listener_queue.context_remove(id);
        self.state.context_remove(id);
        self.style.context_remove(id);
        self.transform.context_remove(id);
        self.tree.context_remove(id);
    }
}
