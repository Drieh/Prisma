use crate::event::{Event, EventType};

pub trait EventData: Sized {
    const TYPE: EventType;

    fn cast(event: Event) -> Option<Self>;
}
