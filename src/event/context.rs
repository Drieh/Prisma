use std::time::{Duration, Instant};

use crate::{
    app::PrismaError,
    event::{Event, EventManager, EventType, managers::event_manager::CallbackID},
    scene::{NodeID, NodeStorage, NodeView, storage::StorageHandler},
};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct CloseRequest {
    pub duration: std::time::Duration,
    pub requested_at: std::time::Instant,
}

pub enum ContextAction {
    Create {
        target: NodeID,
    },
    Destroy {
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
        callback: Box<dyn FnMut(&mut EventContext) + 'static>,
    },
    RemoveSceneEventListener {
        target: CallbackID,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum PropagationState {
    None,
    Bubble,
    Stopped,
}

pub struct EventContext<'a> {
    pub(crate) target: Option<NodeID>,
    pub(crate) current_target: Option<NodeID>,
    pub(crate) current_callback: Option<CallbackID>,
    pub(crate) event: Option<Event>,
    pub(crate) close_request: Option<CloseRequest>,

    propagation_state: PropagationState,
    action_queue: Vec<ContextAction>,
    cancel_close_requested: bool,
    pending_created_nodes: Vec<NodeID>,
    pending_destroyed_nodes: Vec<NodeID>,

    nodes: &'a mut NodeStorage,
}
impl<'a> EventContext<'a> {
    pub fn new(nodes: &'a mut NodeStorage) -> Self {
        Self {
            event: None,
            target: None,
            current_target: None,
            current_callback: None,
            close_request: None,

            propagation_state: PropagationState::None,
            action_queue: Vec::new(),
            pending_created_nodes: Vec::new(),
            pending_destroyed_nodes: Vec::new(),
            cancel_close_requested: false,
            nodes,
        }
    }

    pub fn event(&self) -> Event {
        self.event.unwrap()
    }

    pub fn og_target(&mut self) -> Option<NodeView<'_>> {
        if let Some(target) = self.target {
            Some(self.get_node(target).unwrap())
        } else {
            None
        }
    }
    pub fn target(&mut self) -> Option<NodeView<'_>> {
        if let Some(target) = self.current_target {
            Some(self.get_node(target).unwrap())
        } else {
            None
        }
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_state = PropagationState::Stopped;
    }

    pub fn bubble(&mut self) {
        self.propagation_state = PropagationState::Bubble;
    }

    pub fn current_callback(&mut self) -> CallbackID {
        self.current_callback.unwrap()
    }

    pub fn storage(&mut self) -> StorageHandler<'_> {
        self.nodes.storage()
    }

    pub fn get_nodes(&mut self) -> Vec<NodeID> {
        self.nodes.storage().get_nodes()
    }

    pub fn on_scene<F>(&mut self, event_type: EventType, callback: F)
    where
        F: FnMut(&mut EventContext) + 'static,
    {
        self.action_queue
            .push(ContextAction::AddSceneEventListener {
                event_type,
                callback: Box::new(callback),
            });
    }
    pub fn off_scene(&mut self, target: CallbackID) {
        self.action_queue
            .push(ContextAction::RemoveSceneEventListener { target });
    }

    pub fn new_node(&mut self) -> NodeView<'_> {
        let new_node = self.nodes.new_node();
        self.action_queue.push(ContextAction::Create {
            target: new_node.get_id(),
        });
        new_node
    }

    pub fn destroy(&mut self, target: NodeID) {
        self.storage()
            .state
            .get_unchecked_mut(target)
            .destruction_requested = true;
        self.action_queue.push(ContextAction::Destroy { target });
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
        });
    }

    /// Doesn't work on Event::Quit
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
    pub(crate) fn take_actions(&mut self) -> Vec<ContextAction> {
        std::mem::take(&mut self.action_queue)
    }

    pub fn get_node(&mut self, id: NodeID) -> Result<NodeView<'_>, PrismaError> {
        NodeView::new(id, self.nodes)
    }

    pub(crate) fn take_created_nodes(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.pending_created_nodes)
    }
    pub(crate) fn take_destroyed_nodes(&mut self) -> Vec<NodeID> {
        std::mem::take(&mut self.pending_destroyed_nodes)
    }

    pub(crate) fn process_context_actions(
        &mut self,
        event_manager: &mut EventManager,
    ) -> Result<(), PrismaError> {
        let context_actions = self.take_actions();

        for action in context_actions {
            match action {
                ContextAction::Create { target } => {
                    self.pending_created_nodes.push(target);
                }
                ContextAction::Destroy { target } => {
                    self.pending_destroyed_nodes.push(target);
                }
                ContextAction::AddChild { parent, child } => {
                    self.get_node(parent)?.add_child(child)?;
                }
                ContextAction::RemoveChild { parent, child } => {
                    self.get_node(parent)?.remove_child(child)?;
                }
                ContextAction::AddSceneEventListener {
                    event_type,
                    callback,
                } => {
                    event_manager.add_scene_event_listener(event_type, callback);
                }
                ContextAction::RemoveSceneEventListener { target } => {
                    event_manager.remove_event_listener(target);
                }
            }
        }
        Ok(())
    }
}
