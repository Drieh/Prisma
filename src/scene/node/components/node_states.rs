use std::{any::Any, collections::HashMap, time::Instant};

use crate::{
    error::PrismaError,
    node::style_view::{StyleCallback, StyleView},
    scene::{
        NodeID,
        node::{ActionType, NodeAction},
    },
};

pub struct NodeState {
    pub id: NodeID,
    pub user_state: HashMap<String, Box<dyn Any>>,
    pub og_style: HashMap<ActionType, NodeAction>,
    pub on_active: Option<Box<StyleCallback>>,
    pub on_hover: Option<Box<StyleCallback>>,

    pub(crate) waiting_until: Option<Instant>,
    pub(crate) destruction_requested: bool,
}
impl NodeState {
    pub fn new(id: NodeID) -> Self {
        Self {
            id,
            user_state: HashMap::new(),
            og_style: HashMap::new(),
            on_active: None,
            on_hover: None,
            waiting_until: None,
            destruction_requested: false,
        }
    }
    pub fn set<T: Any>(&mut self, key: impl Into<String>, value: T) {
        self.user_state.insert(key.into(), Box::new(value));
    }

    pub fn get<T: Any>(&self, key: &str) -> Result<&T, PrismaError> {
        self.user_state
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
            .ok_or(PrismaError::NodeStateNotFound(self.id, key.to_string()))
    }

    pub fn get_mut<T: Any>(&mut self, key: &str) -> Result<&mut T, PrismaError> {
        self.user_state
            .get_mut(key)
            .and_then(|value| value.downcast_mut::<T>())
            .ok_or(PrismaError::NodeStateNotFound(self.id, key.to_string()))
    }

    pub fn remove<T: Any>(&mut self, key: &str) -> Result<T, PrismaError> {
        self.user_state
            .remove(key)
            .and_then(|value| value.downcast::<T>().ok())
            .map(|value| *value)
            .ok_or(PrismaError::NodeStateNotFound(self.id, key.to_string()))
    }

    pub fn has_state(&self, key: &str) -> bool {
        self.user_state.contains_key(key)
    }
}
