use crate::{
    error::PrismaError,
    scene::{NodeID, storage::handler::TreeHandler},
};

impl<'a> TreeHandler<'a> {
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
}
