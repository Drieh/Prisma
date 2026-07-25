use std::collections::{HashMap, VecDeque};

use crate::{
    app::PrismaError,
    scene::{NodeAction, NodeID, node_storage::ActionQueue},
};

pub struct ActionQueueHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, ActionQueue>,
}
impl<'a> ActionQueueHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, VecDeque<NodeAction>>) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: NodeID) -> Result<&ActionQueue, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn context_get(&self, id: NodeID) -> &ActionQueue {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn context_get_mut(&mut self, id: NodeID) -> &mut ActionQueue {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn context_insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.entry(id).insert_entry(VecDeque::new());
    }

    pub(crate) fn context_remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
