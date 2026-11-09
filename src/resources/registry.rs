use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::{
    error::PrismaError,
    node::NodeID,
    resources::{Resource, ResourceID, ResourceType},
};

pub enum ResourceEntry<T: Resource> {
    External {
        id: T::ResourceID,
        resource: ResourceType,
        path: PathBuf,
        file_name: String,
        users: HashSet<NodeID>,
    },
    Internal {
        id: T::ResourceID,
        resource: ResourceType,
        users: HashSet<NodeID>,
    },
}

pub trait ResourceRegister<T: Resource> {
    fn register_internal(&mut self, resource: T) -> Result<T::ResourceID, PrismaError>;
}

pub struct ResourceRegistry<T: Resource> {
    registry: HashMap<T::ResourceID, ResourceEntry<T>>,
}

impl<T: Resource> ResourceRegistry<T>
where
    T::ResourceID: Debug,
{
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn remove_registry(&mut self, id: T::ResourceID) -> Result<ResourceEntry<T>, PrismaError> {
        self.registry
            .remove(&id)
            .ok_or(PrismaError::ResourceNotFound(
                "No registry to remove".to_string(),
            ))
    }

    pub fn register_user(
        &mut self,
        resource: T::ResourceID,
        id: NodeID,
    ) -> Result<(), PrismaError> {
        if let Some(entry) = self.registry.get_mut(&resource) {
            match entry {
                ResourceEntry::External { users, .. } => {
                    users.insert(id);
                }
                ResourceEntry::Internal { users, .. } => {
                    users.insert(id);
                }
            }
            Ok(())
        } else {
            Err(PrismaError::ResourceNotFound(format!(
                "No registry found for resource id {:?} type {:?}",
                resource,
                T::TYPE
            )))
        }
    }

    pub fn remove_user(&mut self, resource: T::ResourceID, id: NodeID) -> Result<(), PrismaError> {
        if let Some(ResourceEntry::Internal { users, .. }) = self.registry.get_mut(&resource) {
            users.remove(&id);
            Ok(())
        } else {
            Err(PrismaError::ResourceNotFound(format!(
                "No registry found for resource id {:?} type {:?}",
                resource,
                T::TYPE
            )))
        }
    }

    pub fn has_users(&self, resource: T::ResourceID) -> Result<bool, PrismaError> {
        match self.from_id(resource)? {
            ResourceEntry::External { users, .. } => Ok(!users.is_empty()),
            ResourceEntry::Internal { users, .. } => Ok(!users.is_empty()),
        }
    }

    pub fn register_external(&mut self, path: &Path) -> Result<T::ResourceID, PrismaError> {
        let id = T::ResourceID::next();
        let registry = ResourceEntry::<T>::External {
            id,
            resource: T::TYPE,
            path: path.to_path_buf(),
            file_name: path
                .file_name()
                .ok_or(PrismaError::ResourceNotFound(
                    "Failed to create resource registry".to_string(),
                ))?
                .to_string_lossy()
                .to_string(),
            users: HashSet::new(),
        };
        self.registry.insert(id, registry);
        Ok(id)
    }

    pub fn register_internal(&mut self) -> T::ResourceID {
        let id = T::ResourceID::next();
        let registry = ResourceEntry::<T>::Internal {
            id,
            resource: T::TYPE,
            users: HashSet::new(),
        };
        self.registry.insert(id, registry);
        id
    }

    pub fn from_id(&self, id: T::ResourceID) -> Result<&ResourceEntry<T>, PrismaError> {
        self.registry
            .get(&id)
            .ok_or(PrismaError::ResourceNotFound(format!(
                "Registry not found for id: {:?} {:?}",
                T::TYPE,
                id
            )))
    }

    pub fn from_path(&self, path: &Path) -> Result<&ResourceEntry<T>, PrismaError> {
        self.registry
            .values()
            .find_map(|entry| match entry {
                ResourceEntry::External {
                    path: entry_path, ..
                } => {
                    if entry_path == path {
                        Some(entry)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .ok_or(PrismaError::ResourceNotFound(format!(
                "Registry not found for path: {path:?}"
            )))
    }
}
