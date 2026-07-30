use std::time::Instant;

use crate::error::PrismaError;
use crate::event::EventData;
use crate::event::context::CloseRequest;
use crate::event::context::EventContext;
use crate::event::event_manager::CallbackID;
use crate::event::{EventManager, EventType};
use crate::node::StyleView;
use crate::scene::NodeID;
use crate::scene::NodeView;
use crate::scene::node::NodeListenerAction;
use crate::scene::storage::NodeStorage;
use crate::scene::storage::StorageHandler;
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

    /// Creates a new node and returns a mutable [`NodeView`] to it.
    ///
    /// The returned view can be used to configure the node before or after the scene is running.
    pub fn new_node(&mut self) -> NodeView<'_> {
        let new_node = self.nodes.new_node();
        self.pending_nodes_handler.push_created(new_node.get_id());
        new_node
    }

    /// Returns `true` if the scene contains a node with the given ID.
    pub fn contains(&self, id: NodeID) -> bool {
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
        self.nodes.get_node_view(id)
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
    pub fn off_event(&mut self, target: CallbackID) {
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

        //context.process_node_actions(&mut self.event_manager)?;
        context.process_context_actions(&mut self.event_manager)?;

        self.pending_nodes_handler.extend_pending(&mut context);

        self.close_handler
            .handle_close(&mut context, &mut self.event_manager);

        Ok(())
    }

    fn process_nodes(&mut self) {
        let nodes_id = self.nodes.get_nodes_id();
        let StorageHandler {
            mut action_queue,
            mut listener_queue,
            mut state,
            mut style,
            mut transform,
            tree,
        } = self.nodes.storage();
        for id in nodes_id {
            /* apply listeners actions */
            let listener_actions = std::mem::take(listener_queue.get_unchecked_mut(id));
            for listener_action in listener_actions {
                match listener_action {
                    NodeListenerAction::Add {
                        event_type,
                        callback,
                    } => {
                        self.event_manager
                            .add_node_event_listener(id, event_type, callback);
                    }
                    NodeListenerAction::Remove { target } => {
                        self.event_manager.remove_event_listener(target);
                    }
                }
            }

            /* destruction queue */

            if state.get_unchecked(id).destruction_requested {
                let family = tree
                    .get_family(id)
                    .expect("Invariant violation: nodes contains an invalid ID");
                for familiar in family {
                    if !self.pending_nodes_handler.destroyed_contains(familiar) {
                        self.pending_nodes_handler.push_destroyed(familiar);
                    }
                }
            }

            /* apply node visual changes */

            let hovered = self.event_manager.is_node_hovered(id);
            let active = self.event_manager.is_node_active(id);

            for action in state.get_unchecked(id).og_style.clone().values() {
                action.apply_action(
                    style.get_unchecked_mut(id),
                    transform.get_unchecked_mut(id),
                    state.get_unchecked_mut(id),
                );
            }
            if hovered {
                if let Some(style_callback) = &mut state.get_unchecked_mut(id).on_hover {
                    let mut style = StyleView::new(action_queue.get_unchecked_mut(id), false);

                    style_callback(&mut style);
                }
            }
            if active {
                if let Some(style_callback) = &mut state.get_unchecked_mut(id).on_active {
                    let mut style = StyleView::new(action_queue.get_unchecked_mut(id), false);

                    style_callback(&mut style);
                }
            }

            let now = Instant::now();
            if let Some(until) = state.get_unchecked(id).waiting_until {
                if now < until {
                    return;
                } else {
                    state.get_unchecked_mut(id).waiting_until = None;
                }
            }

            while let Some(node_action) = action_queue.get_unchecked_mut(id).pop_front() {
                node_action.apply_action(
                    style.get_unchecked_mut(id),
                    transform.get_unchecked_mut(id),
                    state.get_unchecked_mut(id),
                );
                if node_action.is_persistent() {
                    state
                        .get_unchecked_mut(id)
                        .og_style
                        .insert(node_action.get_type(), node_action);
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
