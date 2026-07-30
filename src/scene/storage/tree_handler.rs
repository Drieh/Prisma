use std::collections::HashMap;

use crate::{
    error::PrismaError,
    scene::{NodeID, node::components::Tree},
};

pub struct TreeHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, Tree>,
}

impl<'a> TreeHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, Tree>) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: NodeID) -> Result<&Tree, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))
    }

    pub(crate) fn get_nodes(&self) -> Vec<NodeID> {
        self.storage.keys().cloned().collect()
    }

    pub fn get_family(&self, id: NodeID) -> Result<Vec<NodeID>, PrismaError> {
        let mut family: Vec<NodeID> = Vec::new();
        self.tree_get_family(id, &mut family)?;
        Ok(family)
    }

    fn tree_get_family(&self, id: NodeID, output: &mut Vec<NodeID>) -> Result<(), PrismaError> {
        output.push(id);
        for child_id in self
            .storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))?
            .get_children()
        {
            self.tree_get_family(child_id, output)?;
        }
        Ok(())
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn get_unchecked(&self, id: NodeID) -> &Tree {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut Tree {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.insert(id, Tree::new(id));
    }

    pub(crate) fn remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
