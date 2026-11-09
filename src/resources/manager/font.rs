use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    path::Path,
};

use sdl3::sys::error::SDL_GetError;
use sdl3_ttf_sys::ttf::TTF_OpenFont;

use crate::{
    Scene,
    error::PrismaError,
    node::NodeID,
    resources::{Font, FontID, ResourceEntry, registry::ResourceRegistry},
};

pub(crate) struct FontManager {
    storage: HashMap<FontID, Font>,
    registry: ResourceRegistry<Font>,
}
impl FontManager {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            registry: ResourceRegistry::new(),
        }
    }

    pub fn register_external(&mut self, path: &Path) -> Result<FontID, PrismaError> {
        self.registry.register_external(path)
    }

    fn open_font(&mut self, id: FontID, size: u16) -> Result<&mut Font, PrismaError> {
        let entry = self.registry.from_id(id)?;
        match entry {
            ResourceEntry::External {
                path, file_name, ..
            } => {
                // font is always an external resource, so .unwrap is ok
                let path = CString::new(path.to_string_lossy().as_bytes()).unwrap();
                unsafe {
                    let raw = TTF_OpenFont(path.as_ptr(), size as f32);

                    if raw.is_null() {
                        let error = CStr::from_ptr(SDL_GetError());

                        println!("error SDL: {}", error.to_string_lossy());
                        return Err(PrismaError::ResourceNotFound(
                            "Font Manager: Font not found".to_string(),
                        ));
                    }
                    self.storage
                        .insert(id, Font::new(raw, size, file_name.to_owned()));

                    println!("opened font {} id: {id:?}", file_name.to_owned());
                }
                self.storage
                    .get_mut(&id)
                    .ok_or(PrismaError::ResourceNotFound(
                        "Font Manager: failed to load font.".to_string(),
                    ))
            }
            ResourceEntry::Internal { .. } => {
                unreachable!("A font cannot be an internal resource")
            }
        }
    }

    pub fn clear(&mut self) {
        self.storage.drain();
    }

    pub fn get(&mut self, id: FontID, size: u16) -> Result<&mut Font, PrismaError> {
        if self.storage.contains_key(&id) {
            return Ok(self.storage.get_mut(&id).unwrap());
        }

        if self.registry.from_id(id).is_ok() {
            self.open_font(id, size)
        } else {
            Err(PrismaError::ResourceNotFound(
                "There is no registry for the given font ID".to_string(),
            ))
        }
    }

    pub fn take(&mut self, id: FontID) -> Result<Font, PrismaError> {
        if self.storage.contains_key(&id) {
            return Ok(self.storage.remove(&id).unwrap());
        }

        if self.registry.from_id(id).is_err() {
            return Err(PrismaError::ResourceNotFound(
                "Font Manager: there is no registry for the given font ID".to_string(),
            ));
        }

        Err(PrismaError::ResourceNotFound(
            "Font Manager: font resource not found".to_string(),
        ))
    }

    pub fn register_user(&mut self, resource: FontID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.register_user(resource, id)
    }

    pub fn remove_user(&mut self, resource: FontID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.remove_user(resource, id)?;
        if let ResourceEntry::Internal { users, .. } = self.registry.from_id(resource)?
            && users.is_empty()
        {
            self.remove(resource)?;
        }
        Ok(())
    }

    pub fn gb_collector(&mut self) {}

    pub fn contains(&self, id: FontID) -> bool {
        self.storage.contains_key(&id)
    }

    pub fn remove(&mut self, id: FontID) -> Result<Font, PrismaError> {
        self.storage
            .remove(&id)
            .ok_or(PrismaError::ResourceNotFound(
                "Font Manager: font resource not found".to_string(),
            ))
    }
}
