use std::any::{Any, TypeId};

use hashbrown::HashMap;
use uuid::Uuid;

use crate::anewthing::channel::Channel;

use super::{
    archetype::Archetype,
    component::{Component, ComponentSet},
    error::Error,
};

pub struct EntityManager {
    channel: Channel,
    entities: HashMap<Uuid, Entity>,
    chunks: HashMap<Archetype, Chunk>,
    shared_components: HashMap<TypeId, Box<dyn Any>>,
}

impl EntityManager {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
            entities: HashMap::new(),
            chunks: HashMap::new(),
            shared_components: HashMap::new(),
        }
    }

    fn get_or_create_chunk<'a: 'b, 'b>(&'a mut self, archetype: Archetype) -> &'b mut Chunk {
        self.chunks.entry(archetype).or_insert_with(|| Chunk {
            entity_ids: Vec::new(),
            components: Vec::new(),
        })
    }

    unsafe fn swap_and_remove_entity(&mut self, id: &Uuid) -> ComponentSet {
        let Entity {
            archetype,
            chunk_index,
            ..
        } = self.entities.get(id).unwrap();
        let chunk_index = *chunk_index;
        let chunk_size = archetype.len();
        let chunk = self.chunks.get_mut(archetype).unwrap();

        let components = if chunk.components.len() == chunk_size {
            chunk.entity_ids.clear();
            chunk.components.drain(..).collect::<Vec<_>>()
        } else {
            // swaps components and removes last components
            let from_components_index = chunk_index * chunk_size;
            let swap_components_index = chunk.components.len() - chunk_size;
            for i in 0..chunk_size {
                chunk
                    .components
                    .swap(from_components_index + i, swap_components_index + i);
            }
            let components = chunk
                .components
                .drain(swap_components_index..)
                .collect::<Vec<_>>();

            // swaps entity id and remove last one
            let swap_entity_id = *chunk.entity_ids.last().unwrap();
            chunk
                .entity_ids
                .swap(chunk_index, chunk.components.len() - 1);
            chunk.entity_ids.truncate(chunk.entity_ids.len() - 1);

            // updates swap entity
            self.entities
                .get_mut(&swap_entity_id)
                .unwrap()
                .set_chunk_index(chunk_index);

            components
        };

        ComponentSet(components)
    }

    pub fn has_entity(&self, id: &Uuid) -> bool {
        self.entities.contains_key(id)
    }

    pub fn create_entity(&mut self, components: ComponentSet) -> Result<Uuid, Error> {
        let archetype = components.archetype();
        let size = archetype.len();
        if size == 0 {
            return Err(Error::EmptyComponents);
        }

        let chunk = self.get_or_create_chunk(archetype.clone());
        let total = chunk.components.len();
        let chunk_index = total / size;
        chunk.components.extend(components.0);
        let entity = Entity {
            version: 0,
            archetype,
            chunk_index,
        };
        let (id, _) = self
            .entities
            .insert_unique_unchecked(Uuid::new_v4(), entity);

        self.channel.send(CreateEntity::new(*id));

        Ok(*id)
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Result<ComponentSet, Error> {
        if !self.has_entity(&id) {
            return Err(Error::NoSuchEntity);
        }

        self.channel.send(RemoveEntity::new(*id));

        unsafe { Ok(self.swap_and_remove_entity(&id)) }
    }

    unsafe fn set_components(&mut self, id: &Uuid, components: ComponentSet) {
        let archetype = components.archetype();
        let chunk = self.get_or_create_chunk(archetype.clone());
        chunk.components.extend(components.0);
        chunk.entity_ids.push(*id);
        let chunk_index = chunk.entity_ids.len() - 1;
        self.entities
            .get_mut(id)
            .unwrap()
            .set_archetype(archetype, chunk_index);
    }

    pub fn has_component<C: Component + 'static>(&mut self, id: &Uuid) -> bool {
        let Some(entity) = self.entities.get(id) else {
            return false;
        };
        entity.archetype.has_component::<C>()
    }

    pub fn add_component<C>(&mut self, id: Uuid, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        if !self.has_entity(&id) {
            return Err(Error::NoSuchEntity);
        }
        if self.has_component::<C>(&id) {
            return Err(Error::DuplicateComponent);
        }

        let components = unsafe { self.swap_and_remove_entity(&id) };
        let components = unsafe { components.add_unique_unchecked(component) };
        unsafe { self.set_components(&id, components) };

        self.channel.send(AddComponent::new::<C>(id));

        Ok(())
    }

    pub fn remove_component<C>(&mut self, id: Uuid) -> Result<C, Error>
    where
        C: Component + 'static,
    {
        if !self.has_entity(&id) {
            return Err(Error::NoSuchEntity);
        }
        if !self.has_component::<C>(&id) {
            return Err(Error::NoSuchComponent);
        }

        let components = unsafe { self.swap_and_remove_entity(&id) };
        let (components, removed) = unsafe { components.remove_unchecked::<C>() };
        unsafe { self.set_components(&id, components) };

        self.channel.send(RemoveComponent::new::<C>(id));

        Ok(removed)
    }

    pub fn has_shared_component<C>(&self) -> bool
    where
        C: Component + 'static,
    {
        self.shared_components.contains_key(&TypeId::of::<C>())
    }

    pub fn add_shared_component<C>(&mut self, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        if self.has_shared_component::<C>() {
            return Err(Error::DuplicateComponent);
        }

        self.shared_components
            .insert_unique_unchecked(TypeId::of::<C>(), Box::new(component));

        Ok(())
    }

    pub fn remove_shared_component<C>(&mut self) -> Result<C, Error>
    where
        C: Component + 'static,
    {
        if !self.has_shared_component::<C>() {
            return Err(Error::NoSuchComponent);
        }

        let removed = *self
            .shared_components
            .remove(&TypeId::of::<C>())
            .unwrap()
            .downcast::<C>()
            .unwrap();

        Ok(removed)
    }

    // pub fn query(&mut self) {
    //     let archetypes = self.chunks.keys().filter(|ar| )
    // }
}

struct Entity {
    version: usize,
    archetype: Archetype,
    chunk_index: usize,
}

impl Entity {
    fn set_archetype(&mut self, archetype: Archetype, chunk_index: usize) {
        self.archetype = archetype;
        self.chunk_index = chunk_index;
        self.version = self.version.wrapping_add(1);
    }

    fn set_chunk_index(&mut self, chunk_index: usize) {
        self.chunk_index = chunk_index;
        self.version = self.version.wrapping_add(1);
    }
}

struct Chunk {
    entity_ids: Vec<Uuid>,
    components: Vec<(TypeId, Box<dyn Any>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateEntity(Uuid);

impl CreateEntity {
    fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn entity_id(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveEntity(Uuid);

impl RemoveEntity {
    fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn entity_id(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddComponent(Uuid, TypeId);

impl AddComponent {
    fn new<C>(id: Uuid) -> Self
    where
        C: Component + 'static,
    {
        Self(id, TypeId::of::<C>())
    }

    pub fn entity_id(&self) -> &Uuid {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveComponent(Uuid, TypeId);

impl RemoveComponent {
    fn new<C>(entity: Uuid) -> Self
    where
        C: Component + 'static,
    {
        Self(entity, TypeId::of::<C>())
    }

    pub fn entity(&self) -> &Uuid {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}
