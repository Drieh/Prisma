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
        self.tree.contains(id)
            && self.state.contains(id)
            && self.transform.contains(id)
            && self.style.contains(id)
            && self.action_queue.contains(id)
            && self.listener_queue.contains(id)
    }

    pub fn get_nodes(&self) -> Vec<NodeID> {
        self.tree.get_nodes()
    }

    pub(crate) fn insert_context(&mut self, id: NodeID) {
        self.action_queue.insert(id);
        self.listener_queue.insert(id);
        self.state.insert(id);
        self.style.insert(id);
        self.transform.insert(id);
        self.tree.insert(id);
    }

    pub(crate) fn remove_context(&mut self, id: NodeID) {
        self.action_queue.remove(id);
        self.listener_queue.remove(id);
        self.state.remove(id);
        self.style.remove(id);
        self.transform.remove(id);
        self.tree.remove(id);
    }
}
