use crate::{
    error::PrismaError,
    event::{EventCallback, EventCallbackID, EventContext, EventData},
    node::{
        EventListenerAction, NodeTreeAction,
        component::{NodeQueue, NodeVisual, StyleVisualState},
        style_view::StyleView,
    },
    scene::{
        NodeID,
        node::component::NodeState,
        storage::{NodeStorage, StorageHandler},
    },
    util::Position,
};
use std::{
    any::Any,
    fmt::{Debug, Display},
};

/// Provides mutable access to a node.
///
/// All modifications to a node must be performed through this type.
pub struct Node<'a> {
    id: NodeID,
    storage: StorageHandler<'a>,
}
impl<'a> Node<'a> {
    pub(crate) fn new(id: NodeID, nodes: &'a mut NodeStorage) -> Result<Self, PrismaError> {
        if nodes.contains(id) {
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
    pub fn get_visual(&self) -> &NodeVisual {
        self.storage.visual.get_unchecked(self.id)
    }

    pub fn get_visual_mut(&mut self) -> &mut NodeVisual {
        self.storage.visual.get_unchecked_mut(self.id)
    }

    pub(crate) fn state(&mut self) -> &mut NodeState {
        self.storage.state.get_unchecked_mut(self.id)
    }

    pub(crate) fn queue(&mut self) -> &mut NodeQueue {
        self.storage.queue.get_unchecked_mut(self.id)
    }

    /// Stores a custom [`Any`] value.
    ///
    /// The value can later be retrieved using [`NodeView::get_state`] or
    /// [`NodeView::get_state_mut`].
    pub fn set_state<T: Any>(&mut self, value: T) {
        self.state().set::<T>(value);
    }

    /// Returns a mutable reference to a custom [`Any`] value.
    ///
    /// Values can be stored using [`NodeView::set_state`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeStateNotFound`] if the key does not exist.
    pub fn get_state_mut<T: Any>(&mut self) -> Result<&mut T, PrismaError> {
        self.state().get_mut::<T>()
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

    pub fn try_state<T: Any>(&self) -> Option<&T> {
        self.get_state().ok()
    }

    pub fn try_state_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.get_state_mut().ok()
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
        self.state().remove::<T>()
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
        self.state().destruction_requested = true;
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
        if !self.storage.contains(child) {
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

        self.queue()
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
        if !self.storage.contains(child) {
            return Err(PrismaError::NodeNotFound(child));
        }
        self.queue()
            .push_back(NodeTreeAction::RemoveChild { child });
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

    /// Returns the node's position relative to the scene.
    pub fn get_scene_position(&mut self) -> Position {
        self.scene_position(self.id).unwrap()
    }

    fn scene_position(&mut self, id: NodeID) -> Result<Position, PrismaError> {
        if !self.storage.contains(id) {
            return Err(PrismaError::NodeNotFound(id));
        }
        let node_visual = self.storage.visual.get_unchecked(id).clone();
        let x = node_visual.get_transform().get_x();
        let y = node_visual.get_transform().get_y();

        if let tree = self.storage.tree.get_unchecked(id).clone()
            && let Some(parent) = tree.get_parent()
        {
            Ok(self.scene_position(parent)? + Position { x, y })
        } else {
            Ok(Position { x, y })
        }
    }

    /// Sets the actions applied while the node is active.
    pub fn active<F>(&mut self, mut callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        callback(&mut StyleView::new(self.queue(), StyleVisualState::Active));
        self.state().style_on_active = Some(Box::new(callback));
        self
    }

    /// Sets the actions applied while the node is hovered.
    pub fn hover<F>(&mut self, mut callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        callback(&mut StyleView::new(self.queue(), StyleVisualState::Hover));
        self.state().style_on_hover = Some(Box::new(callback));
        self
    }

    /// A handler to modify the node's visual appearance.
    ///
    /// # Examples
    ///
    /// ```
    /// scene.new_node().style(|style| {
    ///     style.background(|bg| {
    ///         bg.color(Color::RED);
    ///     })
    /// });
    /// ```
    ///
    /// # Behavior
    ///
    /// All operations made with this function are deferred.
    ///
    /// Creates a 100 x 100 pixel node with a red background color.
    pub fn style<F>(&mut self, mut callback: F) -> &mut Self
    where
        F: FnMut(&mut StyleView<'_>) + 'static,
    {
        callback(&mut StyleView::new(self.queue(), StyleVisualState::Normal));
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
        self.queue().push_back(EventListenerAction::Add {
            event_type: T::TYPE,
            callback: EventCallback::new(real_callback),
        });
        self
    }

    /// Removes the callback identified by the given [`EventCallbackID`].
    pub fn off_event(&mut self, target: EventCallbackID) -> &mut Self {
        self.queue()
            .push_back(EventListenerAction::Remove { target });
        self
    }
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node view for node {}", self.id)
    }
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node view for node {}", self.id)
    }
}
