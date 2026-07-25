use std::collections::HashMap;

use crate::{
    error::PrismaError,
    scene::{NodeID, node::components::Transform},
};

pub struct TransformHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, Transform>,
}

impl<'a> TransformHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, Transform>) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: NodeID) -> Result<&Transform, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn get_unchecked(&self, id: NodeID) -> &Transform {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut Transform {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.insert(id, Transform::new());
    }
    pub(crate) fn remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
