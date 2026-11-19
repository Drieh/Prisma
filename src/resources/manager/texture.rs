use std::{
    collections::HashMap,
    path::Path,
};

use crate::{
    error::PrismaError,
    node::NodeID,
    resources::{
        PrismaTexture, ResourceEntry, TextureID, registry::ResourceRegistry,
    },
};

pub(crate) struct TextureManager {
    storage: HashMap<TextureID, PrismaTexture>,
    registry: ResourceRegistry<PrismaTexture>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
            registry: ResourceRegistry::new(),
        }
    }

    pub fn clear(&mut self) {
        self.storage.drain();
    }

    pub fn register_user(&mut self, resource: TextureID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.register_user(resource, id)
    }

    pub fn remove_user(&mut self, resource: TextureID, id: NodeID) -> Result<(), PrismaError> {
        self.registry.remove_user(resource, id)?;
        if let ResourceEntry::Internal { users, .. } = self.registry.from_id(resource)?
            && users.is_empty()
        {
            self.remove(resource)?;
        }
        Ok(())
    }

    pub fn register_external(&mut self, path: &Path) -> Result<TextureID, PrismaError> {
        self.registry.register_external(path)
    }

    pub fn register_internal(&mut self, texture: PrismaTexture) -> Result<TextureID, PrismaError> {
        let id = self.registry.register_internal();
        self.storage.insert(id, texture);
        Ok(id)
    }

    pub fn get(&mut self, id: TextureID) -> Result<&PrismaTexture, PrismaError> {
        self.storage
            .get(&id)
            .ok_or(PrismaError::resource_not_found(format!(
                "Texture Manager: {id:?} not found"
            )))
    }

    pub fn contains(&self, id: TextureID) -> bool {
        self.storage.contains_key(&id)
    }

    pub fn remove(&mut self, id: TextureID) -> Result<PrismaTexture, PrismaError> {
        self.registry.remove_registry(id)?;
        self.storage
            .remove(&id)
            .ok_or(PrismaError::ResourceNotFound(format!(
                "Texture Manager: {id:?} not found"
            )))
    }
}
