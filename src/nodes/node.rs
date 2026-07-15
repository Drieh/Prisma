use std::{any::Any, collections::HashMap, fmt::Display, sync::atomic::AtomicU32};

use crate::{
    common::Position,
    nodes::components::{Rect, Style, Transform},
};
use sdl3::{pixels::Color, render::Canvas, video::Window};
use std::{
    collections::VecDeque,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum Action {
    Position { x: i32, y: i32 },
    Scale { x: f32, y: f32 },
    Size { width: u32, height: u32 },
    BGColor { color: Color },
    Layer { layer: usize },
    Wait { duration: Duration },
    DestructionRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NodeID(u32);

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

impl NodeID {
    pub fn next() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Node {
    transform: Transform,
    id: NodeID,
    rect: Rect,
    style: Style,
    state: HashMap<String, Box<dyn Any>>,

    waiting_until: Option<Instant>,
    action_queue: VecDeque<Action>,

    destruction_requested: bool,

    pub(crate) children: Vec<NodeID>,
    pub(crate) parent: Option<NodeID>,
}

impl Node {
    pub(crate) fn new() -> Self {
        Self {
            id: NodeID::next(),
            transform: Transform::new(),
            style: Style::new(),
            rect: Rect::new(),
            state: HashMap::new(),

            action_queue: VecDeque::new(),
            waiting_until: None,

            destruction_requested: false,

            children: Vec::new(),
            parent: None,
        }
    }
    // state control
    pub fn set_state<T: Any>(&mut self, key: impl Into<String>, value: T) {
        self.state.insert(key.into(), Box::new(value));
    }

    pub fn get_state<T: Any>(&self, key: &str) -> Option<&T> {
        self.state
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
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

    pub fn get_id(&self) -> NodeID {
        self.id
    }
    pub fn set_parent(&mut self, parent: Option<NodeID>) -> &mut Self {
        self.parent = parent;
        self
    }
    pub fn get_parent(&self) -> Option<NodeID> {
        self.parent
    }

    pub(crate) fn add_child(&mut self, child_id: NodeID) {
        self.children.push(child_id);
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
        self.action_queue.push_back(Action::DestructionRequest);
    }

    pub fn is_destruction_requested(&mut self) -> bool {
        self.destruction_requested
    }

    // Node builder
    pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
        self.action_queue.push_back(Action::Position { x, y });
        self
    }
    /*
    pub fn rotation(&mut self, angle: f32) -> &mut Self {
        self.transform.rotation = angle;
        self
    }
     */
    pub fn scale(&mut self, x: f32, y: f32) -> &mut Self {
        self.action_queue.push_back(Action::Scale { x, y });
        self
    }
    pub fn size(&mut self, width: u32, height: u32) -> &mut Self {
        self.action_queue.push_back(Action::Size { width, height });
        self
    }
    pub fn bg_color(&mut self, r: u8, g: u8, b: u8, a: u8) -> &mut Self {
        self.action_queue.push_back(Action::BGColor {
            color: Color { r, g, b, a },
        });
        self
    }
    pub fn layer(&mut self, layer: usize) -> &mut Node {
        self.action_queue.push_back(Action::Layer { layer });
        self
    }

    // Properties change management
    pub fn wait(&mut self, ms: u64) -> &mut Self {
        self.action_queue.push_back(Action::Wait {
            duration: Duration::from_millis(ms),
        });
        self
    }
    pub fn process_action_queue(&mut self) {
        if let Some(until) = self.waiting_until {
            if Instant::now() < until {
                return;
            }
            self.waiting_until = None;
        }

        if let Some(action) = self.action_queue.pop_front() {
            match action {
                Action::Position { x, y } => {
                    self.transform.position = Position {
                        x: x as f32,
                        y: y as f32,
                    };
                }
                Action::Size { width, height } => {
                    self.rect.size(width, height);
                }
                Action::Scale { x, y } => {
                    self.transform.scale = (x, y);
                }
                Action::BGColor { color } => {
                    self.style.color = color;
                }
                Action::Layer { layer } => {
                    self.transform.layer = Some(layer);
                }
                Action::Wait { duration } => {
                    self.waiting_until = Some(Instant::now() + duration);
                }
                Action::DestructionRequest => {
                    self.destruction_requested = true;
                }
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
    /*
    pub fn remove_child(&mut self, child_id: usize) {
        let index = self
            ._children
            .iter()
            .position(|child| child.get_id() == child_id);

        if let Some(index) = index {
            self._children.remove(index);
        }
    }

     */
}
