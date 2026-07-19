use crate::event::managers::window_manager::{WindowEvent, WindowEventType, WindowManager};
use crate::event::managers::{LifecycleEvent, LifecycleEventType, LifecycleManager};
use crate::event::managers::{MouseEvent, MouseEventType, MouseManager};
use crate::event::{EventContext, MouseButton};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::common::Position;
use crate::scene::{Node, NodeID};

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

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct EventListenerID(u32);
static NEXT_LISTENER_ID: AtomicU32 = AtomicU32::new(0);
impl EventListenerID {
    pub fn next() -> Self {
        Self(NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for EventListenerID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type NodeCallback = Box<dyn FnMut(&mut EventContext, &mut Node) + 'static>;
pub type SceneCallback = Box<dyn FnMut(&mut EventContext) + 'static>;

pub struct EventManager {
    node_event_listeners:
        HashMap<NodeID, HashMap<EventType, HashMap<EventListenerID, NodeCallback>>>,
    scene_event_listeners: HashMap<EventType, HashMap<EventListenerID, SceneCallback>>,

    node_listeners_lookup: HashMap<EventListenerID, (NodeID, EventType)>,
    scene_listener_lookup: HashMap<EventListenerID, EventType>,

    close_request: Option<CloseRequest>,
    close_request_dispatched: bool,
    cancel_close_requested: bool,
    quitting: bool,

    hovered_node: Option<NodeID>,
    active_node: Option<NodeID>,
    dragged_node: Option<NodeID>,

    mouse_manager: MouseManager,
    window_manager: WindowManager,
    lifecycle_manager: LifecycleManager,
}
impl EventManager {
    pub fn new() -> Self {
        Self {
            node_event_listeners: HashMap::new(),
            scene_event_listeners: HashMap::new(),

            node_listeners_lookup: HashMap::new(),
            scene_listener_lookup: HashMap::new(),

            close_request: None,
            close_request_dispatched: false,
            cancel_close_requested: false,
            quitting: false,

            hovered_node: None,
            active_node: None,
            dragged_node: None,

            mouse_manager: MouseManager::new(),
            window_manager: WindowManager::new(),
            lifecycle_manager: LifecycleManager::new(),
        }
    }

    pub fn is_node_active(&self, target: NodeID) -> bool {
        self.active_node == Some(target)
    }
    pub fn is_node_hovered(&self, target: NodeID) -> bool {
        self.hovered_node == Some(target)
    }

    pub fn send_close_request(&mut self, close_request: CloseRequest) {
        self.close_request = Some(close_request);
    }
    pub fn cancel_close(&mut self) {
        self.cancel_close_requested = true;
        self.close_request = None;
    }
    pub fn send_quit(&mut self) {
        self.close_request = None;
        self.quitting = true;
    }

    pub fn add_node_event_listener(
        &mut self,
        node_id: NodeID,
        event_type: EventType,
        callback: NodeCallback,
    ) -> EventListenerID {
        let id = EventListenerID::next();
        self.node_event_listeners
            .entry(node_id)
            .or_default()
            .entry(event_type)
            .or_default()
            .insert(id, callback);

        self.node_listeners_lookup.insert(id, (node_id, event_type));
        id
    }

    pub fn add_scene_event_listener(
        &mut self,
        event_type: EventType,
        callback: SceneCallback,
    ) -> EventListenerID {
        let id = EventListenerID::next();
        self.scene_event_listeners
            .entry(event_type)
            .or_default()
            .insert(id, callback);
        id
    }

    pub fn remove_event_listener(&mut self, target: EventListenerID) {
        if let Some((node_id, event_type)) = self.node_listeners_lookup.get(&target) {
            self.node_event_listeners
                .get_mut(node_id)
                .expect("Invalid node listener lookup.")
                .get_mut(event_type)
                .expect("Invalid event listener lookup.")
                .remove(&target);

            self.node_listeners_lookup.remove(&target);
        } else if let Some(event_type) = self.scene_listener_lookup.get(&target) {
            self.scene_event_listeners
                .get_mut(event_type)
                .expect("Invalid scene listener lookup.")
                .remove(&target);
            self.scene_listener_lookup.remove(&target);
        }
    }

    pub fn manage_user_event(&mut self, sdl_event: &sdl3::event::Event) {
        self.mouse_manager.handle_sdl_event(sdl_event);
        self.window_manager.handle_sdl_event(sdl_event);
        //self.keyboard_manager.handle_sdl_event(event);
    }

    pub fn poll_lifecycle_events(
        &mut self,
        destruction_queue: &[NodeID],
        creation_queue: &[NodeID],
    ) {
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
    fn take_events(&mut self) -> Vec<Event> {
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

    fn hit_test(&self, x: f32, y: f32, nodes: &HashMap<NodeID, Node>) -> Vec<NodeID> {
        let mut result: Vec<NodeID> = Vec::new();
        let mut max_layer: usize = 0;

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

            let node_layer = node.get_transform().layer.unwrap_or(0);
            if max_layer < node_layer {
                max_layer = node_layer;
            }

            if inside {
                result.push(*id);
            }
        }
        result.sort_by(|node_id_1, node_id_2| {
            let node_layer_1 = nodes
                .get(node_id_1)
                .unwrap()
                .get_transform()
                .layer
                .unwrap_or(0);

            let node_layer_2 = nodes
                .get(node_id_2)
                .unwrap()
                .get_transform()
                .layer
                .unwrap_or(0);

            if node_layer_1 > node_layer_2 {
                std::cmp::Ordering::Greater
            } else if node_layer_1 < node_layer_2 {
                std::cmp::Ordering::Less
            } else {
                if node_id_1 > node_id_2 {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            }
        });

        result
    }

    fn world_position(&self, nodes: &HashMap<NodeID, Node>, id: NodeID) -> Position {
        let node = nodes.get(&id).expect("Invalid Node ID.");

        if let Some(parent) = node.get_parent() {
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
    pub fn dispatch(&mut self, nodes: &mut HashMap<NodeID, Node>, context: &mut EventContext) {
        let events = self.take_events();

        for event in events {
            context.event = Some(event);
            let event_type = event.event_type();

            self.dispatch_scene(event_type, context);
            match event {
                Event::Mouse { event } => match event {
                    MouseEvent::MouseDown { x, y, .. } => {
                        if let Some(target_id) = self.hit_test(x, y, nodes).pop() {
                            self.dispatch_node(event_type, context, nodes, target_id);

                            self.active_node = Some(target_id);
                        }
                    }
                    MouseEvent::MouseUp { .. } => {
                        self.dispatch_all_nodes(event_type, context, nodes);
                        self.active_node = None;
                    }
                    MouseEvent::MouseMove { x, y } => {
                        self.dispatch_all_nodes(event_type, context, nodes);
                        if let Some(target) = self.hit_test(x, y, nodes).pop() {
                            self.hovered_node = Some(target);
                        } else {
                            self.hovered_node = None;
                        }
                    }
                    MouseEvent::Click { .. } => {
                        if let Some(active) = self.active_node {
                            self.dispatch_node(event_type, context, nodes, active);
                        }
                    }
                    MouseEvent::DragStart { x, y, .. } => {
                        if let Some(target) = self.hit_test(x, y, nodes).pop() {
                            self.dispatch_node(event_type, context, nodes, target);
                            self.dragged_node = Some(target);
                        }
                    }
                    MouseEvent::Drag { .. } => {
                        if let Some(target) = self.dragged_node {
                            self.dispatch_node(event_type, context, nodes, target);
                        }
                    }
                    MouseEvent::DragEnd { .. } => {
                        if let Some(target_id) = self.dragged_node {
                            self.dispatch_node(event_type, context, nodes, target_id);
                        }
                        self.dragged_node = None;
                    }
                    _ => {}
                },
                Event::Window { .. } => {
                    self.dispatch_all_nodes(event_type, context, nodes);
                }
                Event::Lifecycle { event } => match event {
                    LifecycleEvent::Destruction { target } => {
                        self.dispatch_node(event_type, context, nodes, target);
                    }
                    LifecycleEvent::Creation { target } => {
                        self.dispatch_node(event_type, context, nodes, target);
                    }
                    LifecycleEvent::Update => {
                        self.dispatch_all_nodes(event_type, context, nodes);
                    }
                },
                Event::AppCloseRequest | Event::Quit => {
                    self.dispatch_all_nodes(event_type, context, nodes);
                }
                Event::CancelAppCloseRequest => {
                    self.close_request = None;
                    self.dispatch_all_nodes(event_type, context, nodes);
                }
            }
        }
    }

    fn dispatch_scene(&mut self, event_type: EventType, context: &mut EventContext) {
        if let Some(callbacks) = self.scene_event_listeners.get_mut(&event_type) {
            for callback in callbacks.values_mut() {
                println!("scene context event: {:?}", context.event);
                (callback)(context);
            }
        }
    }
    fn dispatch_all_nodes(
        &mut self,
        event_type: EventType,
        context: &mut EventContext,
        nodes: &mut HashMap<NodeID, Node>,
    ) {
        for node in nodes.values_mut() {
            let Some(listeners) = self.node_event_listeners.get_mut(&node.id()) else {
                continue;
            };
            let Some(callbacks) = listeners.get_mut(&event_type) else {
                continue;
            };
            for callback in callbacks.values_mut() {
                callback(context, node);
            }
        }
    }
    fn dispatch_node(
        &mut self,
        event_type: EventType,
        context: &mut EventContext,
        nodes: &mut HashMap<NodeID, Node>,
        target: NodeID,
    ) {
        let mut dispatch_to_parent = false;
        let node = nodes.get_mut(&target).expect("Invalid ID");

        println!("dispatch node context event: {:?}", context.event);
        if let Some(listeners) = self.node_event_listeners.get_mut(&target) {
            if let Some(callbacks) = listeners.get_mut(&event_type) {
                for callback in callbacks.values_mut() {
                    callback(context, node);
                }
            } else {
                dispatch_to_parent = true;
            }
        } else {
            dispatch_to_parent = true;
        }

        if dispatch_to_parent && let Some(parent) = node.get_parent() {
            self.dispatch_node(event_type, context, nodes, parent);
        }
    }
}
