#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum LifecycleEventType {
    Update,
    Creation,
    Destruction,
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum LifecycleEvent {
    Update,
    Creation { target: usize },
    Destruction { target: usize },
}
impl LifecycleEvent {
    pub fn event_type(&self) -> LifecycleEventType {
        match self {
            LifecycleEvent::Creation { .. } => LifecycleEventType::Creation,
            LifecycleEvent::Update => LifecycleEventType::Update,
            LifecycleEvent::Destruction { .. } => LifecycleEventType::Destruction,
        }
    }
}

pub struct LifecycleManager {
    queue: Vec<LifecycleEvent>,
}
impl LifecycleManager {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn handle_creation(&mut self, target: usize) {
        self.queue.push(LifecycleEvent::Creation { target });
    }

    pub fn handle_update(&mut self) {
        self.queue.push(LifecycleEvent::Update);
    }

    pub fn handle_destruction(&mut self, target: usize) {
        self.queue.push(LifecycleEvent::Destruction { target });
    }

    pub fn take_events(&mut self) -> Vec<LifecycleEvent> {
        std::mem::take(&mut self.queue)
    }
}
