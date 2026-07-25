use std::{any::Any, collections::VecDeque, time::Duration};

use sdl3::pixels::Color;

use crate::{
    app::PrismaError,
    event::{EventContext, EventType, managers::event_manager::CallbackID},
    scene::{
        NodeAction, NodeID, NodeStorage,
        components::{NodeState, Style, Transform, TreeNode},
        node::NodeListenerAction,
        storage::StorageHandler,
    },
    util::Position,
};

pub struct NodeView<'a> {
    id: NodeID,
    storage: StorageHandler<'a>,
}
impl<'a> NodeView<'a> {
    pub fn new(id: NodeID, nodes: &'a mut NodeStorage) -> Result<Self, PrismaError> {
        if nodes.exists(id) {
            Ok(Self {
                id,
                storage: nodes.get_handler(),
            })
        } else {
            Err(PrismaError::NodeNotFound(id))
        }
    }
    // getters

    pub fn get_id(&self) -> NodeID {
        self.id
    }
    pub fn get_tree(&self) -> &TreeNode {
        self.storage.tree.get(self.id).unwrap()
    }
    pub fn get_style(&self) -> &Style {
        self.storage.style.get(self.id).unwrap()
    }
    pub fn get_transform(&self) -> &Transform {
        self.storage.transform.get(self.id).unwrap()
    }
    pub fn get_node_state(&self) -> &NodeState {
        self.storage.state.context_get(self.id)
    }
    pub fn get_action_queue(&self) -> &VecDeque<NodeAction> {
        self.storage.action_queue.context_get(self.id)
    }
    pub fn get_listener_action_queue(&self) -> &Vec<NodeListenerAction> {
        self.storage.listener_queue.context_get(self.id)
    }

    pub(crate) fn get_tree_mut(&mut self) -> &mut TreeNode {
        self.storage.tree.context_get_mut(self.id)
    }
    pub(crate) fn get_style_mut(&mut self) -> &mut Style {
        self.storage.style.context_get_mut(self.id)
    }
    pub(crate) fn get_transform_mut(&mut self) -> &mut Transform {
        self.storage.transform.context_get_mut(self.id)
    }
    pub(crate) fn get_node_state_mut(&mut self) -> &mut NodeState {
        self.storage.state.context_get_mut(self.id)
    }
    pub(crate) fn get_action_queue_mut(&mut self) -> &mut VecDeque<NodeAction> {
        self.storage.action_queue.context_get_mut(self.id)
    }
    pub(crate) fn get_listener_action_queue_mut(&mut self) -> &mut Vec<NodeListenerAction> {
        self.storage.listener_queue.context_get_mut(self.id)
    }

    // states
    pub fn set_state<T: Any>(&mut self, key: impl Into<String>, value: T) {
        self.get_node_state_mut()
            .user_state
            .insert(key.into(), Box::new(value));
    }

    pub fn get_state_mut<T: Any>(&mut self, key: &str) -> Result<&mut T, PrismaError> {
        self.get_node_state_mut().get_mut(key)
    }

    pub fn get_state<T: Any>(&self, key: &str) -> Result<&T, PrismaError> {
        self.get_node_state().get(key)
    }

    pub fn has_state(&self, key: &str) -> bool {
        self.get_node_state().has_state(key)
    }

    pub fn remove_state<T: Any>(&mut self, key: &str) -> Result<T, PrismaError> {
        self.get_node_state_mut().remove(key)
    }

    // node
    pub fn destroy(&mut self) {
        self.get_node_state_mut().destruction_requested = true;
    }

    pub fn add_child(&mut self, child_id: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child_id) {
            return Err(PrismaError::NodeNotFound(child_id));
        }
        if self.id == child_id {
            return Err(PrismaError::InvalidTreeState((self.id, child_id)));
        }
        if let Some(parent) = self.get_parent()
            && parent == child_id
        {
            return Err(PrismaError::InvalidTreeState((self.id, child_id)));
        }
        let self_id = self.id;
        self.storage
            .tree
            .context_get_mut(child_id)
            .set_parent(Some(self_id));
        self.get_tree_mut().add_child(child_id);
        Ok(self)
    }

    pub fn remove_child(&mut self, child_id: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child_id) {
            return Err(PrismaError::NodeNotFound(child_id));
        }
        self.get_tree_mut().remove_child(child_id);
        self.storage.tree.context_get_mut(child_id).set_parent(None);

        Ok(self)
    }

    pub fn get_parent(&self) -> Option<NodeID> {
        self.get_tree().get_parent()
    }

    pub fn get_children(&self) -> Vec<NodeID> {
        self.get_tree().get_children()
    }

    pub fn get_family(&mut self, target: NodeID, output: &mut Vec<NodeID>) {
        output.push(target);
        for child_id in self.get_children() {
            self.get_family(child_id, output);
        }
    }

    pub fn get_bouding_box_size(&self) -> (u32, u32) {
        (
            (self.get_style().size.0 as f32 * self.get_transform().scale.0)
                .abs()
                .round() as u32,
            (self.get_style().size.1 as f32 * self.get_transform().scale.1)
                .abs()
                .round() as u32,
        )
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.get_style().size
    }

    pub fn get_world_position(&mut self) -> Position {
        self.world_position(self.id).unwrap()
    }

    fn world_position(&mut self, id: NodeID) -> Result<Position, PrismaError> {
        if !self.storage.has_node(id) {
            return Err(PrismaError::NodeNotFound(id));
        }
        let self_transform = self.storage.transform.context_get(id).clone();
        if !self_transform.position_absolute
            && let tree = self.storage.tree.context_get(id).clone()
            && let Some(parent) = tree.get_parent()
        {
            Ok(self.world_position(parent)? + self_transform.position)
        } else {
            Ok(self_transform.position)
        }
    }

    // Queued actions

    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.get_action_queue_mut().push_back(NodeAction::Position {
            position: Some(Position {
                x: x as f32,
                y: y as f32,
            }),
            absolute: None,
        });
        self
    }

    pub fn position_absolute(&mut self) -> &mut Self {
        self.get_action_queue_mut().push_back(NodeAction::Position {
            position: None,
            absolute: Some(true),
        });
        self
    }

    pub fn position_relative(&mut self) -> &mut Self {
        self.get_action_queue_mut().push_back(NodeAction::Position {
            position: None,
            absolute: Some(false),
        });
        self
    }

    pub fn push_action(&mut self, action: NodeAction) -> &mut Self {
        self.get_action_queue_mut().push_back(action);
        self
    }

    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.get_action_queue_mut()
            .push_back(NodeAction::Scale { x, y });
        self
    }

    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.get_action_queue_mut()
            .push_back(NodeAction::Size { width, height });
        self
    }

    pub fn bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) -> &mut Self {
        self.get_action_queue_mut().push_back(NodeAction::BGColor {
            color: Color { r, g, b, a },
        });
        self
    }

    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.get_action_queue_mut()
            .push_back(NodeAction::Layer { layer });
        self
    }

    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.get_action_queue_mut()
            .push_back(NodeAction::BorderRadius { radius });
        self
    }

    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.get_action_queue_mut().push_back(NodeAction::Wait {
            duration: Duration::from_millis(ms),
        });
        self
    }

    pub fn on_event<F>(&mut self, event_type: EventType, callback: F) -> &mut Self
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        self.get_listener_action_queue_mut()
            .push(NodeListenerAction::Add {
                event_type,
                callback: Box::new(callback),
            });
        self
    }

    pub fn off_event(&mut self, target: CallbackID) -> &mut Self {
        self.get_listener_action_queue_mut()
            .push(NodeListenerAction::Remove { target });
        self
    }

    pub fn on_active(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.get_node_state_mut()
                .on_active
                .insert(action.get_type(), *action);
        }
        self
    }

    pub fn on_hover(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.get_node_state_mut()
                .on_hover
                .insert(action.get_type(), *action);
        }
        self
    }
}
