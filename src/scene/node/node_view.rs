use crate::{
    error::PrismaError,
    event::{EventCallback, EventCallbackID, EventData, context::EventContext},
    node::{
        ActionOrigin, NodeTreeAction,
        component::{NodeQueue, NodeVisual},
        node_action::EventListenerAction,
        style_view::StyleView,
    },
    scene::{
        NodeID,
        node::component::{NodeState, NodeTree},
        storage::{NodeStorage, StorageHandler},
    },
    util::{Color, Position, Size},
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
    origin: ActionOrigin,
    storage: StorageHandler<'a>,
}
impl<'a> NodeView<'a> {
    pub(crate) fn new(
        id: NodeID,
        nodes: &'a mut NodeStorage,
        origin: ActionOrigin,
    ) -> Result<Self, PrismaError> {
        if nodes.exists(id) {
            Ok(Self {
                id,
                origin,
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

    /// Returns an immutable reference to the node's `NodeTree` component
    pub(crate) fn get_tree(&self) -> &NodeTree {
        self.storage.tree.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `NodeVisual` component
    pub(crate) fn get_visual(&self) -> &NodeVisual {
        self.storage.visual.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `NodeState` component
    pub(crate) fn get_node_state(&self) -> &NodeState {
        self.storage.state.get_unchecked(self.id)
    }

    pub(crate) fn get_tree_mut(&mut self) -> &mut NodeTree {
        self.storage.tree.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_state_mut(&mut self) -> &mut NodeState {
        self.storage.state.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_queue_mut(&mut self) -> &mut NodeQueue {
        self.storage.queue.get_unchecked_mut(self.id)
    }

    fn style(&mut self) -> StyleView<'_> {
        let origin = self.origin;
        StyleView::new(self.get_node_queue_mut(), origin)
    }

    /// Stores a custom [`Any`] value.
    ///
    /// The value can later be retrieved using [`NodeView::get_state`] or
    /// [`NodeView::get_state_mut`].
    pub fn set_state<T: Any>(&mut self, value: T) {
        self.get_node_state_mut().set::<T>(value);
    }

    /// Returns a mutable reference to a custom [`Any`] value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist.
    pub fn get_state_mut<T: Any>(&mut self) -> Result<&mut T, PrismaError> {
        self.get_node_state_mut().get_mut::<T>()
    }

    /// Returns an immutable reference to the custom [`Any`] value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist.
    pub fn get_state<T: Any>(&self) -> Result<&T, PrismaError> {
        self.get_node_state().get::<T>()
    }

    /// Returns `true` if the given state exists.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    pub fn has_state<T: Any>(&self) -> bool {
        self.get_node_state().contains::<T>()
    }

    /// Removes and returns the value associated with the given key.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the given key does not exist.
    pub fn remove_state<T: Any>(&mut self) -> Result<T, PrismaError> {
        self.get_node_state_mut().remove::<T>()
    }

    pub fn get_family(&self, id: NodeID) -> Result<Vec<NodeID>, PrismaError> {
        let mut family: Vec<NodeID> = Vec::new();
        self.tree_get_family(id, &mut family)?;
        Ok(family)
    }

    fn tree_get_family(&self, id: NodeID, output: &mut Vec<NodeID>) -> Result<(), PrismaError> {
        output.push(id);
        for child_id in self.storage.tree.get(id)?.get_children() {
            self.tree_get_family(child_id, output)?;
        }
        Ok(())
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

        let origin = self.origin;

        self.get_node_queue_mut()
            .push_back::<NodeTreeAction>(NodeTreeAction::AddChild { child: child_id }, origin);

        self.storage
            .queue
            .get_unchecked_mut(child_id)
            .push_back::<NodeTreeAction>(
                NodeTreeAction::SetParent {
                    parent: Some(self.id),
                },
                origin,
            );

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

        let origin = self.origin;

        self.get_node_queue_mut()
            .push_back(NodeTreeAction::RemoveChild { child: child_id }, origin);

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
    pub fn get_bounding_box_size(&self) -> Size {
        self.get_visual().get_bounding_box()
    }

    /// Returns the size of the node's box before applying its current scale.
    pub fn get_size(&self) -> Size {
        self.get_visual().get_size()
    }

    /// Returns the node's position relative to its parent.
    pub fn get_relative_position(&self) -> Position {
        self.get_visual().get_relative_position()
    }

    /// Returns the node's position relative to the scene.
    pub fn get_absolute_position(&mut self) -> Position {
        self.absolute_position(self.id).unwrap()
    }

    fn absolute_position(&mut self, id: NodeID) -> Result<Position, PrismaError> {
        if !self.storage.has_node(id) {
            return Err(PrismaError::NodeNotFound(id));
        }
        let self_transform = *self.storage.visual.get_unchecked(id);
        if !self_transform.is_position_absolute()
            && let tree = self.storage.tree.get_unchecked(id).clone()
            && let Some(parent) = tree.get_parent()
        {
            Ok(self.absolute_position(parent)? + self_transform.get_relative_position())
        } else {
            Ok(self_transform.get_relative_position())
        }
    }

    /// Sets the node's position to the given `x` and `y`.
    ///
    /// Returns a mutable reference to [`Self`]
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.style().position(x, y);
        self
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_absolute(&mut self) -> &mut Self {
        self.style().position_absolute();
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn position_relative(&mut self) -> &mut Self {
        self.style().position_relative();
        self
    }

    /// Sets the node's scale.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.style().scale(x, y);
        self
    }

    /// Sets the node's size.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.style().size(width, height);
        self
    }

    /// Sets the node's background color.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.style().bg_color(color);
        self
    }

    /// Sets the node's render layer.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.style().layer(layer);
        self
    }

    /// Sets the node's border radius.
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.style().border_radius(radius);
        self
    }

    /// Adds a timer to the node's [`NodeAction`] queue, delaying the execution of its actions.
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.style().wait(ms);
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
        self.get_node_queue_mut().push_back(
            EventListenerAction::Add {
                event_type: T::TYPE,
                callback: EventCallback::new(real_callback),
            },
            ActionOrigin::Code,
        );
        self
    }

    /// Removes the callback identified by the given [`CallbackID`].
    ///
    /// Returns a mutable reference to [`Self`].
    pub fn off_event(&mut self, target: EventCallbackID) -> &mut Self {
        self.get_node_queue_mut()
            .push_back(EventListenerAction::Remove { target }, ActionOrigin::Code);
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
