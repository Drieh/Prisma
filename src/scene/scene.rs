use std::time::Instant;

use crate::error::PrismaError;
use crate::event::EventCallbackID;
use crate::event::EventData;
use crate::event::EventManager;
use crate::event::context::CloseRequest;
use crate::event::context::EventContext;
use crate::node::Action;
use crate::node::ActionOrigin;
use crate::node::EventListenerAction;
use crate::node::NodeTreeAction;
use crate::node::StyleAction;
use crate::node::StyleView;
use crate::scene::NodeID;
use crate::scene::NodeView;
use crate::scene::storage::{NodeStorage, StorageHandler};
use crate::util::Color;

struct CloseHandler {
    close_request: Option<CloseRequest>,
    quitting: bool,
}
impl CloseHandler {
    fn new() -> Self {
        Self {
            close_request: None,
            quitting: false,
        }
    }

    fn handle_close(&mut self, context: &mut EventContext, event_manager: &mut EventManager) {
        if let Some(close_request) = context.close_request {
            self.close_request = Some(close_request);
            event_manager.send_close_request(close_request);
        }
        if let Some(close_request) = self.close_request
            && close_request.requested_at.elapsed() >= close_request.duration
        {
            self.quitting = true;
            event_manager.send_quit();
            for id in context.get_nodes() {
                context.destroy(id);
            }
        }
        if context.is_cancel_close_requested() {
            self.close_request = None;
            event_manager.cancel_close();
        }
    }
}

struct PendingNodesHandler {
    created: Vec<NodeID>,
    destroyed: Vec<NodeID>,
}
impl PendingNodesHandler {
    pub fn new() -> Self {
        Self {
            created: Vec::new(),
            destroyed: Vec::new(),
        }
    }

    pub fn extend_pending(&mut self, context: &mut EventContext) {
        self.created.extend(context.take_created_nodes());
        self.destroyed.extend(context.take_destroyed_nodes());
    }

    pub fn take_created(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.created)
    }

    pub fn take_destroyed(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.destroyed)
    }

    pub fn push_created(&mut self, id: NodeID) {
        self.created.push(id);
    }

    pub fn push_destroyed(&mut self, id: NodeID) {
        self.destroyed.push(id);
    }

    pub fn destroyed_contains(&self, id: NodeID) -> bool {
        self.destroyed.contains(&id)
    }
}

/// Represents a UI scene displayed inside an window.
///
/// A scene owns all nodes and processes events within its window.
pub struct Scene {
    pub color: Color,
    nodes: NodeStorage,
    event_manager: EventManager,
    pending_nodes_handler: PendingNodesHandler,
    close_handler: CloseHandler,
}
impl Scene {
    pub fn new() -> Self {
        Self {
            color: Color::rgb(255, 255, 255),
            nodes: NodeStorage::new(),
            event_manager: EventManager::new(),
            pending_nodes_handler: PendingNodesHandler::new(),
            close_handler: CloseHandler::new(),
        }
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        self.nodes.storage()
    }

    /// Creates a new node and returns a mutable [`NodeView`] to it.
    ///
    /// The returned view can be used to configure the node before or after Prisma is running.
    pub fn new_node(&mut self) -> NodeView<'_> {
        let new_node = self.nodes.new_node(ActionOrigin::Code);
        self.pending_nodes_handler.push_created(new_node.get_id());
        new_node
    }

    /// Returns `true` if the scene contains a node with the given ID.
    pub fn contains(&mut self, id: NodeID) -> bool {
        self.nodes.exists(id)
    }

    /// Returns a list of all available nodes.
    pub fn get_nodes_id(&self) -> Vec<NodeID> {
        self.nodes.get_nodes_id()
    }

    /// Returns a [`NodeView`] of the given [`NodeID`].
    ///
    /// # Errors
    ///
    /// Returns [`PrismaError::NodeNotFound`] if there is no node for the given [`NodeID`].
    pub fn get_node(&mut self, id: NodeID) -> Result<NodeView<'_>, PrismaError> {
        self.nodes.get_node_view(id, ActionOrigin::Code)
    }

    /// Registers a callback for the given event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The event to listen for.
    /// * `callback` - The function to invoke when the event is dispatched.
    pub fn on_event<T>(&mut self, mut callback: impl FnMut(&mut EventContext, T) + 'static)
    where
        T: EventData,
    {
        let real_callback = move |ctx: &mut EventContext| {
            let event = ctx.expect_event::<T>().unwrap();
            callback(ctx, event);
        };
        self.event_manager
            .add_scene_event_listener(T::TYPE, real_callback);
    }

    /// Removes the callback identified by the given [`CallbackID`].
    pub fn off_event(&mut self, target: EventCallbackID) {
        self.event_manager.remove_event_listener(target);
    }

    /// Sets the background color of the scene.
    pub fn bg_color(&mut self, color: Color) {
        self.color = Color { a: 255, ..color };
    }

    pub fn get_bg_color(&self) -> Color {
        self.color
    }

    /// Returns `true` if window is about to close.
    pub fn is_quitting(&self) -> bool {
        self.close_handler.quitting
    }

    pub(crate) fn manage_lifecycle_events(&mut self) -> Result<(), PrismaError> {
        self.process_nodes();

        let mut context = EventContext::new(&mut self.nodes);
        let pending_created_nodes = self.pending_nodes_handler.take_created();
        let pending_destroyed_nodes = self.pending_nodes_handler.take_destroyed();

        self.event_manager
            .manage_lifecycle_events(&pending_created_nodes, &pending_destroyed_nodes);

        self.event_manager.dispatch(&mut context);

        context.process_context_actions(&mut self.event_manager)?;

        self.pending_nodes_handler.extend_pending(&mut context);

        self.close_handler
            .handle_close(&mut context, &mut self.event_manager);

        for id in pending_destroyed_nodes {
            self.event_manager.clear_node_listeners(id);
            self.nodes.destroy_node(id)?;
        }
        Ok(())
    }

    pub(crate) fn manage_sdl_events(
        &mut self,
        sdl_event: &sdl3::event::Event,
    ) -> Result<(), PrismaError> {
        let mut context = EventContext::new(&mut self.nodes);

        self.event_manager.manage_sdl_event(sdl_event);

        self.event_manager.dispatch(&mut context);

        context.process_context_actions(&mut self.event_manager)?;

        self.pending_nodes_handler.extend_pending(&mut context);

        self.close_handler
            .handle_close(&mut context, &mut self.event_manager);

        Ok(())
    }

    fn process_nodes(&mut self) {
        let nodes_id = self.nodes.get_nodes_id();

        for id in nodes_id {
            self.process_node_event_listener_actions(id);
            self.process_node_destruction_queue(id);
            self.process_node_style_actions(id);
        }
    }

    fn process_node_event_listener_actions(&mut self, id: NodeID) {
        let StorageHandler { mut queue, .. } = self.nodes.storage();
        while let Some(listener_action) = queue
            .get_unchecked_mut(id)
            .pop_front::<EventListenerAction>()
        {
            match listener_action {
                EventListenerAction::Add {
                    event_type,
                    callback,
                } => {
                    self.event_manager
                        .add_node_event_listener(id, event_type, callback.callback);
                }
                EventListenerAction::Remove { target } => {
                    self.event_manager.remove_event_listener(target);
                }
            }
        }
    }

    fn process_node_style_actions(&mut self, id: NodeID) {
        let StorageHandler {
            mut state,
            mut queue,
            mut visual,
            ..
        } = self.nodes.storage();
        let node_state = state.get_unchecked_mut(id);
        let node_visual = visual.get_unchecked_mut(id);
        let node_queue = queue.get_unchecked_mut(id);

        let is_hovered = self.event_manager.is_node_hovered(id);
        let is_active = self.event_manager.is_node_active(id);

        if is_hovered && let Some(style_callback) = &mut node_state.on_hover {
            let mut style = StyleView::new(node_queue, ActionOrigin::Visual);
            style_callback(&mut style);
        }

        if is_active && let Some(style_callback) = &mut node_state.on_active {
            let mut style = StyleView::new(node_queue, ActionOrigin::Visual);
            style_callback(&mut style);
        }

        if let Some(timer) = node_visual.get_timer()
            && let Some(start) = node_visual.get_timer_start()
        {
            if Instant::now() >= start + timer {
                return;
            }
            node_visual.clear_timer();
        }

        node_visual.set_normal();
        while let Some(Action::Style {
            action: style_action,
            origin,
        }) = node_queue.pop_front_action::<StyleAction>()
        {
            match origin {
                ActionOrigin::Visual => {
                    if is_hovered {
                        node_visual.set_hover();
                    }
                    if is_active {
                        node_visual.set_active();
                    }
                }
                _ => {}
            }

            match style_action {
                StyleAction::BGColor { color } => {
                    node_visual.set_color(color);
                }
                StyleAction::Layer { layer } => {
                    node_visual.set_layer(layer);
                }
                StyleAction::BorderRadius { radius } => {
                    node_visual.set_border_radius(radius);
                }
                StyleAction::Scale { x, y } => {
                    node_visual.set_scale(x, y);
                }
                StyleAction::Position { position, absolute } => {
                    if let Some(value) = position {
                        node_visual.set_position(value);
                    }
                    if let Some(value) = absolute {
                        if value {
                            node_visual.set_position_absolute();
                        } else {
                            node_visual.set_position_relative();
                        }
                    }
                }
                StyleAction::Size { width, height } => {
                    node_visual.set_size(width, height);
                }
                StyleAction::Wait { ms } => {
                    node_visual.set_timer(ms);
                    break;
                }
            }
        }
    }

    fn process_node_destruction_queue(&mut self, id: NodeID) {
        let node = self.get_node(id).unwrap();
        if node.get_node_state().destruction_requested {
            let family = node
                .get_family(id)
                .expect("Invariant violation: nodes contains an invalid ID");
            for familiar in family {
                if !self.pending_nodes_handler.destroyed_contains(familiar) {
                    self.pending_nodes_handler.push_destroyed(familiar);
                }
            }
        }
    }

    fn process_node_tree_actions(&mut self, id: NodeID) {
        let StorageHandler {
            mut tree,
            mut queue,
            ..
        } = self.nodes.storage();
        let node_queue = queue.get_unchecked_mut(id);
        let node_tree = tree.get_unchecked_mut(id);
        while let Some(action) = node_queue.pop_front::<NodeTreeAction>() {
            match action {
                NodeTreeAction::AddChild { child } => {
                    node_tree.add_child(child);
                }
                NodeTreeAction::RemoveChild { child } => {
                    node_tree.remove_child(child);
                }
                NodeTreeAction::SetParent { parent } => {
                    node_tree.set_parent(parent);
                }
            }
        }
    }
}
impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
