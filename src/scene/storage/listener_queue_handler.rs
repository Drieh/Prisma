use std::collections::HashMap;

use crate::scene::node_storage::ListenerQueue;
use crate::{app::PrismaError, scene::NodeID};
pub struct ListenerQueueHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, ListenerQueue>,
}
impl<'a> ListenerQueueHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, ListenerQueue>) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: NodeID) -> Result<&ListenerQueue, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn get_unchecked(&self, id: NodeID) -> &ListenerQueue {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut ListenerQueue {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.insert(id, Vec::new());
    }

    pub(crate) fn remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
