use std::{any::Any, collections::HashMap, time::Instant};

use crate::{
    app::PrismaError,
    scene::{NodeAction, NodeActionType, NodeID},
};

pub struct NodeState {
    pub id: NodeID,
    pub user_state: HashMap<String, Box<dyn Any>>,
    pub og_state: HashMap<NodeActionType, NodeAction>,
    pub on_active: HashMap<NodeActionType, NodeAction>,
    pub on_hover: HashMap<NodeActionType, NodeAction>,
    pub(crate) waiting_until: Option<Instant>,
    pub(crate) destruction_requested: bool,
}
impl NodeState {
    pub fn new(id: NodeID) -> Self {
        Self {
            id,
            user_state: HashMap::new(),
            og_state: HashMap::new(),
            on_active: HashMap::new(),
            on_hover: HashMap::new(),
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
            .ok_or(PrismaError::NodeStateNotFound((self.id, key.to_string())))
    }

    pub fn get_mut<T: Any>(&mut self, key: &str) -> Result<&mut T, PrismaError> {
        self.user_state
            .get_mut(key)
            .and_then(|value| value.downcast_mut::<T>())
            .ok_or(PrismaError::NodeStateNotFound((self.id, key.to_string())))
    }

    pub fn remove<T: Any>(&mut self, key: &str) -> Result<T, PrismaError> {
        self.user_state
            .remove(key)
            .and_then(|value| value.downcast::<T>().ok())
            .map(|value| *value)
            .ok_or(PrismaError::NodeStateNotFound((self.id, key.to_string())))
    }

    pub fn has_state(&self, key: &str) -> bool {
        self.user_state.contains_key(key)
    }

    pub fn on_active(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.on_active.insert(action.get_type(), *action);
        }
        self
    }

    pub fn on_hover(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.on_hover.insert(action.get_type(), *action);
        }
        self
    }
}
