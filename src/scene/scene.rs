use crate::app::PrismaError;
use crate::event::context::ContextAction;
use crate::event::context::EventContext;
use crate::event::managers::event_manager::CloseRequest;
use crate::event::managers::event_manager::EventListenerID;
use crate::event::{EventManager, EventType, SceneCallback};
use crate::scene::Node;
use crate::scene::NodeID;
use crate::scene::node::EventListenerAction;
use sdl3::pixels::Color;

use std::collections::HashMap;

pub struct Scene {
    pub color: Color,

    nodes: HashMap<NodeID, Node>,

    pending_created_nodes: Vec<NodeID>,

    event_manager: EventManager,
    close_request: Option<CloseRequest>,
    //cancel_close_requested: bool,
    quitting: bool,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            color: Color::RGB(255, 255, 255),
            nodes: HashMap::new(),
            pending_created_nodes: Vec::new(),

            event_manager: EventManager::new(),

            close_request: None,
            quitting: false,
        }
    }

    pub fn new_node(&mut self) -> &mut Node {
        let node = Node::new();
        let id = node.id();

        self.pending_created_nodes.push(id);
        self.nodes.insert(id, node);

        self.node(id).unwrap()
    }

    pub fn add_child(&mut self, parent_id: NodeID, child_id: NodeID) -> Result<(), PrismaError> {
        let parent = self.node(parent_id)?;
        parent.add_child(child_id);
        let parent_transform = parent.get_transform().clone();

        let child = self.node(child_id)?;
        child.set_parent(Some(parent_id));

        let child_transform = child.get_transform().clone();

        child.get_transform_as_mut().position =
            child_transform.position + parent_transform.position;

        if child_transform.layer == None {
            child.get_transform_as_mut().layer = parent_transform.layer
        }
        Ok(())
    }

    pub fn remove_child(&mut self, parent_id: NodeID, child_id: NodeID) -> Result<(), PrismaError> {
        // This order ensures an error is returned if the child ID doesn't exist before removing it to the parent.
        self.node(child_id)?.set_parent(None);
        self.node(parent_id)?.remove_child(child_id);

        Ok(())
    }

    pub fn node(&mut self, id: NodeID) -> Result<&mut Node, PrismaError> {
        self.nodes.get_mut(&id).ok_or(PrismaError::NodeNotFound(id))
    }
    pub fn get_node(&self, id: NodeID) -> Result<&Node, PrismaError> {
        self.nodes.get(&id).ok_or(PrismaError::NodeNotFound(id))
    }

    pub(crate) fn nodes(&self) -> &HashMap<NodeID, Node> {
        &self.nodes
    }

    pub fn on(&mut self, event_type: EventType, callback: SceneCallback) {
        self.event_manager
            .add_scene_event_listener(event_type, callback);
    }
    pub fn off(&mut self, target: EventListenerID) {
        self.event_manager.remove_event_listener(target);
    }

    pub fn bg_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color { r, g, b, a: 255 };
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting
    }
    fn manage_close(&mut self) {
        if let Some(close_request) = &self.close_request {
            self.event_manager.send_close_request(*close_request);

            if close_request.requested_at.elapsed() >= close_request.duration {
                self.close_request = None;
                self.quitting = true;
                self.event_manager.send_quit();
            }
        }
    }
    fn manage_cancel_close(&mut self, context: &mut EventContext) {
        if let Some(request) = context.close_request {
            self.close_request = Some(request);
        }
        if context.is_cancel_close_requested() {
            self.close_request = None;
            self.event_manager.cancel_close();
        }
    }
    pub(crate) fn manage_lifecycle(&mut self) -> Result<(), PrismaError> {
        let mut context = EventContext::new();

        self.process_node_listeners();

        self.manage_close();

        let pending_created_nodes = std::mem::take(&mut self.pending_created_nodes);

        let destruction_queue = self.build_node_destruction_queue();
        self.event_manager
            .poll_lifecycle_events(&destruction_queue, &pending_created_nodes);

        self.event_manager.dispatch(&mut self.nodes, &mut context);

        self.process_context_actions(&mut context)?;
        self.process_node_actions();
        self.manage_cancel_close(&mut context);

        for node_id in destruction_queue {
            self.destroy_node(node_id)?;
        }

        Ok(())
    }

    fn process_node_listeners(&mut self) {
        for node in self.nodes.values_mut() {
            for action in node.take_event_listener_actions() {
                match action {
                    EventListenerAction::Add {
                        event_type,
                        callback,
                    } => {
                        self.event_manager
                            .add_node_event_listener(node.id(), event_type, callback);
                    }
                    EventListenerAction::Remove { target } => {
                        self.event_manager.remove_event_listener(target);
                    }
                }
            }
        }
    }

    fn process_node_actions(&mut self) {
        for node in self.nodes.values_mut() {
            let id = node.id();
            node.process_action_queue(
                self.event_manager.is_node_hovered(id),
                self.event_manager.is_node_active(id),
            );
        }
    }

    pub(crate) fn manage_sdl_event(
        &mut self,
        sdl_event: &sdl3::event::Event,
    ) -> Result<(), PrismaError> {
        let mut context = EventContext::new();

        self.event_manager.manage_user_event(sdl_event);
        self.event_manager.dispatch(&mut self.nodes, &mut context);

        self.process_context_actions(&mut context)?;

        self.manage_cancel_close(&mut context);
        Ok(())
    }

    fn process_context_actions(&mut self, context: &mut EventContext) -> Result<(), PrismaError> {
        let context_actions = context.take_actions();

        for node in context.take_nodes() {
            self.pending_created_nodes.push(node.id());
            self.nodes.insert(node.id(), node);
        }
        for action in context_actions {
            match action {
                ContextAction::Destruction { target } => {
                    self.node(target)?.destroy();
                }
                ContextAction::AddChild { parent, child } => {
                    self.node(parent)?.add_child(child);
                    self.node(child)?.set_parent(Some(parent));
                }
                ContextAction::RemoveChild { parent, child } => {
                    self.node(parent)?.remove_child(child);
                }

                ContextAction::AddSceneEventListener {
                    event_type,
                    callback,
                } => {
                    self.on(event_type, callback);
                }
                ContextAction::RemoveSceneEventListener { target } => {
                    self.off(target);
                }
            }
        }
        Ok(())
    }

    fn build_node_destruction_queue(&mut self) -> Vec<NodeID> {
        let mut destruction_queue: Vec<NodeID> = Vec::new();
        for node in self.nodes.values_mut() {
            if node.is_destruction_requested() {
                destruction_queue.push(node.id());
                if node.get_children().len() > 0 {
                    for child_id in node.get_children() {
                        destruction_queue.push(child_id);
                    }
                }
            }
        }
        destruction_queue
    }

    fn destroy_node(&mut self, node_id: NodeID) -> Result<(), PrismaError> {
        let node = self.node(node_id)?;

        if node.get_children().len() > 0 {
            for child_id in node.get_children() {
                self.destroy_node(child_id)?;
            }
        }
        self.nodes.remove(&node_id);
        Ok(())
    }
}
