use std::{
    any::{Any, TypeId},
    borrow::Cow,
};

use hashbrown::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKey {
    Usize(usize),
    Uuid(Uuid),
    String(Cow<'static, str>),
}

pub struct Resources(
    HashMap<ResourceKey, Box<dyn Any>>,
    HashMap<TypeId, Box<dyn Any>>,
);

impl Resources {
    pub fn new() -> Self {
        Self(HashMap::new(), HashMap::new())
    }
}

impl Resources {
    pub fn clear(&mut self) {
        self.0.clear();
        self.1.clear();
    }

    pub fn add_resource<T>(&mut self, key: ResourceKey, resource: T) -> Option<Box<dyn Any>>
    where
        T: Any,
    {
        self.0.insert(key, Box::new(resource))
    }

    pub fn remove_resource<T>(&mut self, key: &ResourceKey) -> Option<Box<dyn Any>> {
        self.0.remove(key)
    }

    pub fn resource(&self, key: &ResourceKey) -> Option<&Box<dyn Any>> {
        self.0.get(key)
    }

    pub fn resource_mut(&mut self, key: &ResourceKey) -> Option<&mut Box<dyn Any>> {
        self.0.get_mut(key)
    }

    pub fn add_resource_concrete<T: Any>(&mut self, resource: T) -> Option<T> {
        match self.1.insert(TypeId::of::<T>(), Box::new(resource)) {
            Some(resource) => Some(*resource.downcast::<T>().unwrap()),
            None => None,
        }
    }

    pub fn remove_resource_concrete<T: Any>(&mut self) -> Option<T> {
        match self.1.remove(&TypeId::of::<T>()) {
            Some(resource) => Some(*resource.downcast::<T>().unwrap()),
            None => None,
        }
    }

    pub fn resource_concrete<T: Any>(&self) -> Option<&T> {
        match self.1.get(&TypeId::of::<T>()) {
            Some(resource) => Some(resource.downcast_ref::<T>().unwrap()),
            None => None,
        }
    }

    pub fn resource_concrete_mut<T: Any>(&mut self) -> Option<&mut T> {
        match self.1.get_mut(&TypeId::of::<T>()) {
            Some(resource) => Some(resource.downcast_mut::<T>().unwrap()),
            None => None,
        }
    }
}
