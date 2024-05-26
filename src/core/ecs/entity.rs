use std::any::TypeId;

use hashbrown::HashMap;
use uuid::Uuid;

use super::{archetype::Archetype, component::Component};

pub struct Entity {
    pub(super) id: Uuid,
    pub(super) components: HashMap<TypeId, Box<dyn Component>>,
}

impl Entity {
    pub(super) fn new(components: HashMap<TypeId, Box<dyn Component>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            components,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn archetype(&self) -> Archetype {
        Archetype::new(self.components.keys().cloned())
    }

    pub fn component_len(&self) -> usize {
        self.components.len()
    }

    pub fn component<T>(&self) -> Option<&T>
    where
        T: Component + 'static,
    {
        match self.components.get(&T::component_type()) {
            Some(component) => Some(component.as_any().downcast_ref::<T>().unwrap()),
            None => None,
        }
    }

    pub fn component_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Component + 'static,
    {
        match self.components.get_mut(&T::component_type()) {
            Some(component) => Some(component.as_any_mut().downcast_mut::<T>().unwrap()),
            None => None,
        }
    }
}
