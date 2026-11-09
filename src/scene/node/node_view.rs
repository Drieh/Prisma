use crate::{
    error::PrismaError,
    event::{EventCallback, EventCallbackID, EventContext, EventData},
    node::{
        EventListenerAction, NodeTreeAction, StyleActionOrigin,
        component::{NodeQueue, NodeVisual},
        style_view::StyleView,
    },
    scene::{
        NodeID,
        node::component::NodeState,
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

    /// Returns an immutable reference to the node's `NodeVisual` component
    pub fn get_node_visual(&self) -> &NodeVisual {
        self.storage.visual.get_unchecked(self.id)
    }

    /// Returns an immutable reference to the node's `NodeVisual` component
    pub(crate) fn get_node_visual_mut(&mut self) -> &mut NodeVisual {
        self.storage.visual.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_state(&mut self) -> &mut NodeState {
        self.storage.state.get_unchecked_mut(self.id)
    }

    pub(crate) fn get_node_queue(&mut self) -> &mut NodeQueue {
        self.storage.queue.get_unchecked_mut(self.id)
    }

    fn style(&mut self) -> StyleView<'_> {
        StyleView::new(self.get_node_queue(), StyleActionOrigin::Code)
    }

    /// Stores a custom [`Any`] value.
    ///
    /// The value can later be retrieved using [`NodeView::get_state`] or
    /// [`NodeView::get_state_mut`].
    pub fn set_state<T: Any>(&mut self, value: T) {
        self.get_node_state().set::<T>(value);
    }

    /// Returns a mutable reference to a custom [`Any`] value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist.
    pub fn get_state_mut<T: Any>(&mut self) -> Result<&mut T, PrismaError> {
        self.get_node_state().get_mut::<T>()
    }

    /// Returns an immutable reference to the custom [`Any`] value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist.
    pub fn get_state<T: Any>(&self) -> Result<&T, PrismaError> {
        self.storage.state.get_unchecked(self.id).get::<T>()
    }

    /// Returns `true` if the given state exists.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    pub fn has_state<T: Any>(&self) -> bool {
        self.storage.state.get_unchecked(self.id).contains::<T>()
    }

    /// Removes and returns the value associated with the given key.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the given key does not exist.
    pub fn remove_state<T: Any>(&mut self) -> Result<T, PrismaError> {
        self.get_node_state().remove::<T>()
    }

    /// Returns the node and all of its descendants recursively.
    pub fn get_family(&self, id: NodeID) -> Result<Vec<NodeID>, PrismaError> {
        self.storage.tree.get_family(id)
    }

    /// Schedules the node for destruction on the next frame.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    pub fn destroy(&mut self) {
        self.get_node_state().destruction_requested = true;
    }

    pub fn is_destruction_requested(&self) -> bool {
        self.storage
            .state
            .get_unchecked(self.id)
            .destruction_requested
    }

    /// Adds the given node as a child and returns a mutable reference to [`Self`].
    ///
    /// The child's parent is set to the current node.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if the child node does not exist.
    pub fn add_child(&mut self, child: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child) {
            return Err(PrismaError::NodeNotFound(child));
        }
        if self.id == child {
            return Err(PrismaError::InvalidTree(self.id, child));
        }
        if let Some(parent) = self.get_parent()
            && parent == child
        {
            return Err(PrismaError::InvalidTree(self.id, child));
        }

        self.get_node_queue()
            .push_back::<NodeTreeAction>(NodeTreeAction::AddChild { child });
        Ok(self)
    }

    /// Removes the given child and returns a mutable reference to [`Self`].
    ///
    /// Also the child's parent is removed.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if the child node does not exist.
    pub fn remove_child(&mut self, child: NodeID) -> Result<&mut Self, PrismaError> {
        if !self.storage.has_node(child) {
            return Err(PrismaError::NodeNotFound(child));
        }
        self.get_node_queue()
            .push_back(NodeTreeAction::RemoveChild { child });
        Ok(self)
    }

    /// Sets the node's parent to the given option [`NodeID`] value.
    ///
    /// # Behavior
    ///
    /// This operation is deferred.
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if the parent node does not exist.
    pub fn set_parent(&mut self, parent: Option<NodeID>) -> Result<&mut Self, PrismaError> {
        if let Some(parent) = parent {
            if !self.storage.has_node(parent) {
                return Err(PrismaError::NodeNotFound(parent));
            }
            self.get_node_queue().push_back(NodeTreeAction::SetParent {
                parent: Some(parent),
            });
        }
        Ok(self)
    }

    /// Returns the [`NodeID`] of the parent node, if any.
    pub fn get_parent(&self) -> Option<NodeID> {
        self.storage.tree.get_unchecked(self.id).get_parent()
    }

    /// Returns a list of the node's children.
    pub fn get_children(&self) -> Vec<NodeID> {
        self.storage.tree.get_unchecked(self.id).get_children()
    }

    /// Returns the size of the node's bounding box after applying its current scale.
    pub fn get_bounding_box(&self) -> Size {
        self.get_node_visual().get_bounding_box()
    }

    /// Returns the size of the node's box before applying its current scale.
    pub fn get_size(&self) -> Size {
        self.get_node_visual().get_size()
    }

    /// Returns the node's position relative to its parent.
    pub fn get_relative_position(&self) -> Position {
        self.get_node_visual().get_relative_position()
    }

    /// Returns the node's position relative to the scene.
    pub fn get_absolute_position(&mut self) -> Position {
        self.absolute_position(self.id).unwrap()
    }

    fn absolute_position(&mut self, id: NodeID) -> Result<Position, PrismaError> {
        if !self.storage.has_node(id) {
            return Err(PrismaError::NodeNotFound(id));
        }
        let self_transform = self.storage.visual.get_unchecked(id).clone();
        if !self_transform.is_position_absolute()
            && let tree = self.storage.tree.get_unchecked(id).clone()
            && let Some(parent) = tree.get_parent()
        {
            Ok(self.absolute_position(parent)? + self_transform.get_relative_position())
        } else {
            Ok(self_transform.get_relative_position())
        }
    }

    /// Sets the node's position to the given `x` and `y` immediately.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.get_node_visual_mut()
            .set_position(Position { x, y }, StyleActionOrigin::Code);

        self
    }

    /// Sets the node's position as an absolute position relative to the scene.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn position_absolute(&mut self) -> &mut Self {
        self.get_node_visual_mut()
            .set_position_absolute(StyleActionOrigin::Code);
        self
    }

    /// Sets the node's position relative to its parent or the scene if it has no parent.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn position_relative(&mut self) -> &mut Self {
        self.get_node_visual_mut()
            .set_position_relative(StyleActionOrigin::Code);
        self
    }

    /// Sets the node's scale.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.get_node_visual_mut()
            .set_scale(x, y, StyleActionOrigin::Code);
        self
    }

    /// Sets the node's size.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.get_node_visual_mut()
            .set_size(width, height, StyleActionOrigin::Code);
        self
    }

    /// Sets the background color of the node.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.get_node_visual_mut()
            .set_bg_color(color, StyleActionOrigin::Code);
        self
    }

    /// Sets the layer of the node.
    ///
    /// Higher layers are rendered on top of lower layers.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn layer(&mut self, layer: usize) -> &mut Self {
        self.get_node_visual_mut()
            .set_layer(layer, StyleActionOrigin::Code);
        self
    }

    /// Sets the border radius of the node.
    ///
    /// # Behavior
    ///
    /// This operation is immediate.
    pub fn border_radius(&mut self, radius: u32) -> &mut Self {
        self.get_node_visual_mut()
            .set_border_radius(radius, StyleActionOrigin::Code);
        self
    }

    /// Sets the actions applied while the node is active.
    pub fn on_active<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        self.get_node_state().style_on_active = Some(Box::new(callback));
        self
    }

    /// Sets the actions applied while the node is hovered.
    pub fn on_hover<F>(&mut self, callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        self.get_node_state().style_on_hover = Some(Box::new(callback));
        self
    }

    /// A handler to modify the node's visual appearance.
    ///
    /// # Examples
    ///
    /// ```
    /// scene.new_node().with_style(|style| {
    ///     style.size(100, 100).bg_color(Color::RED);
    /// });
    /// ```
    ///
    /// # Behavior
    ///
    /// All operations made with this function are deferred.
    ///
    /// Creates a 100 x 100 pixel node with a red background color.
    pub fn with_style<F>(&mut self, mut callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        callback(&mut self.style());
        self
    }

    /// Registers a callback for the given event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event to listen for.
    /// * `callback` - The function to invoke when the event is dispatched.
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
        self.get_node_queue().push_back(EventListenerAction::Add {
            event_type: T::TYPE,
            callback: EventCallback::new(real_callback),
        });
        self
    }

    /// Removes the callback identified by the given [`EventCallbackID`].
    pub fn off_event(&mut self, target: EventCallbackID) -> &mut Self {
        self.get_node_queue()
            .push_back(EventListenerAction::Remove { target });
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
