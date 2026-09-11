use std::collections::HashMap;

use crate::{
    error::PrismaError,
    resources::resource::{Font, FontID},
};

pub struct FontManager {
    font: HashMap<FontID, Font>,
}
impl FontManager {
    pub fn new() -> Self {
        Self {
            font: HashMap::new(),
        }
    }

    pub fn load_fonts(&mut self) -> Result<(), PrismaError> {
        // cargar
        Ok(())
    }

    pub fn get_font(&self, id: FontID) -> Result<&Font, PrismaError> {
        self.font
            .get(&id)
            .ok_or(PrismaError::ResourceNotFound(format!(
                "Font with ID {} not found",
                id
            )))
    }
}
