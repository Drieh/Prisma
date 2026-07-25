use std::time::Instant;

use crate::app::PrismaError;
use crate::event::context::CloseRequest;
use crate::event::context::EventContext;
use crate::event::managers::event_manager::CallbackID;
use crate::event::{EventManager, EventType};
use crate::scene::nodes::NodeAction;
use crate::scene::nodes::NodeActionType;
use crate::scene::nodes::NodeID;
use crate::scene::nodes::NodeListenerAction;
use crate::scene::nodes::NodeView;
use crate::scene::storage::NodeStorage;
use sdl3::pixels::Color;

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

    pub fn created_contains(&self, id: NodeID) -> bool {
        self.created.contains(&id)
    }

    pub fn destroyed_contains(&self, id: NodeID) -> bool {
        self.destroyed.contains(&id)
    }
}

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
            color: Color::RGB(255, 255, 255),
            nodes: NodeStorage::new(),
            event_manager: EventManager::new(),
            pending_nodes_handler: PendingNodesHandler::new(),
            close_handler: CloseHandler::new(),
        }
    }

    pub fn new_node(&mut self) -> NodeView<'_> {
        let new_node = self.nodes.new_node();
        self.pending_nodes_handler.push_created(new_node.get_id());
        new_node
    }

    pub fn node_exists(&self, id: NodeID) -> bool {
        self.nodes.exists(id)
    }

    pub fn get_nodes_id(&self) -> Vec<NodeID> {
        self.nodes.get_nodes_id()
    }

    pub fn get_node(&mut self, id: NodeID) -> Result<NodeView<'_>, PrismaError> {
        self.nodes.get_node(id)
    }

    pub fn on_event<F>(&mut self, event_type: EventType, callback: F)
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        self.event_manager
            .add_scene_event_listener(event_type, callback);
    }
    pub fn off_event(&mut self, target: CallbackID) {
        self.event_manager.remove_event_listener(target);
    }

    pub fn bg_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color { r, g, b, a: 255 };
    }

    pub fn is_quitting(&self) -> bool {
        self.close_handler.quitting
    }

    pub(crate) fn manage_lifecycle_events(&mut self) -> Result<(), PrismaError> {
        self.process_nodes();

        let mut context = EventContext::new(&mut self.nodes);
        let pending_created_nodes = self.pending_nodes_handler.take_created();
        let pending_destroyed_nodes = self.pending_nodes_handler.take_destroyed();

        self.event_manager.manage_lifecycle_events(
            &mut context,
            &pending_destroyed_nodes,
            &pending_created_nodes,
        );

        self.event_manager.dispatch(&mut context);

        context.process_context_actions(&mut self.event_manager)?;

        // for next frame
        self.pending_nodes_handler.extend_pending(&mut context);

        // usan context
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
        for id in self.nodes.get_nodes_id() {
            let listener_actions = self.nodes.take_listener_queue(id);
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
            if self
                .nodes
                .storage()
                .state
                .get_unchecked(id)
                .destruction_requested
            {
                let family = self
                    .nodes
                    .storage()
                    .tree
                    .get_family(id)
                    .expect("Invariant violation: nodes contains an invalid ID");
                for familiar in family {
                    if !self.pending_nodes_handler.destroyed_contains(familiar) {
                        self.pending_nodes_handler.push_destroyed(familiar);
                    }
                }
            }
            let is_hovered = self.event_manager.is_node_hovered(id);
            let is_active = self.event_manager.is_node_active(id);

            for action in self
                .nodes
                .storage()
                .state
                .get_unchecked_mut(id)
                .og_state
                .clone()
                .values()
            {
                if action.get_type() != NodeActionType::Wait {
                    self.execute_node_action(id, *action);
                }
            }
            if is_hovered {
                for action in self
                    .nodes
                    .storage()
                    .state
                    .get_unchecked_mut(id)
                    .on_hover
                    .clone()
                    .values()
                {
                    if action.get_type() != NodeActionType::Wait {
                        self.execute_node_action(id, *action);
                    }
                }
            }
            if is_active {
                for action in self
                    .nodes
                    .storage()
                    .state
                    .get_unchecked_mut(id)
                    .on_active
                    .clone()
                    .values()
                {
                    if action.get_type() != NodeActionType::Wait {
                        self.execute_node_action(id, *action);
                    }
                }
            }
            if let Some(until) = self
                .nodes
                .storage()
                .state
                .get_unchecked_mut(id)
                .waiting_until
            {
                if Instant::now() < until {
                    return;
                }
                self.nodes
                    .storage()
                    .state
                    .get_unchecked_mut(id)
                    .waiting_until = None;
            }
            if let Some(action) = self
                .nodes
                .storage()
                .action_queue
                .get_unchecked_mut(id)
                .pop_front()
            {
                self.nodes
                    .storage()
                    .state
                    .get_unchecked_mut(id)
                    .og_state
                    .insert(action.get_type(), action);
                if action.get_type() == NodeActionType::Wait {
                    self.execute_node_action(id, action);
                }
            }
        }
    }

    fn execute_node_action(&mut self, id: NodeID, action: NodeAction) {
        match action {
            NodeAction::Position { position, absolute } => {
                if let Some(value) = position {
                    self.nodes
                        .storage()
                        .transform
                        .get_unchecked_mut(id)
                        .position = value;
                }
                if let Some(value) = absolute {
                    self.nodes
                        .storage()
                        .transform
                        .get_unchecked_mut(id)
                        .position_absolute = value;
                }
            }

            NodeAction::Size { width, height } => {
                self.nodes.storage().style.get_unchecked_mut(id).size = (width, height);
            }
            NodeAction::Scale { x, y } => {
                self.nodes.storage().transform.get_unchecked_mut(id).scale = (x, y);
            }
            NodeAction::BGColor { color } => {
                self.nodes.storage().style.get_unchecked_mut(id).color = color;
            }
            NodeAction::Layer { layer } => {
                self.nodes.storage().transform.get_unchecked_mut(id).layer = Some(layer);
            }
            NodeAction::BorderRadius { radius } => {
                self.nodes
                    .storage()
                    .style
                    .get_unchecked_mut(id)
                    .border_radius = radius;
            }
            NodeAction::Wait { duration } => {
                self.nodes
                    .storage()
                    .state
                    .get_unchecked_mut(id)
                    .waiting_until = Some(Instant::now() + duration);
            }
        }
    }
}
