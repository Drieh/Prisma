use crate::{error::PrismaError, resources::font::FontManager};
pub struct ResourceManager {
    font_manager: FontManager,
}
impl ResourceManager {
    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
        }
    }
    pub fn load_resources(&mut self) -> Result<(), PrismaError> {
        self.font_manager.load_fonts()?;
        Ok(())
    }
}
