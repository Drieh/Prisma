use crate::{node::NodeID, scene::document::element::ElementID};

pub enum ElementType {
    Title,
    Text,
    Span,
    Custom,
}

pub struct Element {
    id: ElementID,
    root: Option<NodeID>,
    children: Vec<ElementID>,
}

impl Element {
    pub fn new(id: ElementID) -> Self {
        Self {
            id,
            root: None,
            children: Vec::new(),
        }
    }
}
