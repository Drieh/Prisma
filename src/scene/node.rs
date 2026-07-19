use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    fmt::Display,
    sync::atomic::AtomicU32,
};

use crate::{
    app::PrismaError,
    common::Position,
    event::{EventType, NodeCallback, managers::event_manager::EventListenerID},
    scene::components::{Rect, Style, Transform},
};
use sdl3::{pixels::Color, render::Canvas, video::Window};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeActionType {
    Position,
    Scale,
    Size,
    BGColor,
    Layer,
    Wait,
    DestructionRequest,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeAction {
    Position { x: i32, y: i32 },
    Scale { x: f32, y: f32 },
    Size { width: u32, height: u32 },
    BGColor { color: Color },
    Layer { layer: usize },
    Wait { duration: Duration },
    DestructionRequest,
}
impl NodeAction {
    fn get_type(&self) -> NodeActionType {
        match self {
            NodeAction::BGColor { .. } => NodeActionType::BGColor,
            NodeAction::DestructionRequest { .. } => NodeActionType::DestructionRequest,
            NodeAction::Layer { .. } => NodeActionType::Layer,
            NodeAction::Position { .. } => NodeActionType::Position,
            NodeAction::Scale { .. } => NodeActionType::Scale,
            NodeAction::Size { .. } => NodeActionType::Size,
            NodeAction::Wait { .. } => NodeActionType::Wait,
        }
    }
}
pub enum EventListenerAction {
    Add {
        event_type: EventType,
        callback: NodeCallback,
    },
    Remove {
        target: EventListenerID,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NodeID(u32);

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

impl NodeID {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub(crate) fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Node {
    transform: Transform,
    id: NodeID,
    rect: Rect,
    style: Style,
    state: HashMap<String, Box<dyn Any>>,

    waiting_until: Option<Instant>,
    action_queue: VecDeque<NodeAction>,
    og_state: HashMap<NodeActionType, NodeAction>,
    on_active: HashMap<NodeActionType, NodeAction>,
    on_hover: HashMap<NodeActionType, NodeAction>,
    last_action: Option<NodeAction>,
    event_listener_action_queue: Vec<EventListenerAction>,

    destruction_requested: bool,

    children: Vec<NodeID>,
    parent: Option<NodeID>,
}
impl Node {
    pub fn new() -> Self {
        Self {
            id: NodeID::next(),
            transform: Transform::new(),
            style: Style::new(),
            rect: Rect::new(),
            state: HashMap::new(),

            waiting_until: None,
            action_queue: VecDeque::new(),
            og_state: HashMap::new(),
            on_active: HashMap::new(),
            on_hover: HashMap::new(),
            last_action: None,
            event_listener_action_queue: Vec::new(),

            destruction_requested: false,

            children: Vec::new(),
            parent: None,
        }
    }

    // state control
    pub fn set_state<T: Any>(&mut self, key: impl Into<String>, value: T) {
        self.state.insert(key.into(), Box::new(value));
    }

    pub fn get_state<T: Any>(&self, key: &str) -> Result<&T, PrismaError> {
        self.state
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
            .ok_or(PrismaError::NodeStateNotFound(key.to_string()))
    }

    pub fn get_state_mut<T: Any>(&mut self, key: &str) -> Option<&mut T> {
        self.state
            .get_mut(key)
            .and_then(|value| value.downcast_mut::<T>())
    }

    pub fn remove_state<T: Any>(&mut self, key: &str) -> Option<T> {
        self.state
            .remove(key)
            .and_then(|value| value.downcast::<T>().ok())
            .map(|value| *value)
    }

    pub fn has_state(&self, key: &str) -> bool {
        self.state.contains_key(key)
    }

    pub fn on_event(&mut self, event_type: EventType, callback: NodeCallback) -> &mut Self {
        self.event_listener_action_queue
            .push(EventListenerAction::Add {
                event_type,
                callback,
            });
        self
    }

    pub fn off_event(&mut self, target: EventListenerID) -> &mut Self {
        self.event_listener_action_queue
            .push(EventListenerAction::Remove { target });
        self
    }

    pub fn on_active(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.on_active.insert(action.get_type(), *action);
        }
        self
    }
    pub fn on_hover(&mut self, actions: &[NodeAction]) -> &mut Self {
        for action in actions {
            self.on_hover.insert(action.get_type(), *action);
        }
        self
    }

    pub(crate) fn take_event_listener_actions(&mut self) -> Vec<EventListenerAction> {
        std::mem::take(&mut self.event_listener_action_queue)
    }

    pub fn id(&self) -> NodeID {
        self.id
    }
    pub(crate) fn set_parent(&mut self, parent: Option<NodeID>) -> &mut Self {
        self.parent = parent;
        self
    }
    pub fn get_parent(&self) -> Option<NodeID> {
        self.parent
    }
    pub fn get_children(&self) -> Vec<NodeID> {
        self.children.clone()
    }

    pub(crate) fn add_child(&mut self, child_id: NodeID) {
        self.children.push(child_id);
    }

    pub(crate) fn remove_child(&mut self, child_id: NodeID) {
        if let Some(i) = self.children.iter().position(|id| *id == child_id) {
            self.children.remove(i);
        }
    }

    pub fn get_transform(&self) -> Transform {
        self.transform
    }
    pub fn get_transform_as_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    pub fn get_size(&self) -> (u32, u32) {
        self.rect.get_size()
    }

    // Node request management

    pub fn destroy(&mut self) {
        if self.last_action == Some(NodeAction::DestructionRequest) {
            return;
        }
        self.action_queue.push_back(NodeAction::DestructionRequest);
    }

    pub fn is_destruction_requested(&mut self) -> bool {
        self.destruction_requested
    }

    // Property changes management
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        if self.last_action == Some(NodeAction::Position { x, y }) {
            return self;
        }
        self.action_queue.push_back(NodeAction::Position { x, y });
        self
    }
    /*
    pub fn rotation(&mut self, angle: f32) -> &mut Self {
        self.transform.rotation = angle;
        self
    }
     */
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        if self.last_action == Some(NodeAction::Scale { x, y }) {
            return self;
        }
        self.action_queue.push_back(NodeAction::Scale { x, y });
        self
    }
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        if self.last_action == Some(NodeAction::Size { width, height }) {
            return self;
        }
        self.action_queue
            .push_back(NodeAction::Size { width, height });
        self
    }
    pub fn bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) -> &mut Self {
        if self.last_action
            == Some(NodeAction::BGColor {
                color: Color { r, g, b, a },
            })
        {
            println!("no added bg color");
            return self;
        }
        self.action_queue.push_back(NodeAction::BGColor {
            color: Color { r, g, b, a },
        });
        println!("added bg color");
        self
    }
    pub fn layer(&mut self, layer: usize) -> &mut Node {
        if self.last_action == Some(NodeAction::Layer { layer }) {
            return self;
        }
        self.action_queue.push_back(NodeAction::Layer { layer });
        self
    }
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.action_queue.push_back(NodeAction::Wait {
            duration: Duration::from_millis(ms),
        });
        self
    }
    pub(crate) fn process_action_queue(&mut self, is_hovered: bool, is_active: bool) {
        for action in self.og_state.clone().values() {
            self.execute_action(action);
        }
        if is_hovered {
            for action in self.on_hover.clone().values() {
                self.execute_action(action);
            }
        }
        if is_active {
            for action in self.on_active.clone().values() {
                self.execute_action(action);
            }
        }
        if let Some(until) = self.waiting_until {
            if Instant::now() < until {
                return;
            }
            self.waiting_until = None;
        }
        if let Some(action) = self.action_queue.pop_front() {
            self.last_action = Some(action);
            self.og_state.insert(action.get_type(), action);
        }
    }

    fn execute_action(&mut self, action: &NodeAction) {
        match action {
            NodeAction::Position { x, y } => {
                self.transform.position = Position {
                    x: *x as f32,
                    y: *y as f32,
                };
            }
            NodeAction::Size { width, height } => {
                self.rect.size(*width, *height);
            }
            NodeAction::Scale { x, y } => {
                self.transform.scale = (*x, *y);
            }
            NodeAction::BGColor { color } => {
                self.style.color = *color;
            }
            NodeAction::Layer { layer } => {
                self.transform.layer = Some(*layer);
            }
            NodeAction::Wait { duration } => {
                self.waiting_until = Some(Instant::now() + *duration);
            }
            NodeAction::DestructionRequest => {
                self.destruction_requested = true;
            }
        }
    }

    pub(crate) fn draw(&self, canvas: &mut Canvas<Window>, world_position: Position) {
        let node_transform = self.get_transform();

        let draw_transform = Transform {
            position: world_position,
            ..node_transform
        };

        self.rect.draw(canvas, &draw_transform, &self.style);
    }
}
