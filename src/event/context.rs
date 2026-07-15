use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use sdl3::pixels::Color;

use crate::{
    common::Position,
    event::{Event, EventType, NodeCallback, SceneCallback, managers::event_manager::CloseRequest},
    nodes::{
        NodeID,
        components::{Style, Transform},
    },
};

pub enum ContextAction {
    Creation {
        builder: NodeBuilder,
    },
    Destruction {
        target: NodeID,
    },
    AddChild {
        parent: NodeID,
        child: NodeID,
    },
    AddNodeEventListener {
        target: NodeID,
        event_type: EventType,
        callback: NodeCallback,
    },
    AddSceneEventListener {
        event_type: EventType,
        callback: SceneCallback,
    },
}

pub struct EventContext {
    pub event: Option<Event>,
    pub close_request: Option<CloseRequest>,

    propagation_stopped: bool,
    action_queue: Vec<ContextAction>,
    cancel_close_requested: bool,
}
impl EventContext {
    pub fn new() -> Self {
        Self {
            event: None,
            close_request: None,

            propagation_stopped: false,
            action_queue: Vec::new(),
            cancel_close_requested: false,
        }
    }
    pub fn on_node(&mut self, target: NodeID, event_type: EventType, callback: NodeCallback) {
        self.action_queue.push(ContextAction::AddNodeEventListener {
            target,
            event_type,
            callback,
        });
    }

    pub fn on_scene(&mut self, event_type: EventType, callback: SceneCallback) {
        self.action_queue
            .push(ContextAction::AddSceneEventListener {
                event_type,
                callback,
            });
    }

    pub fn new_node_builder(&mut self) -> NodeBuilder {
        NodeBuilder {
            transform: Transform::new(),
            size: (0, 0),
            style: Style::new(),
            event_listeners: HashMap::new(),
        }
    }

    pub fn build_node(&mut self, builder: NodeBuilder) {
        self.action_queue.push(ContextAction::Creation { builder });
    }

    pub fn destroy(&mut self, target: NodeID) {
        self.action_queue
            .push(ContextAction::Destruction { target });
    }

    pub fn add_child(&mut self, parent: NodeID, child: NodeID) {
        self.action_queue
            .push(ContextAction::AddChild { parent, child });
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
    pub fn _take_actions(&mut self) -> Vec<ContextAction> {
        std::mem::take(&mut self.action_queue)
    }
}

pub struct NodeBuilder {
    pub transform: Transform,
    pub size: (u32, u32),
    pub style: Style,
    pub event_listeners: HashMap<EventType, Vec<NodeCallback>>,
}
impl NodeBuilder {
    pub fn on(mut self, event_type: EventType, callback: NodeCallback) -> Self {
        self.event_listeners
            .entry(event_type)
            .or_default()
            .push(callback);
        self
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.transform.position = Position {
            x: x as f32,
            y: y as f32,
        };
        self
    }
    pub fn rotation(mut self, angle: f32) -> Self {
        self.transform.rotation = angle;
        self
    }
    pub fn scale(mut self, x: f32, y: f32) -> Self {
        self.transform.scale = (x, y);
        self
    }
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }
    pub fn bg_color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }
    pub fn layer(mut self, layer: usize) -> Self {
        self.transform.layer = Some(layer);
        self
    }
}
