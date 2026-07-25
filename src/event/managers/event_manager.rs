use crate::event::EventContext;
use crate::event::context::{CloseRequest, PropagationState};
use crate::event::managers::{LifecycleEvent, LifecycleEventType, LifecycleManager};
use crate::event::managers::{MouseEvent, MouseEventType, MouseManager};
use crate::event::managers::{WindowEvent, WindowEventType, WindowManager};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::scene::nodes::NodeID;
use crate::util::Position;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum EventType {
    Mouse(MouseEventType),
    Window(WindowEventType),
    Lifecycle(LifecycleEventType),
    AppCloseRequest,
    CancelAppCloseRequest,
    Quit,
}
impl EventType {
    pub fn bubbles_by_default(&self) -> bool {
        match self {
            // excluded events
            EventType::Lifecycle(..)
            | EventType::Mouse(MouseEventType::Enter)
            | EventType::Mouse(MouseEventType::Leave)
            | EventType::Mouse(MouseEventType::MouseMove)
            | EventType::Mouse(MouseEventType::DragStart)
            | EventType::Mouse(MouseEventType::Drag)
            | EventType::Mouse(MouseEventType::DragEnd)
            | EventType::AppCloseRequest
            | EventType::CancelAppCloseRequest
            | EventType::Quit
            | EventType::Window(..) => false,

            EventType::Mouse(MouseEventType::MouseDown)
            | EventType::Mouse(MouseEventType::MouseUp)
            | EventType::Mouse(MouseEventType::Click) => true,
        }
    }
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
    pub fn get_type(&self) -> EventType {
        match self {
            Event::Mouse { event } => EventType::Mouse(event.get_type()),
            Event::Window { event } => EventType::Window(event.event_type()),
            Event::Lifecycle { event } => EventType::Lifecycle(event.event_type()),
            Event::AppCloseRequest => EventType::AppCloseRequest,
            Event::CancelAppCloseRequest => EventType::CancelAppCloseRequest,
            Event::Quit => EventType::Quit,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct CallbackID(u32);
static NEXT_LISTENER_ID: AtomicU32 = AtomicU32::new(0);
impl CallbackID {
    pub fn next() -> Self {
        Self(NEXT_LISTENER_ID.fetch_add(1, Ordering::Relaxed))
    }
}
impl Display for CallbackID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Callback {
    pub id: CallbackID,
    pub callback: Box<dyn FnMut(&mut EventContext) + 'static>,
}

pub struct EventManager {
    node_event_listeners: HashMap<NodeID, HashMap<EventType, HashMap<CallbackID, Callback>>>,
    scene_event_listeners: HashMap<EventType, HashMap<CallbackID, Callback>>,

    node_listeners_lookup: HashMap<CallbackID, (NodeID, EventType)>,
    scene_listener_lookup: HashMap<CallbackID, EventType>,

    close_request: Option<CloseRequest>,
    close_request_dispatched: bool,
    quit_request_dispatched: bool,
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
            quit_request_dispatched: false,
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

    pub fn add_node_event_listener<F>(
        &mut self,
        node_id: NodeID,
        event_type: EventType,
        callback: F,
    ) -> CallbackID
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        let id = CallbackID::next();
        self.node_event_listeners
            .entry(node_id)
            .or_default()
            .entry(event_type)
            .or_default()
            .insert(
                id,
                Callback {
                    id,
                    callback: Box::new(callback),
                },
            );

        self.node_listeners_lookup.insert(id, (node_id, event_type));
        id
    }

    pub fn add_scene_event_listener<F>(&mut self, event_type: EventType, callback: F) -> CallbackID
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        let id = CallbackID::next();
        self.scene_event_listeners
            .entry(event_type)
            .or_default()
            .insert(
                id,
                Callback {
                    id,
                    callback: Box::new(callback),
                },
            );
        id
    }

    pub fn remove_event_listener(&mut self, target: CallbackID) {
        // nodes
        if let Some((node_id, event_type)) = self.node_listeners_lookup.get(&target) {
            self.node_event_listeners
                .get_mut(node_id)
                .expect("Invalid node listener lookup.")
                .get_mut(event_type)
                .expect("Invalid event listener lookup.")
                .remove(&target);
            self.node_listeners_lookup.remove(&target);
        }
        // scene
        else if let Some(event_type) = self.scene_listener_lookup.get(&target) {
            self.scene_event_listeners
                .get_mut(event_type)
                .expect("Invalid scene listener lookup.")
                .remove(&target);
            self.scene_listener_lookup.remove(&target);
        }
    }

    pub fn clear_node_listeners(&mut self, id: NodeID) {
        self.node_event_listeners.remove(&id);
    }

    pub fn manage_sdl_event(&mut self, sdl_event: &sdl3::event::Event) {
        self.mouse_manager.handle_sdl_event(sdl_event);
        self.window_manager.handle_sdl_event(sdl_event);
        //self.keyboard_manager.handle_sdl_event(event);
    }

    pub fn manage_lifecycle_events(
        &mut self,
        pending_created: &Vec<NodeID>,
        pending_destroyed: &Vec<NodeID>,
    ) {
        let mut already_updated: HashSet<NodeID> = HashSet::new();
        for (target, event_type) in self.node_listeners_lookup.values() {
            if *event_type != EventType::Lifecycle(LifecycleEventType::Update) {
                continue;
            }
            if pending_created.contains(target) || pending_destroyed.contains(target) {
                continue;
            }
            if !already_updated.insert(*target) {
                continue;
            }
            self.lifecycle_manager.handle_update(*target);
        }
        for target in pending_created {
            self.lifecycle_manager.handle_creation(*target);
        }
        for target in pending_destroyed {
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
        if self.quitting && !self.quit_request_dispatched {
            self.quit_request_dispatched = true;
            events.push(Event::Quit);
        }

        events
    }

    fn hit_test(&self, x: f32, y: f32, context: &mut EventContext) -> Vec<NodeID> {
        let mut result: Vec<NodeID> = Vec::new();
        let mut max_layer: usize = 0;

        for id in context.storage().get_nodes() {
            let mut node = context
                .get_node(id)
                .expect("Invariant violated: node tree contains an invalid ID.");
            let (w, h) = node.get_bouding_box_size();
            let Position {
                x: node_x,
                y: node_y,
            } = node.get_world_position();

            let inside =
                x >= node_x && y >= node_y && x <= (node_x + w as f32) && y <= (node_y + h as f32);

            let node_layer = node.get_transform().layer.unwrap_or(0);
            if max_layer < node_layer {
                max_layer = node_layer;
            }

            if inside {
                result.push(id);
            }
        }
        result.sort_by(|node_id_1, node_id_2| {
            let node_layer_1 = context
                .storage()
                .transform
                .get_unchecked(*node_id_1)
                .layer
                .unwrap_or(0);
            let node_layer_2 = context
                .storage()
                .transform
                .get_unchecked(*node_id_2)
                .layer
                .unwrap_or(0);

            if node_layer_1 > node_layer_2 {
                std::cmp::Ordering::Greater
            } else if node_layer_1 < node_layer_2 {
                std::cmp::Ordering::Less
            } else {
                node_id_1.cmp(node_id_2)
            }
        });

        result
    }

    /**
     * Dispatches the events to the appropriate listeners.
     * Contains build_events() so can be called after every poll() to dispatch the events. Even two times per frame, if needed.
     * As it contains build_events(), clears the events from the managers after dispatching them.
     */
    pub fn dispatch(&mut self, context: &mut EventContext) {
        let events = self.build_events();

        for event in events {
            context.event = Some(event);
            let event_type = event.get_type();

            self.dispatch_scene(event_type, context);

            match event {
                Event::Mouse { event } => {
                    self.dispatch_mouse_event(event_type, context, event);
                }
                Event::Window { .. } => {
                    self.dispatch_window_event(event_type, context);
                }
                Event::Lifecycle { event } => {
                    self.dispatch_lifecycle_event(event_type, context, event)
                }
                Event::AppCloseRequest | Event::Quit => {
                    self.dispatch_all_nodes(event_type, context);
                }
                Event::CancelAppCloseRequest => {
                    self.close_request = None;
                    self.dispatch_all_nodes(event_type, context);
                }
            }
        }
    }
    fn dispatch_mouse_event(
        &mut self,
        event_type: EventType,
        context: &mut EventContext,
        event: MouseEvent,
    ) {
        match event {
            MouseEvent::MouseDown { x, y, .. } => {
                if let Some(target) = self.hit_test(x, y, context).pop() {
                    context.target = Some(target);
                    self.dispatch_node(event_type, context, target);
                    self.active_node = Some(target);
                }
            }
            MouseEvent::MouseUp { .. } => {
                if let Some(active) = self.active_node {
                    context.target = Some(active);
                    self.dispatch_node(event_type, context, active);
                }
                self.active_node = None;
            }
            MouseEvent::MouseMove { x, y } => {
                self.dispatch_all_nodes(event_type, context);
                let hit_test = self.hit_test(x, y, context).pop();

                if let Some(target) = hit_test {
                    if Some(target) != self.hovered_node {
                        if let Some(hovered) = self.hovered_node {
                            context.target = Some(hovered);
                            context.event = Some(Event::Mouse {
                                event: MouseEvent::Leave { x, y },
                            });
                            let event_type = context.event.unwrap().get_type();
                            self.dispatch_node(event_type, context, hovered);
                        }
                        context.target = Some(target);
                        context.event = Some(Event::Mouse {
                            event: MouseEvent::Enter { x, y },
                        });
                        let event_type = context.event.unwrap().get_type();
                        self.dispatch_node(event_type, context, target);

                        self.hovered_node = Some(target);
                    }
                } else {
                    if let Some(hovered) = self.hovered_node {
                        context.target = Some(hovered);
                        context.event = Some(Event::Mouse {
                            event: MouseEvent::Leave { x, y },
                        });
                        let event_type = context.event.unwrap().get_type();
                        self.dispatch_node(event_type, context, hovered);
                        self.hovered_node = None;
                    }
                }
            }
            MouseEvent::Click { .. } => {
                if let Some(target) = self.active_node {
                    context.target = Some(target);
                    self.dispatch_node(event_type, context, target);
                }
            }
            MouseEvent::DragStart { x, y, .. } => {
                if let Some(target) = self.hit_test(x, y, context).pop() {
                    context.target = Some(target);
                    self.dispatch_node(event_type, context, target);
                    self.dragged_node = Some(target);
                }
            }
            MouseEvent::Drag { .. } => {
                if let Some(target) = self.dragged_node {
                    context.target = Some(target);
                    self.dispatch_node(event_type, context, target);
                }
            }
            MouseEvent::DragEnd { .. } => {
                if let Some(target) = self.dragged_node {
                    context.target = Some(target);
                    self.dispatch_node(event_type, context, target);
                }
                self.dragged_node = None;
            }
            _ => {}
        }
    }

    fn dispatch_window_event(&mut self, event_type: EventType, context: &mut EventContext) {
        self.dispatch_all_nodes(event_type, context);
    }

    fn dispatch_lifecycle_event(
        &mut self,
        event_type: EventType,
        context: &mut EventContext,
        event: LifecycleEvent,
    ) {
        match event {
            LifecycleEvent::Creation { target }
            | LifecycleEvent::Update { target }
            | LifecycleEvent::Destruction { target } => {
                context.target = Some(target);
                self.dispatch_node(event_type, context, target);
            }
        }
    }

    fn dispatch_scene(&mut self, event_type: EventType, context: &mut EventContext) {
        if let Some(callbacks) = self.scene_event_listeners.get_mut(&event_type) {
            for callback in callbacks.values_mut() {
                context.current_callback = Some(callback.id);
                (callback.callback)(context);
            }
        }
    }
    fn dispatch_all_nodes(&mut self, event_type: EventType, context: &mut EventContext) {
        for node_id in context.get_nodes() {
            self.dispatch_node(event_type, context, node_id);
        }
    }
    fn dispatch_node(&mut self, event_type: EventType, context: &mut EventContext, target: NodeID) {
        context.current_target = Some(target);

        if let Some(listeners) = self.node_event_listeners.get_mut(&target)
            && let Some(callbacks) = listeners.get_mut(&event_type)
        {
            for callback in callbacks.values_mut() {
                context.current_callback = Some(callback.id);
                (callback.callback)(context);
            }
        }

        let parent_opt = context
            .storage()
            .tree
            .get(target)
            .expect("Invalid ID in dispatch.")
            .get_parent();

        if let Some(parent) = parent_opt {
            match context.propagation_state() {
                PropagationState::None => {
                    if event_type.bubbles_by_default() {
                        self.dispatch_node(event_type, context, parent);
                    }
                }
                PropagationState::Bubble => {
                    self.dispatch_node(event_type, context, parent);
                }
                PropagationState::Stopped => {}
            }
        }
    }
}
