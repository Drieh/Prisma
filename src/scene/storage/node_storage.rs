use std::collections::HashMap;

use crate::{
    error::PrismaError,
    node::component::{NodePosition, NodeQueue, NodeVisual},
    scene::{
        NodeID,
        node::{
            Node,
            component::{NodeState, NodeTree},
        },
        storage::{
            StorageHandler,
            handler::{
                PositionHandler, QueueHandler, StateHandler, TreeHandler, VisualStateHandler,
            },
        },
    },
    util::DisplayLayout,
};

pub struct NodeStorage {
    tree: HashMap<NodeID, NodeTree>,
    visual: HashMap<NodeID, NodeVisual>,
    state: HashMap<NodeID, NodeState>,
    queue: HashMap<NodeID, NodeQueue>,
    position: HashMap<NodeID, NodePosition>,
}
impl NodeStorage {
    pub(crate) fn new() -> Self {
        Self {
            tree: HashMap::new(),
            state: HashMap::new(),
            visual: HashMap::new(),
            queue: HashMap::new(),
            position: HashMap::new(),
        }
    }

    pub fn new_node(&mut self) -> Node<'_> {
        let id = NodeID::next();
        self.storage().insert_context(id);

        self.get_node(id).expect("Node creation failed!")
    }

    pub fn contains(&mut self, id: NodeID) -> bool {
        self.storage().contains(id)
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        StorageHandler {
            tree: TreeHandler::new(&mut self.tree),
            state: StateHandler::new(&mut self.state),
            queue: QueueHandler::new(&mut self.queue),
            visual: VisualStateHandler::new(&mut self.visual),
            position: PositionHandler::new(&mut self.position),
        }
    }

    pub fn get_node(&mut self, id: NodeID) -> Result<Node<'_>, PrismaError> {
        Node::new(id, self)
    }

    pub fn get_nodes(&self) -> Vec<NodeID> {
        self.tree
            .keys()
            .filter_map(|id| {
                self.visual
                    .get(id)
                    .and_then(|node_visual| Some(node_visual.get_display()))
                    .filter(|display| display != &DisplayLayout::None)
                    .and_then(|_| Some(*id))
            })
            .collect()
    }

    pub fn get_all_nodes(&self) -> Vec<NodeID> {
        self.tree.keys().cloned().collect()
    }

    pub fn destroy_node(&mut self, id: NodeID) -> Result<(), PrismaError> {
        if !self.contains(id) {
            return Err(PrismaError::NodeNotFound(id));
        }

        self.storage().remove_context(id);
        Ok(())
    }

    pub fn get_node_position(&self) {
        let _nodes = self.get_nodes();
    }
}
