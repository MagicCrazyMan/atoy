use std::any::TypeId;

use hashbrown::HashMap;
use smallvec::SmallVec;
use uuid::Uuid;

use super::{
    archetype::{Archetype, ToArchetype},
    component::Component,
};

pub struct Entity {
    id: Uuid,
    components: HashMap<TypeId, Box<dyn Component>>,
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
        let mut compnent_types: SmallVec<[TypeId; 3]> = self.components.keys().cloned().collect();
        compnent_types.sort();
        Archetype::from_vec_unchecked(compnent_types)
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

impl ToArchetype for Entity {
    #[inline]
    fn to_archetype(&self) -> Archetype {
        self.archetype()
    }
}
