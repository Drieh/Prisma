use std::any::Any;

use crate::{
    error::PrismaError,
    scene::{NodeID, storage::handler::StateHandler},
};

impl<'a> StateHandler<'a> {
    pub fn get_state<T: Any>(&self, id: NodeID) -> Result<&T, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))?
            .get::<T>()
    }

    pub fn contains_state<T: Any>(&self, id: NodeID) -> Result<bool, PrismaError> {
        Ok(self
            .storage
            .get(&id)
            .ok_or(PrismaError::NodeComponentNotFound(id))?
            .contains::<T>())
    }
}
