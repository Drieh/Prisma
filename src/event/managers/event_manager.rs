use crate::event::EventContext;
use crate::event::managers::window_manager::{WindowEvent, WindowEventType, WindowManager};
use crate::event::managers::{LifecycleEvent, LifecycleEventType, LifecycleManager};
use crate::event::managers::{MouseEvent, MouseEventType, MouseManager};
use std::collections::HashMap;

use crate::common::Position;
use crate::nodes::Node;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct CloseRequest {
    pub duration: std::time::Duration,
    pub requested_at: std::time::Instant,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum EventType {
    Mouse(MouseEventType),
    Window(WindowEventType),
    Lifecycle(LifecycleEventType),
    AppCloseRequest,
    CancelAppCloseRequest,
    Quit,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Mouse { event: MouseEvent },
    Window { event: WindowEvent },
    Lifecycle { event: LifecycleEvent },
    AppCloseRequest,
    CancelAppCloseRequest,
    Quit,
}
impl Event {
    pub fn event_type(&self) -> EventType {
        match self {
            Event::Mouse { event } => EventType::Mouse(event.event_type()),
            Event::Window { event } => EventType::Window(event.event_type()),
            Event::Lifecycle { event } => EventType::Lifecycle(event.event_type()),
            Event::AppCloseRequest => EventType::AppCloseRequest,
            Event::CancelAppCloseRequest => EventType::CancelAppCloseRequest,
            Event::Quit => EventType::Quit,
        }
    }
}
pub type NodeCallback = Box<dyn FnMut(&mut EventContext, &mut Node) + 'static>;
pub type SceneCallback = Box<dyn FnMut(&mut EventContext) + 'static>;

pub struct EventManager {
    node_event_listeners: HashMap<usize, HashMap<EventType, Vec<NodeCallback>>>,
    scene_event_listeners: HashMap<EventType, Vec<SceneCallback>>,

    close_request: Option<CloseRequest>,

    close_request_dispatched: bool,
    cancel_close_requested: bool,
    quitting: bool,

    mouse_manager: MouseManager,
    window_manager: WindowManager,
    lifecycle_manager: LifecycleManager,
}
impl EventManager {
    pub fn new() -> Self {
        Self {
            node_event_listeners: HashMap::new(),
            scene_event_listeners: HashMap::new(),

            close_request: None,
            close_request_dispatched: false,
            cancel_close_requested: false,
            quitting: false,

            mouse_manager: MouseManager::new(),
            window_manager: WindowManager::new(),
            lifecycle_manager: LifecycleManager::new(),
        }
    }

    pub fn send_close_request(&mut self, close_request: CloseRequest) {
        self.close_request = Some(close_request);
    }
    pub fn cancel_close_request(&mut self) {
        self.cancel_close_requested = true;
        self.close_request = None;
    }
    pub fn send_quit(&mut self) {
        self.close_request = None;
        self.quitting = true;
    }

    pub fn add_event_listener(
        &mut self,
        node_id: usize,
        event_type: EventType,
        callback: NodeCallback,
    ) {
        self.node_event_listeners
            .entry(node_id)
            .or_default()
            .entry(event_type)
            .or_default()
            .push(callback);
    }

    pub fn add_scene_event_listener(&mut self, event_type: EventType, callback: SceneCallback) {
        self.scene_event_listeners
            .entry(event_type)
            .or_default()
            .push(callback);
    }

    pub fn manage_user_event(&mut self, sdl_event: &sdl3::event::Event) {
        self.mouse_manager.handle_sdl_event(sdl_event);
        self.window_manager.handle_sdl_event(sdl_event);
        //self.keyboard_manager.handle_sdl_event(event);
    }

    pub fn poll_lifecycle_events(&mut self, destruction_queue: &[usize], creation_queue: &[usize]) {
        for target in creation_queue {
            self.lifecycle_manager.handle_creation(*target);
        }
        self.lifecycle_manager.handle_update();

        for target in destruction_queue {
            self.lifecycle_manager.handle_destruction(*target);
        }
    }

    /**
     * Builds a list of events from the various managers.
     * Returns the list of events.
     */
    fn build_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();

        for event in self.mouse_manager.take_events() {
            events.push(Event::Mouse { event });
        }
        for event in self.window_manager.take_events() {
            events.push(Event::Window { event });
        }
        for event in self.lifecycle_manager.take_events() {
            events.push(Event::Lifecycle { event });
        }

        if !self.close_request_dispatched && self.close_request.is_some() {
            self.close_request_dispatched = true;
            events.push(Event::AppCloseRequest);
        } else if self.cancel_close_requested {
            self.close_request_dispatched = false;
            self.cancel_close_requested = false;
            self.close_request = None;
            events.push(Event::CancelAppCloseRequest);
        }
        if self.quitting {
            self.quitting = false;
            events.push(Event::Quit);
        }

        events
    }

    fn hit_test(&self, x: f32, y: f32, nodes: &HashMap<usize, Node>) -> Option<usize> {
        let mut result: Option<usize> = None;
        let mut max_layer = i32::MIN;

        for (id, node) in nodes.iter() {
            let (w, h) = node.get_size();

            let Position {
                x: node_x,
                y: node_y,
            } = self.world_position(nodes, *id);

            let inside = x >= node_x as f32
                && y >= node_y as f32
                && x <= (node_x + w as f32)
                && y <= (node_y + h as f32);

            if inside {
                let layer = node.get_transform().layer.unwrap_or(0) as i32;

                if layer >= max_layer {
                    max_layer = layer;
                    result = Some(*id);
                }
            }
        }
        result
    }

    fn world_position(&self, nodes: &HashMap<usize, Node>, id: usize) -> Position {
        let node = nodes.get(&id).expect("Invalid Node ID.");

        if let Some(parent) = node.parent {
            self.world_position(nodes, parent) + node.get_transform().position
        } else {
            node.get_transform().position
        }
    }

    /**
     * Dispatches the events to the appropriate listeners.
     * Contains build_events() so can be called after every poll() to dispatch the events. Even two times per frame, if needed.
     * As it contains build_events(), clears the events from the managers after dispatching them.
     */
    pub fn dispatch(&mut self, nodes: &mut HashMap<usize, Node>, context: &mut EventContext) {
        let events = self.build_events();

        for event in events {
            context.event = Some(event);
            let event_type = event.event_type();

            match &context.event.unwrap() {
                Event::Mouse { event } => {
                    match event {
                        MouseEvent::MouseDown { x, y, .. }
                        | MouseEvent::MouseUp { x, y, .. }
                        | MouseEvent::Click { x, y, .. } => {
                            if let Some(target_id) = self.hit_test(*x, *y, nodes) {
                                self.dispatch_nodes(event_type, context, nodes, Some(target_id));
                            }
                            self.dispatch_scene(event_type, context);
                        }
                        MouseEvent::MouseMove { .. } => {
                            // opcional: hover system después
                            self.dispatch_scene(event_type, context);
                        }
                    }
                }
                Event::Window { .. } => {
                    self.dispatch_scene(event_type, context);
                    self.dispatch_nodes(event_type, context, nodes, None);
                }
                Event::Lifecycle { event } => match event {
                    LifecycleEvent::Destruction { target } => {
                        self.dispatch_scene(event_type, context);
                        self.dispatch_nodes(event_type, context, nodes, Some(*target));
                    }
                    LifecycleEvent::Creation { target } => {
                        self.dispatch_scene(event_type, context);
                        self.dispatch_nodes(event_type, context, nodes, Some(*target));
                    }
                    LifecycleEvent::Update => {
                        self.dispatch_scene(event_type, context);
                        self.dispatch_nodes(event_type, context, nodes, None);
                    }
                },
                Event::AppCloseRequest | Event::Quit => {
                    self.dispatch_scene(event_type, context);
                    self.dispatch_nodes(event_type, context, nodes, None);
                }
                Event::CancelAppCloseRequest => {
                    self.close_request = None;
                    self.dispatch_scene(event_type, context);
                    self.dispatch_nodes(event_type, context, nodes, None);
                }
            }
        }
    }

    fn dispatch_scene(&mut self, event_type: EventType, context: &mut EventContext) {
        if let Some(callbacks) = self.scene_event_listeners.get_mut(&event_type) {
            for callback in callbacks {
                (callback)(context);
            }
        }
    }
    fn dispatch_nodes(
        &mut self,
        event_type: EventType,
        context: &mut EventContext,
        nodes: &mut HashMap<usize, Node>,
        target: Option<usize>,
    ) {
        if let Some(target_id) = target {
            if let Some(node) = nodes.get_mut(&target_id) {
                if let Some(listeners) = self.node_event_listeners.get_mut(&target_id) {
                    if let Some(callbacks) = listeners.get_mut(&event_type) {
                        for callback in callbacks {
                            callback(context, node);
                        }
                    }
                }
            }
        } else {
            for node in nodes.values_mut() {
                if let Some(listeners) = self.node_event_listeners.get_mut(&node.get_id()) {
                    if let Some(callbacks) = listeners.get_mut(&event_type) {
                        for callback in callbacks {
                            callback(context, node);
                        }
                    }
                }
            }
        }
    }
}
