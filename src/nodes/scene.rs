use crate::app::PrismaError;
use crate::common::Position;
use crate::event::context::ContextAction;
use crate::event::context::EventContext;
use crate::event::context::NodeBuilder;
use crate::event::managers::event_manager::CloseRequest;
use crate::event::{EventManager, EventType, NodeCallback, SceneCallback};
use crate::nodes::Node;
use crate::nodes::NodeID;
use sdl3::pixels::Color;
use std::collections::HashMap;

pub struct Scene {
    pub color: Color,

    nodes: HashMap<NodeID, Node>,

    pending_created_nodes: Vec<NodeID>,
    hovered_node: Option<NodeID>,
    active_node: Option<NodeID>,

    event_manager: EventManager,
    context_action_queue: Vec<ContextAction>,
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
            hovered_node: None,
            active_node: None,

            event_manager: EventManager::new(),
            context_action_queue: Vec::new(),
            close_request: None,
            quitting: false,
        }
    }

    pub fn new_node(&mut self) -> NodeID {
        let node = Node::new();
        let node_id = node.get_id();
        self.nodes.insert(node_id, node);
        self.pending_created_nodes.push(node_id);
        node_id
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

    pub fn node(&mut self, id: NodeID) -> Result<&mut Node, PrismaError> {
        self.nodes.get_mut(&id).ok_or(PrismaError::NodeNotFound(id))
    }
    pub fn get_node(&self, id: NodeID) -> Result<&Node, PrismaError> {
        self.nodes.get(&id).ok_or(PrismaError::NodeNotFound(id))
    }

    pub(crate) fn nodes(&self) -> &HashMap<NodeID, Node> {
        &self.nodes
    }

    pub fn on_scene(&mut self, event_type: EventType, callback: SceneCallback) {
        self.event_manager
            .add_scene_event_listener(event_type, callback);
    }

    pub fn on_node(&mut self, node: NodeID, event_type: EventType, callback: NodeCallback) {
        self.event_manager
            .add_node_event_listener(node, event_type, callback);
    }

    pub fn bg_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color { r, g, b, a: 255 };
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting
    }

    pub(crate) fn manage_lifecycle(&mut self) {
        let mut event_context = EventContext::new();
        let created = std::mem::take(&mut self.pending_created_nodes);

        if let Some(close_request) = &self.close_request {
            self.event_manager.send_close_request(*close_request);

            if close_request.requested_at.elapsed() >= close_request.duration {
                self.close_request = None;
                self.quitting = true;
                self.event_manager.send_quit();
            }
        }
        let destruction_queue = self.build_node_destruction_queue();

        self.event_manager
            .poll_lifecycle_events(&destruction_queue, &created);

        self.event_manager
            .dispatch(&mut self.nodes, &mut event_context);

        self.context_action_queue = event_context._take_actions();
        let created = self.process_context_actions();
        self.pending_created_nodes.extend(created);

        if let Some(request) = event_context.close_request {
            self.close_request = Some(request);
        }

        if event_context.is_cancel_close_requested() {
            self.close_request = None;
            self.event_manager.cancel_close_request();
        }

        for node in self.nodes.values_mut() {
            node.process_action_queue();
        }

        for node_id in destruction_queue {
            let _ = self.destroy_node(node_id);
        }
    }

    pub(crate) fn manage_sdl_event(&mut self, sdl_event: &sdl3::event::Event) {
        let mut event_context = EventContext::new();

        self.event_manager.manage_user_event(sdl_event);
        self.event_manager
            .dispatch(&mut self.nodes, &mut event_context);

        self.context_action_queue = event_context._take_actions();
        let created = self.process_context_actions();
        self.pending_created_nodes.extend(created);

        if let Some(request) = event_context.close_request {
            self.close_request = Some(request);
        }

        if event_context.is_cancel_close_requested() {
            self.close_request = None;
            self.event_manager.cancel_close_request();
        }
    }

    fn process_context_actions(&mut self) -> Vec<NodeID> {
        let context_actions = std::mem::take(&mut self.context_action_queue);
        let mut new_nodes: Vec<NodeID> = Vec::new();

        for action in context_actions {
            match action {
                ContextAction::Creation { builder } => {
                    let new_node = self.build_node(builder);
                    let id = new_node.get_id();
                    new_nodes.push(id);
                    self.nodes.insert(id, new_node);
                }
                ContextAction::Destruction { target } => {
                    if let Some(node) = self.nodes.get_mut(&target) {
                        node.destroy();
                    }
                }
                ContextAction::AddChild { parent, child } => {
                    if let Some(parent_node) = self.nodes.get_mut(&parent) {
                        parent_node.add_child(child);
                    }
                }
                ContextAction::AddNodeEventListener {
                    target,
                    event_type,
                    callback,
                } => {
                    self.on_node(target, event_type, callback);
                }
                ContextAction::AddSceneEventListener {
                    event_type,
                    callback,
                } => {
                    self.on_scene(event_type, callback);
                }
            }
        }
        new_nodes
    }

    fn build_node(&mut self, builder: NodeBuilder) -> Node {
        let NodeBuilder {
            transform,
            size,
            style,
            event_listeners,
        } = builder;
        let mut new_node = Node::new();

        for (event_type, callbacks) in event_listeners {
            for callback in callbacks {
                self.on_node(new_node.get_id(), event_type, callback);
            }
        }

        if let Some(layer) = transform.layer {
            new_node.layer(layer);
        }

        let (scale_x, scale_y) = transform.scale;
        new_node.scale(scale_x, scale_y);

        let Color { r, g, b, a } = style.color;
        new_node.bg_color(r, g, b, a);

        let (w, h) = size;
        new_node.size(w, h);

        let Position { x, y } = transform.position;
        new_node.position(x as i32, y as i32);

        new_node
    }

    fn build_node_destruction_queue(&mut self) -> Vec<NodeID> {
        let mut destruction_queue: Vec<NodeID> = Vec::new();
        for node in self.nodes.values_mut() {
            if node.is_destruction_requested() {
                destruction_queue.push(node.get_id());
                if node.children.len() > 0 {
                    for child_id in &node.children {
                        destruction_queue.push(*child_id);
                    }
                }
            }
        }
        destruction_queue
    }

    fn destroy_node(&mut self, node_id: NodeID) -> Result<(), PrismaError> {
        let node = self.node(node_id)?;

        if node.children.len() > 0 {
            for child_id in node.children.clone() {
                let _ = self.destroy_node(child_id);
            }
        }
        self.nodes.remove(&node_id);
        Ok(())
    }
}
