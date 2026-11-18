use std::collections::HashMap;

use crate::scene::document::element::{Element, ElementID};

pub struct Document {
    elements: HashMap<ElementID, Element>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    pub fn new_element(&mut self) -> &mut Element {
        let id = ElementID::next();
        self.elements.insert(id, Element::new(id));
        self.elements.get_mut(&id).unwrap()
    }
}
