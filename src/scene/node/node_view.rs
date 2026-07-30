use crate::{
    error::PrismaError,
    event::{EventData, context::EventContext, event_manager::CallbackID},
    node::{ActionQueue, ListenerQueue, style_view::StyleView},
    scene::{
        NodeID,
        node::{
            NodeListenerAction,
            components::{NodeState, Style, Transform, Tree},
        },
        storage::{NodeStorage, StorageHandler},
    },
    util::{Color, Position},
};
use std::{
    any::Any,
    fmt::{Debug, Display},
};

/// Provides mutable access to a node.
///
/// All modifications to a node must be performed through this type.
pub struct NodeView<'a> {
    id: NodeID,
    storage: StorageHandler<'a>,
}
impl<'a> NodeView<'a> {
    pub(crate) fn new(id: NodeID, nodes: &'a mut NodeStorage) -> Result<Self, PrismaError> {
        if nodes.exists(id) {
            Ok(Self {
                id,
                storage: nodes.storage(),
            })
        } else {
            Err(PrismaError::NodeNotFound(id))
        }
    }
    // getters

    /// Returns the node's unique identifier.
    pub fn get_id(&self) -> NodeID {
        self.id
    }

    /// Returns an immutable reference to the node's `Tree` component
    pub fn get_tree(&self) -> &Tree {
        self.storage.tree.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `Style` component
    pub fn get_style(&self) -> &Style {
        self.storage.style.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `Transform` component
    pub fn get_transform(&self) -> &Transform {
        self.storage.transform.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `NodeState` component
    pub fn get_node_state(&self) -> &NodeState {
        self.storage.state.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `ActionQueue` component
    pub fn get_action_queue(&self) -> &ActionQueue {
        self.storage.action_queue.get_unchecked(self.id)
    }

    pub(crate) fn get_tree_mut(&mut self) -> &mut Tree {
        self.storage.tree.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_style_mut(&mut self) -> &mut Style {
        self.storage.style.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_transform_mut(&mut self) -> &mut Transform {
        self.storage.transform.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_state_mut(&mut self) -> &mut NodeState {
        self.storage.state.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_action_queue_mut(&mut self) -> &mut ActionQueue {
        self.storage.action_queue.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_listener_queue_mut(&mut self) -> &mut ListenerQueue {
        self.storage.listener_queue.get_unchecked_mut(self.id)
    }

    /// Stores a custom [`Any`] value associated with the given key.
    ///
    /// The value can later be retrieved using [`NodeView::get_state`] or
    /// [`NodeView::get_state_mut`].
    pub fn set_state<T: Any>(&mut self, key: impl Into<String>, value: T) {
        self.get_node_state_mut()
            .user_state
            .insert(key.into(), Box::new(value));
    }

    /// Returns a mutable reference to the custom [`Any`] value associated with the given key.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist or if the
    /// stored value cannot be downcast to `T`.
    pub fn get_state_mut<T: Any>(&mut self, key: &str) -> Result<&mut T, PrismaError> {
        self.get_node_state_mut().get_mut(key)
    }

    /// Returns an immutable reference to the custom [`Any`] value associated with the given key.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist or if the
    /// stored value cannot be downcast to `T`.
    pub fn get_state<T: Any>(&self, key: &str) -> Result<&T, PrismaError> {
        self.get_node_state().get(key)
    }

    /// Returns `true` if the given key has an associated value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    pub fn has_state(&self, key: &str) -> bool {
        self.get_node_state().has_state(key)
    }

    /// Removes and returns the value associated with the given key.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the given key does not exist.
    pub fn remove_state<T: Any>(&mut self, key: &str) -> Result<T, PrismaError> {
        self.get_node_state_mut().remove(key)
    }

    /// Schedules the node for destruction on the next frame.
    pub fn destroy(&mut self) {
        self.get_node_state_mut().destruction_requested = true;
    }

    /// Adds the given node as a child and returns a mutable reference to [`Self`].
    ///
    /// The child's parent is set to the current node.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if the child node does not exist.
    pub fn add_child(&mut self, child_id: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child_id) {
            return Err(PrismaError::NodeNotFound(child_id));
        }
        if self.id == child_id {
            return Err(PrismaError::InvalidTree(self.id, child_id));
        }
        if let Some(parent) = self.get_parent()
            && parent == child_id
        {
            return Err(PrismaError::InvalidTree(self.id, child_id));
        }
        let self_id = self.id;
        self.storage
            .tree
            .get_unchecked_mut(child_id)
            .set_parent(Some(self_id));
        self.get_tree_mut().add_child(child_id);
        Ok(self)
    }

    /// Removes the given child and returns a mutable reference to [`Self`].
    ///
    /// Also the child's parent is removed.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if the child node does not exist.
    pub fn remove_child(&mut self, child_id: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child_id) {
            return Err(PrismaError::NodeNotFound(child_id));
        }
        self.get_tree_mut().remove_child(child_id);
        self.storage
            .tree
            .get_unchecked_mut(child_id)
            .set_parent(None);

        Ok(self)
    }

    /// Returns the [`NodeID`] of the parent node, if any.
    pub fn get_parent(&self) -> Option<NodeID> {
        self.get_tree().get_parent()
    }

    /// Returns a list of the node's children.
    pub fn get_children(&self) -> Vec<NodeID> {
        self.get_tree().get_children()
    }

    /// Returns the size of the node's bounding box after applying its current scale.
    pub fn get_bounding_box_size(&self) -> (u32, u32) {
        (
            (self.get_style().size.0 as f32 * self.get_transform().scale.0)
                .abs()
                .round() as u32,
            (self.get_style().size.1 as f32 * self.get_transform().scale.1)
                .abs()
                .round() as u32,
        )
    }

    /// Returns the size of the node's box before applying its current scale.
    pub fn get_size(&self) -> (u32, u32) {
        self.get_style().size
    }

    /// Returns the node's position relative to its parent.
    pub fn get_relative_position(&self) -> Position {
        self.get_transform().position
    }

    /// Returns the node's position relative to the scene.
    pub fn get_absolute_position(&mut self) -> Position {
        self.absolute_position(self.id).unwrap()
    }

    fn absolute_position(&mut self, id: NodeID) -> Result<Position, PrismaError> {
        if !self.storage.has_node(id) {
            return Err(PrismaError::NodeNotFound(id));
        }
        let self_transform = *self.storage.transform.get_unchecked(id);
        if !self_transform.position_absolute
            && let tree = self.storage.tree.get_unchecked(id).clone()
            && let Some(parent) = tree.get_parent()
        {
            Ok(self.absolute_position(parent)? + self_transform.position)
        } else {
            Ok(self_transform.position)
        }
    }

    pub(crate) fn into_style(&mut self) -> StyleView<'_> {
        StyleView::new(self.get_node_action_queue_mut(), true)
    }

    /// Sets the node's position to the given `x` and `y`.
    ///
    /// Returns a mutable reference to [`Self`]
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.into_style().position(x, y);
        self
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_absolute(&mut self) -> &mut Self {
        self.into_style().position_absolute();
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_relative(&mut self) -> &mut Self {
        self.into_style().position_relative();
        self
    }

    /// Sets the node's scale.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.into_style().scale(x, y);
        self
    }

    /// Sets the node's size.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.into_style().size(width, height);
        self
    }

    /// Sets the node's background color.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.into_style().bg_color(color);
        self
    }

    /// Sets the node's render layer.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.into_style().layer(layer);
        self
    }

    /// Sets the node's border radius.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.into_style().border_radius(radius);
        self
    }

    /// Adds a timer to the node's [`NodeAction`] queue, delaying the execution of its actions.
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.into_style().wait(ms);
        self
    }

    /// Registers a callback for the given event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event to listen for.
    /// * `callback` - The function to invoke when the event is dispatched.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn on_event<T>(
        &mut self,
        mut callback: impl FnMut(&mut EventContext, T) + 'static,
    ) -> &mut Self
    where
        T: EventData,
    {
        let real_callback = move |ctx: &mut EventContext| {
            let event = ctx.expect_event::<T>().unwrap();
            callback(ctx, event);
        };
        self.get_node_listener_queue_mut()
            .push(NodeListenerAction::Add {
                event_type: T::TYPE,
                callback: Box::new(real_callback),
            });
        self
    }

    /// Removes the callback identified by the given [`CallbackID`].
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn off_event(&mut self, target: CallbackID) -> &mut Self {
        self.get_node_listener_queue_mut()
            .push(NodeListenerAction::Remove { target });
        self
    }

    /// Sets the actions applied while the node is active.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn on_active<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        self.get_node_state_mut().on_active = Some(Box::new(callback));
        self
    }

    /// Sets the actions applied while the node is hovered.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn on_hover<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        self.get_node_state_mut().on_hover = Some(Box::new(callback));
        self
    }
}
impl<'a> Display for NodeView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node view for node {}", self.id)
    }
}
impl<'a> Debug for NodeView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node view for node {}", self.id)
    }
}
