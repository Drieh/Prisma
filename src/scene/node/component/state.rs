use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{error::PrismaError, node::style_view::StyleCallback};

pub struct TypeMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}
impl<'a> TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set<T: 'static>(&'a mut self, value: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&'a self) -> Option<&'a T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&'a mut self) -> Option<&'a mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    pub fn remove<T: 'static>(&'a mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn contains<T: 'static>(&'a self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }
}

pub struct NodeState {
    pub destruction_requested: bool,
    pub(crate) on_hover: Option<Box<StyleCallback>>,
    pub(crate) on_active: Option<Box<StyleCallback>>,

    user_type_map: TypeMap,
}
impl NodeState {
    pub fn new() -> Self {
        Self {
            user_type_map: TypeMap::new(),
            on_active: None,
            on_hover: None,
            destruction_requested: false,
        }
    }
    pub fn set<T: Any>(&mut self, value: T) {
        self.user_type_map.set::<T>(value);
    }

    pub fn get<T: Any>(&self) -> Result<&T, PrismaError> {
        let name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap()
            .to_string();
        self.user_type_map
            .get::<T>()
            .ok_or(PrismaError::NodeStateNotFound(name))
    }

    pub fn get_mut<T: Any>(&mut self) -> Result<&mut T, PrismaError> {
        let name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap()
            .to_string();
        self.user_type_map
            .get_mut::<T>()
            .ok_or(PrismaError::NodeStateNotFound(name))
    }

    pub fn remove<T: Any>(&mut self) -> Result<T, PrismaError> {
        let name = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap()
            .to_string();
        self.user_type_map
            .remove::<T>()
            .ok_or(PrismaError::NodeStateNotFound(name))
    }

    pub fn contains<T: Any>(&self) -> bool {
        self.user_type_map.contains::<T>()
    }
}
