use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    hash::Hash,
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
struct FontMap {
    storage: HashMap<u16, Font>,
}

impl Default for FontMap {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
}

pub(crate) struct FontManager {
    storage: HashMap<FontID, FontMap>,
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

    fn open_font(&mut self, id: FontID, size: u16) -> Result<&Font, PrismaError> {
        // returns a font if it exists
        let exists = self
            .storage
            .get(&id)
            .is_some_and(|font_map| font_map.storage.contains_key(&size));

        if !exists {
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

                            return Err(PrismaError::resource_not_found(format!(
                                "Font Manager: Font not found, SDL3 error: {error:?}"
                            )));
                        }
                        println!("opened font {} id: {id:?} size: {size}", file_name);

                        self.storage
                            .entry(id)
                            .or_default()
                            .storage
                            .insert(size, Font::new(raw, size, file_name.to_owned()));
                    }
                }
                ResourceEntry::Internal { .. } => {
                    unreachable!("A font cannot be an internal resource")
                }
            }
        }
        self.storage
            .get(&id)
            .ok_or(PrismaError::resource_not_found(
                "Could not find the desired font.",
            ))
            .and_then(|map| {
                map.storage
                    .get(&size)
                    .ok_or(PrismaError::resource_not_found("asd"))
            })

        // else open a new font
    }

    pub fn clear(&mut self) {
        self.storage.drain();
    }

    pub fn get(&mut self, id: FontID, size: u16) -> Result<&Font, PrismaError> {
        self.open_font(id, size)
    }

    pub fn register_user(&mut self, resource: FontID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.register_user(resource, id)
    }

    pub fn remove_user(&mut self, resource: FontID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.remove_user(resource, id)?;
        if let ResourceEntry::Internal { users, .. } = self.registry.from_id(resource)?
            && users.is_empty()
        {
            self.destroy(resource)?;
        }
        Ok(())
    }

    pub fn contains(&self, id: FontID) -> bool {
        self.storage
            .get(&id)
            .is_some_and(|map| !map.storage.is_empty())
    }

    pub fn destroy(&mut self, id: FontID) -> Result<(), PrismaError> {
        self.storage
            .remove(&id)
            .and(Some(()))
            .ok_or(PrismaError::resource_not_found(format!(
                "Font not found for id {id:?}"
            )))
    }
}
