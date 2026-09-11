use std::collections::HashMap;

use crate::{
    error::PrismaError,
    node::{
        ActionOrigin,
        component::{NodeQueue, NodeText, NodeVisual},
    },
    scene::{
        NodeID,
        node::{
            NodeView,
            component::{NodeState, NodeTree},
        },
        storage::{
            StorageHandler,
            handler::{QueueHandler, StateHandler, TextHandler, TreeHandler, VisualStateHandler},
        },
    },
};

pub struct NodeStorage {
    tree: HashMap<NodeID, NodeTree>,
    visual_state: HashMap<NodeID, NodeVisual>,
    state: HashMap<NodeID, NodeState>,
    queue: HashMap<NodeID, NodeQueue>,
    text: HashMap<NodeID, NodeText>,
}
impl NodeStorage {
    pub(crate) fn new() -> Self {
        Self {
            tree: HashMap::new(),
            state: HashMap::new(),
            visual_state: HashMap::new(),
            queue: HashMap::new(),
            text: HashMap::new(),
        }
    }

    pub fn new_node(&mut self, origin: ActionOrigin) -> NodeView<'_> {
        let id = NodeID::next();
        self.storage().insert_context(id);
        self.get_node_view(id, origin)
            .expect("Node creation failed!")
    }

    pub fn exists(&mut self, id: NodeID) -> bool {
        self.storage().has_node(id)
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        StorageHandler {
            tree: TreeHandler::new(&mut self.tree),
            state: StateHandler::new(&mut self.state),
            queue: QueueHandler::new(&mut self.queue),
            text: TextHandler::new(&mut self.text),
            visual: VisualStateHandler::new(&mut self.visual_state),
        }
    }

    pub fn get_state(&mut self) -> StateHandler<'_> {
        StateHandler::new(&mut self.state)
    }

    pub fn get_node_view(
        &mut self,
        id: NodeID,
        origin: ActionOrigin,
    ) -> Result<NodeView<'_>, PrismaError> {
        NodeView::new(id, self, origin)
    }

    pub fn get_nodes_id(&self) -> Vec<NodeID> {
        self.tree.keys().copied().collect()
    }

    pub fn destroy_node(&mut self, id: NodeID) -> Result<(), PrismaError> {
        if !self.exists(id) {
            return Err(PrismaError::NodeNotFound(id));
        }

        self.storage().remove_context(id);
        Ok(())
    }
}
