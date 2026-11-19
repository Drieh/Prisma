use std::path::Path;

use crate::{
    error::PrismaError,
    node::NodeID,
    resources::{
        Font, FontID, FontOptions, PrismaTexture, Resource, ResourceGetter, ResourceID,
        ResourceType, TextureID, TextureOptions,
        manager::{FontManager, TextureManager},
        registry::ResourceRegister,
    },
};
pub struct ResourceManager {
    pub(crate) font_manager: FontManager,
    pub(crate) texture_manager: TextureManager,
}
impl ResourceManager {
    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
            texture_manager: TextureManager::new(),
        }
    }

    pub fn register_external<T>(&mut self, path: &Path) -> Result<T::ResourceID, PrismaError>
    where
        T: Resource,
    {
        match T::TYPE {
            ResourceType::Font => self
                .font_manager
                .register_external(path)
                .and_then(|id| Ok(T::ResourceID::cast(id))),
            ResourceType::PrismaTexture => self
                .texture_manager
                .register_external(path)
                .and_then(|id| Ok(T::ResourceID::cast(id))),
        }
    }

    pub fn register_internal<T>(&mut self, resource: T) -> Result<T::ResourceID, PrismaError>
    where
        T: Resource,
        Self: ResourceRegister<T>,
    {
        let resource_id = <Self as ResourceRegister<T>>::register_internal(self, resource)?;
        Ok(resource_id)
    }

    pub fn add_user<T>(&mut self, resource: T, id: NodeID) -> Result<(), PrismaError>
    where
        T: ResourceID,
    {
        match T::Resource::TYPE {
            ResourceType::Font => self.font_manager.register_user(FontID::cast(resource), id),
            ResourceType::PrismaTexture => self
                .texture_manager
                .register_user(TextureID::cast(resource), id),
        }
    }

    pub fn remove_user<T>(&mut self, resource: T, id: NodeID) -> Result<(), PrismaError>
    where
        T: ResourceID,
    {
        match T::Resource::TYPE {
            ResourceType::Font => self.font_manager.remove_user(FontID::cast(resource), id),
            ResourceType::PrismaTexture => self
                .texture_manager
                .remove_user(TextureID::cast(resource), id),
        }
    }

    pub fn contains<T>(&self, id: T) -> bool
    where
        T: ResourceID,
    {
        match T::Resource::TYPE {
            ResourceType::Font => self.font_manager.contains(FontID::cast(id)),
            ResourceType::PrismaTexture => self.texture_manager.contains(TextureID::cast(id)),
        }
    }

    pub fn destroy<T>(&mut self, id: T) -> Result<(), PrismaError>
    where
        T: ResourceID,
        Self: ResourceGetter<T::Resource>,
    {
        <Self as ResourceGetter<T::Resource>>::destroy(self, id)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.font_manager.clear();
        self.texture_manager.clear();
    }

    pub fn get_resource<T>(
        &mut self,
        id: T,
        options: <T::Resource as Resource>::GetOptions,
    ) -> Result<&T::Resource, PrismaError>
    where
        T: ResourceID,
        Self: ResourceGetter<T::Resource>,
    {
        <Self as ResourceGetter<T::Resource>>::get(self, id, options)
    }
}

// FONT

impl ResourceGetter<Font> for ResourceManager {
    fn get(&mut self, id: FontID, options: FontOptions) -> Result<&Font, PrismaError> {
        self.font_manager.get(id, options.size)
    }
    fn destroy(&mut self, id: FontID) -> Result<(), PrismaError> {
        self.font_manager.destroy(id)?;
        Ok(())
    }
}

impl ResourceRegister<Font> for ResourceManager {
    fn register_internal(&mut self, _resource: Font) -> Result<FontID, PrismaError> {
        todo!("yet cannot register an internal font")
    }
}

// TEXTURE

impl ResourceGetter<PrismaTexture> for ResourceManager {
    fn get(
        &mut self,
        id: TextureID,
        _options: TextureOptions,
    ) -> Result<&PrismaTexture, PrismaError> {
        self.texture_manager.get(id)
    }

    fn destroy(&mut self, id: <PrismaTexture as Resource>::ResourceID) -> Result<(), PrismaError> {
        self.texture_manager.remove(id)?;
        Ok(())
    }
}

impl ResourceRegister<PrismaTexture> for ResourceManager {
    fn register_internal(&mut self, resource: PrismaTexture) -> Result<TextureID, PrismaError> {
        self.texture_manager.register_internal(resource)
    }
}
