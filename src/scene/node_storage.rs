use std::collections::{HashMap, VecDeque};

use crate::{
    app::PrismaError,
    scene::{
        NodeAction, NodeID, NodeListenerAction, NodeView,
        components::{NodeState, Style, Transform, TreeNode},
        storage::{
            ActionQueueHandler, ListenerQueueHandler, StateHandler, StorageHandler, StyleHandler,
            TransformHandler, TreeHandler,
        },
    },
};

// Types
pub type ListenerQueue = Vec<NodeListenerAction>;
pub type ActionQueue = VecDeque<NodeAction>;

pub struct NodeStorage {
    tree: HashMap<NodeID, TreeNode>,
    style: HashMap<NodeID, Style>,
    state: HashMap<NodeID, NodeState>,
    transform: HashMap<NodeID, Transform>,
    action_queue: HashMap<NodeID, ActionQueue>,
    listener_queue: HashMap<NodeID, ListenerQueue>,
}
impl NodeStorage {
    pub fn new() -> Self {
        Self {
            tree: HashMap::new(),
            style: HashMap::new(),
            state: HashMap::new(),
            transform: HashMap::new(),
            action_queue: HashMap::new(),
            listener_queue: HashMap::new(),
        }
    }

    pub fn new_node(&mut self) -> NodeView<'_> {
        let id = NodeID::next();

        self.get_handler().context_insert(id);

        self.get_node(id).expect("Node creation failed!")
    }

    pub fn exists(&self, id: NodeID) -> bool {
        if self.tree.contains_key(&id)
            && self.state.contains_key(&id)
            && self.transform.contains_key(&id)
            && self.style.contains_key(&id)
            && self.action_queue.contains_key(&id)
            && self.listener_queue.contains_key(&id)
        {
            true
        } else {
            false
        }
    }

    pub fn get_handler(&mut self) -> StorageHandler<'_> {
        StorageHandler {
            tree: TreeHandler::new(&mut self.tree),
            state: StateHandler::new(&mut self.state),
            transform: TransformHandler::new(&mut self.transform),
            style: StyleHandler::new(&mut self.style),
            action_queue: ActionQueueHandler::new(&mut self.action_queue),
            listener_queue: ListenerQueueHandler::new(&mut self.listener_queue),
        }
    }

    pub fn get_node(&mut self, id: NodeID) -> Result<NodeView<'_>, PrismaError> {
        NodeView::new(id, self)
    }

    pub fn get_nodes_id(&self) -> Vec<NodeID> {
        self.tree.keys().copied().collect()
    }

    pub fn destroy_node(&mut self, id: NodeID) -> Result<(), PrismaError> {
        if !self.exists(id) {
            return Err(PrismaError::NodeNotFound(id));
        }

        self.get_handler().context_remove(id);
        Ok(())
    }

    pub fn take_listener_queue(&mut self, id: NodeID) -> Vec<NodeListenerAction> {
        std::mem::take(&mut self.listener_queue.get_mut(&id).unwrap())
    }
}
