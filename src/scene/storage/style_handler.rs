use std::collections::HashMap;

use crate::{
    app::PrismaError,
    scene::{NodeID, components::Style},
};

pub struct StyleHandler<'a> {
    pub(crate) storage: &'a mut HashMap<NodeID, Style>,
}

impl<'a> StyleHandler<'a> {
    pub(crate) fn new(storage: &'a mut HashMap<NodeID, Style>) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: NodeID) -> Result<&Style, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))
    }

    pub fn contains(&self, id: NodeID) -> bool {
        self.storage.contains_key(&id)
    }

    pub(crate) fn get_unchecked(&self, id: NodeID) -> &Style {
        self.storage.get(&id).expect("Node component not found!")
    }

    pub(crate) fn get_unchecked_mut(&mut self, id: NodeID) -> &mut Style {
        self.storage
            .get_mut(&id)
            .expect("Node component not found!")
    }

    pub(crate) fn insert(&mut self, id: NodeID) {
        if self.contains(id) {
            panic!("Node component already exists!");
        }
        self.storage.insert(id, Style::new());
    }

    pub(crate) fn remove(&mut self, id: NodeID) {
        self.storage.remove(&id).expect("Node component not found!");
    }
}
