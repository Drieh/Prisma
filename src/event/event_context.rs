use std::time::{Duration, Instant};

use crate::{
    app::{CloseRequest, error::PrismaError},
    event::{Event, EventCallback, EventCallbackID, EventData, EventType},
    scene::{
        NodeID, Node,
        storage::{NodeStorage, StorageHandler},
    },
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum PropagationState {
    None,
    Bubble,
    Stopped,
}

pub(crate) struct EventTarget {
    original: NodeID,
    current: NodeID,
}

/// Provides contextual information and utilities while an event is being dispatched.
///
/// `EventContext` is passed to every event callback and grants access to the
/// current event, its target, and scene-level operations.
///
/// All structural and scene modifications requested through the context are deferred until event dispatch has finished.
///
/// # Capabilities
/// - Access the dispatched event through [`event`] or [`expect_event`].
/// - Access the original target and the current propagation target.
/// - Read or modify nodes through [`NodeView`].
/// - Create, destroy, or reorganize nodes.
/// - Register or remove scene-level event listeners.
/// - Control event propagation.
/// - Request or cancel application shutdown.
///
/// # Target
///
/// Some events participate in propagation. In these cases:
///
/// - [`target`] refers to the original event target.
/// - [`current_target`] refers to the node currently handling the event.
///
/// For scene- or window-level events, neither value may exist, therefore both methods return an `Option`.
///
/// # Example
///
/// ```ignore
/// .on_event::<MouseDown>(|ctx, event| {
///     if let Some(mut node) = ctx.current_target() {
///         node.position(
///             event.position.x as i32,
///             event.position.y as i32,
///         );
///     }
/// })
/// ```
pub struct EventContext<'a> {
    pub(crate) target: Option<EventTarget>,
    pub(crate) current_callback: Option<EventCallbackID>,
    pub(crate) event: Option<Event>,
    pub(crate) close_request: Option<CloseRequest>,

    propagation_state: PropagationState,
    cancel_close_requested: bool,
    pending_event_listeners: Vec<(EventType, EventCallback)>,
    pending_destroyed_event_listeners: Vec<EventCallbackID>,
    pending_created_nodes: Vec<NodeID>,
    pending_destroyed_nodes: Vec<NodeID>,

    nodes: &'a mut NodeStorage,
}
impl<'a> EventContext<'a> {
    pub fn new(nodes: &'a mut NodeStorage) -> Self {
        Self {
            event: None,
            target: None,
            current_callback: None,
            close_request: None,

            propagation_state: PropagationState::None,
            cancel_close_requested: false,
            pending_event_listeners: Vec::new(),
            pending_destroyed_event_listeners: Vec::new(),
            pending_created_nodes: Vec::new(),
            pending_destroyed_nodes: Vec::new(),
            nodes,
        }
    }

    pub(crate) fn expect_event<T>(&self) -> Result<T, PrismaError>
    where
        T: EventData,
    {
        let event = self.event.unwrap();
        T::cast(event).ok_or(PrismaError::UnexpectedEventType(
            T::TYPE.get_kind(),
            event.get_kind(),
        ))
    }

    pub(crate) fn set_current_target(&mut self, target: NodeID) {
        if let Some(event_target) = &mut self.target {
            event_target.original = target;
        } else {
            self.target = Some(EventTarget {
                original: target,
                current: target,
            });
        }
    }

    pub(crate) fn set_original_target(&mut self, target: NodeID) {
        if let Some(event_target) = &mut self.target {
            event_target.original = target;
        } else {
            self.target = Some(EventTarget {
                original: target,
                current: target,
            });
        }
    }

    pub fn target(&mut self) -> Node<'_> {
        self.get_node(
            self.target
                .as_ref()
                .expect("There is no original target in this event.")
                .original,
        )
        .unwrap()
    }

    pub fn current_target(&mut self) -> Node<'_> {
        self.get_node(
            self.target
                .as_ref()
                .expect("There is no current target in this event.")
                .current,
        )
        .unwrap()
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_state = PropagationState::Stopped;
    }

    pub fn bubble(&mut self) {
        self.propagation_state = PropagationState::Bubble;
    }

    pub fn current_callback(&mut self) -> EventCallbackID {
        self.current_callback.unwrap()
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        self.nodes.storage()
    }

    pub fn get_nodes(&mut self) -> Vec<NodeID> {
        self.nodes.storage().get_nodes()
    }

    pub fn scene_on_event<F>(&mut self, event_type: EventType, callback: F)
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        self.pending_event_listeners
            .push((event_type, EventCallback::new(callback)));
    }

    pub fn scene_off_event(&mut self, target: EventCallbackID) {
        self.pending_destroyed_event_listeners.push(target);
    }

    pub fn new_node(&mut self) -> Node<'_> {
        let new_node = self.nodes.new_node();
        self.pending_created_nodes.push(new_node.get_id());
        new_node
    }

    pub fn destroy(&mut self, target: NodeID) {
        self.storage()
            .state
            .get_unchecked_mut(target)
            .destruction_requested = true;
        self.pending_destroyed_nodes.push(target);
    }

    pub fn close(&mut self, timer: u64) {
        self.cancel_close_requested = false;
        self.close_request = Some(CloseRequest {
            duration: Duration::from_millis(timer),
            requested_at: Instant::now(),
        });
    }

    /// Does not work on Event::Quit
    pub fn cancel_close(&mut self) {
        self.cancel_close_requested = true;
        self.close_request = None;
    }

    pub(crate) fn is_cancel_close_requested(&self) -> bool {
        self.cancel_close_requested
    }

    pub(crate) fn propagation_state(&self) -> PropagationState {
        self.propagation_state
    }

    pub(crate) fn take_event_listeners(&mut self) -> Vec<(EventType, EventCallback)> {
        std::mem::take(&mut self.pending_event_listeners)
    }

    pub(crate) fn take_destroyed_event_listeners(&mut self) -> Vec<EventCallbackID> {
        std::mem::take(&mut self.pending_destroyed_event_listeners)
    }

    pub fn get_node(&mut self, id: NodeID) -> Result<Node<'_>, PrismaError> {
        Node::new(id, self.nodes)
    }

    pub(crate) fn take_created_nodes(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.pending_created_nodes)
    }
    pub(crate) fn take_destroyed_nodes(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.pending_destroyed_nodes)
    }
}
