use std::{any::TypeId, rc::Rc};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::core::{channel::Sender, Rrc};

use super::{
    archetype::{Archetype, Chunk},
    component::Component,
    manager::EntityManager,
    message::Message,
};

pub struct Entity {
    pub(super) id: Uuid,
    sender: Sender<Message>,
    components: HashMap<TypeId, Box<dyn Component>>,

    chunks: Rrc<HashMap<Archetype, Chunk>>,
    entities: Rrc<HashMap<Uuid, Rrc<Self>>>,
}

impl Entity {
    pub(super) fn new(
        manager: &EntityManager,
        components: HashMap<TypeId, Box<dyn Component>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: manager.sender.clone(),
            components,

            chunks: Rc::clone(&manager.chunks),
            entities: Rc::clone(&manager.entities),
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

    pub fn component_unchecked<T>(&self) -> &T
    where
        T: Component + 'static,
    {
        self.component::<T>().unwrap()
    }

    pub fn component_mut_unchecked<T>(&mut self) -> &mut T
    where
        T: Component + 'static,
    {
        self.component_mut::<T>().unwrap()
    }

    pub fn has_component<T>(&self) -> bool
    where
        T: Component + 'static,
    {
        self.components.contains_key(&T::component_type())
    }

    pub fn add_component<T>(&mut self, component: T) -> Result<(), T>
    where
        T: Component + 'static,
    {
        if self.components.contains_key(&T::component_type()) {
            return Err(component);
        };

        let old_archetype = self.archetype();
        self.components
            .insert_unique_unchecked(T::component_type(), Box::new(component));
        let new_archetype = self.archetype();

        self.swap_archetype(&old_archetype, &new_archetype);
        self.sender.send(Message::AddComponent {
            entity_id: self.id,
            old_archetype,
            new_archetype,
        });

        Ok(())
    }

    pub fn remove_component<T>(&mut self)
    where
        T: Component + 'static,
    {
        if self.components.contains_key(&T::component_type()) {
            return;
        };

        let old_archetype = self.archetype();
        self.components.remove(&T::component_type()).unwrap();
        let new_archetype = self.archetype();

        self.swap_archetype(&old_archetype, &new_archetype);
        self.sender.send(Message::RemoveComponent {
            entity_id: self.id,
            old_archetype,
            new_archetype,
        });
    }

    fn swap_archetype(&self, old_archetype: &Archetype, new_archetype: &Archetype) {
        if old_archetype == new_archetype {
            return;
        }

        let mut chunks = self.chunks.borrow_mut();

        let entity = chunks
            .get_mut(old_archetype)
            .unwrap()
            .remove_entity(&self.id)
            .unwrap();

        match chunks.entry(new_archetype.clone()) {
            Entry::Occupied(mut o) => {
                o.get_mut().add_entity_unchecked(entity);
            }
            Entry::Vacant(v) => {
                let mut chunk = Chunk::new(self.sender.clone());
                chunk.add_entity_unchecked(entity);
                v.insert(chunk);
            }
        };
    }

    pub fn remove(self) {
        self.chunks
            .borrow_mut()
            .get_mut(&self.archetype())
            .unwrap()
            .remove_entity(&self.id);
        self.entities.borrow_mut().remove(&self.id);
    }
}
