use std::{any::Any, collections::HashMap};

use crate::{
    error::PrismaError,
    scene::{NodeID, node::components::NodeState},
};

pub struct StateHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, NodeState>,
}

impl<'a> StateHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, NodeState>) -> Self {
        Self { storage }
    }

    pub fn get<T: Any>(&self, id: NodeID, key: &str) -> Result<&T, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))?
            .get(key)
    }

    pub fn has_state(&self, id: NodeID, key: &str) -> Result<bool, PrismaError> {
        Ok(self
            .storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))?
            .has_state(key))
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn get_unchecked(&self, id: NodeID) -> &NodeState {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut NodeState {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.insert(id, NodeState::new(id));
    }

    pub(crate) fn remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
