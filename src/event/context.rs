use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl3::pixels::Color;

use crate::{
    common::Position,
    event::{
        Event, EventType, NodeCallback, SceneCallback,
        managers::event_manager::{CloseRequest, EventListenerID},
    },
    scene::{
        Node, NodeID,
        components::{Style, Transform},
    },
};

pub enum ContextAction {
    Destruction {
        target: NodeID,
    },
    AddChild {
        parent: NodeID,
        child: NodeID,
    },
    RemoveChild {
        parent: NodeID,
        child: NodeID,
    },
    AddSceneEventListener {
        event_type: EventType,
        callback: SceneCallback,
    },
    RemoveSceneEventListener {
        target: EventListenerID,
    },
}

pub struct EventContext {
    pub event: Option<Event>,
    pub(crate) close_request: Option<CloseRequest>,

    propagation_stopped: bool,
    action_queue: Vec<ContextAction>,
    new_nodes: Vec<Node>,
    cancel_close_requested: bool,
}
impl EventContext {
    pub fn new() -> Self {
        Self {
            event: None,
            close_request: None,

            propagation_stopped: false,
            action_queue: Vec::new(),
            new_nodes: Vec::new(),
            cancel_close_requested: false,
        }
    }

    pub fn on_scene(&mut self, event_type: EventType, callback: SceneCallback) {
        self.action_queue
            .push(ContextAction::AddSceneEventListener {
                event_type,
                callback,
            });
    }
    pub fn off_scene(&mut self, target: EventListenerID) {
        self.action_queue
            .push(ContextAction::RemoveSceneEventListener { target });
    }

    pub fn new_node(&mut self) -> &mut Node {
        let node = Node::new();
        let id = node.id();
        self.new_nodes.push(node);
        self.new_nodes
            .iter_mut()
            .find(|node| node.id() == id)
            .unwrap()
    }

    pub fn take_nodes(&mut self) -> Vec<Node> {
        std::mem::take(&mut self.new_nodes)
    }

    pub fn destroy(&mut self, target: NodeID) {
        self.action_queue
            .push(ContextAction::Destruction { target });
    }

    pub fn add_child(&mut self, parent: NodeID, child: NodeID) {
        self.action_queue
            .push(ContextAction::AddChild { parent, child });
    }
    pub fn remove_child(&mut self, parent: NodeID, child: NodeID) {
        self.action_queue
            .push(ContextAction::RemoveChild { parent, child });
    }

    pub fn close(&mut self, timer: u64) {
        self.cancel_close_requested = false;
        self.close_request = Some(CloseRequest {
            duration: Duration::from_millis(timer),
            requested_at: Instant::now(),
        })
    }

    /**
     * Doesn't work in Event::Quit
     */
    pub fn cancel_close(&mut self) {
        self.cancel_close_requested = true;
        self.close_request = None;
    }

    pub fn is_cancel_close_requested(&self) -> bool {
        self.cancel_close_requested
    }
    pub(crate) fn take_actions(&mut self) -> Vec<ContextAction> {
        std::mem::take(&mut self.action_queue)
    }
}
