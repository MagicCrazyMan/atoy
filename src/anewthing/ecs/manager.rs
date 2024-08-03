use std::any::Any;

use hashbrown::HashMap;

use crate::anewthing::channel::Channel;

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, ComponentSet, SharedComponentKey},
    entity::EntityKey,
    error::Error,
    iter::{EntityComponentsIter, EntityComponentsIterMut},
};

pub struct EntityManager {
    channel: Channel,
    entities: HashMap<EntityKey, EntityItem>,
    pub(super) chunks: HashMap<Archetype, ChunkItem>,
    pub(super) shared_components: HashMap<SharedComponentKey, SharedComponentItem>,
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

    fn get_or_create_chunk<'a: 'b, 'b>(&'a mut self, archetype: Archetype) -> &'b mut ChunkItem {
        self.chunks.entry(archetype).or_insert_with(|| ChunkItem {
            entity_keys: Vec::new(),
            components: Vec::new(),
        })
    }

    unsafe fn swap_and_remove_entity(&mut self, key: &EntityKey) -> ComponentSet {
        let EntityItem {
            archetype,
            chunk_index,
            ..
        } = self.entities.get(key).unwrap();
        let chunk_index = *chunk_index;
        let chunk_size = archetype.components_len();
        let chunk = self.chunks.get_mut(archetype).unwrap();

        // reduces count of shared components
        for key in &archetype.1 {
            let shared = self.shared_components.get_mut(key).unwrap();
            shared.count -= 1;

            if shared.auto_remove && shared.count == 0 {
                self.shared_components.remove(key);
            }
        }

        // swaps and removes components
        let components = if chunk.components.len() == chunk_size {
            chunk.entity_keys.clear();
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
            let swap_entity_id = *chunk.entity_keys.last().unwrap();
            chunk
                .entity_keys
                .swap(chunk_index, chunk.components.len() - 1);
            chunk.entity_keys.truncate(chunk.entity_keys.len() - 1);

            // updates swap entity
            self.entities
                .get_mut(&swap_entity_id)
                .unwrap()
                .set_chunk_index(chunk_index);

            components
        };

        ComponentSet(
            components
                .into_iter()
                .map(
                    |ComponentItem {
                         component,
                         key: type_id,
                     }| (type_id, component),
                )
                .collect(),
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn archetypes(&self) -> Vec<Archetype> {
        self.chunks.keys().cloned().collect()
    }

    pub fn entity_keys(&self) -> Vec<EntityKey> {
        self.entities.keys().copied().collect()
    }

    pub fn shared_components_keys(&self) -> Vec<SharedComponentKey> {
        self.shared_components.keys().copied().collect()
    }

    pub fn has_entity(&self, entity_key: &EntityKey) -> bool {
        self.entities.contains_key(entity_key)
    }

    pub fn create_entity(&mut self, components: ComponentSet) -> Result<EntityKey, Error> {
        let archetype = components.archetype();
        let size = archetype.components_len();
        if size == 0 {
            return Err(Error::EmptyComponents);
        }

        let ComponentSet(components, shared_component_types, shared_components) = components;

        // checks the duplication of shared components
        for shared in shared_component_types
            .iter()
            .chain(shared_components.iter().map(|(id, _)| id))
        {
            if self.shared_components.contains_key(shared) {
                return Err(Error::DuplicateComponent);
            }
        }

        let entity_key = EntityKey::new();

        let chunk = self.get_or_create_chunk(archetype.clone());
        let chunk_index = chunk.components.len() / size;
        // pushes entity id
        chunk.entity_keys.push(entity_key);
        // pushes components
        chunk.components.extend(
            components
                .into_iter()
                .map(|(type_id, component)| ComponentItem {
                    component,
                    key: type_id,
                }),
        );
        let entity = EntityItem {
            version: 0,
            archetype,
            chunk_index,
        };
        // pushes shared components
        shared_components.into_iter().for_each(|(id, component)| {
            self.shared_components.insert_unique_unchecked(
                id,
                SharedComponentItem {
                    component,
                    count: 0,
                    auto_remove: true,
                },
            );
        });

        let (entity_key, _) = self.entities.insert_unique_unchecked(entity_key, entity);

        self.channel.send(CreateEntity::new(*entity_key));

        Ok(*entity_key)
    }

    pub fn remove_entity(&mut self, key: &EntityKey) -> Result<ComponentSet, Error> {
        if !self.has_entity(&key) {
            return Err(Error::NoSuchEntity);
        }

        self.channel.send(RemoveEntity::new(*key));

        unsafe { Ok(self.swap_and_remove_entity(&key)) }
    }

    unsafe fn set_components(&mut self, key: &EntityKey, components: ComponentSet) {
        let archetype = components.archetype();
        let chunk = self.get_or_create_chunk(archetype.clone());
        chunk
            .components
            .extend(
                components
                    .0
                    .into_iter()
                    .map(|(type_id, component)| ComponentItem {
                        component,
                        key: type_id,
                    }),
            );
        chunk.entity_keys.push(*key);
        let chunk_index = chunk.entity_keys.len() - 1;
        self.entities
            .get_mut(key)
            .unwrap()
            .set_archetype(archetype, chunk_index);
    }

    pub fn has_component<C>(&mut self, entity_key: &EntityKey) -> bool
    where
        C: Component + 'static,
    {
        let Some(entity) = self.entities.get(entity_key) else {
            return false;
        };
        entity.archetype.has_component::<C>()
    }

    pub fn add_component<C>(&mut self, entity_key: &EntityKey, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        if !self.has_entity(entity_key) {
            return Err(Error::NoSuchEntity);
        }
        if self.has_component::<C>(entity_key) {
            return Err(Error::DuplicateComponent);
        }

        unsafe {
            let mut components = self.swap_and_remove_entity(entity_key);
            components.add_unique_unchecked(component);
            self.set_components(entity_key, components)
        };

        self.channel.send(AddComponent::new::<C>(*entity_key));

        Ok(())
    }

    pub fn remove_component<C>(&mut self, entity_key: &EntityKey) -> Result<C, Error>
    where
        C: Component + 'static,
    {
        if !self.has_entity(entity_key) {
            return Err(Error::NoSuchEntity);
        }
        if !self.has_component::<C>(entity_key) {
            return Err(Error::NoSuchComponent);
        }

        let removed = unsafe {
            let mut components = self.swap_and_remove_entity(entity_key);
            let removed = components.remove::<C>().unwrap();
            self.set_components(entity_key, components);
            removed
        };

        self.channel.send(RemoveComponent::new::<C>(*entity_key));

        Ok(removed)
    }

    pub fn has_shared_component<C, T>(&self) -> bool
    where
        C: Component + 'static,
        T: 'static,
    {
        self.shared_components
            .contains_key(&SharedComponentKey::new::<C, T>())
    }

    pub fn add_shared_component<C, T>(&mut self, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        if self.has_shared_component::<C, T>() {
            return Err(Error::DuplicateComponent);
        }

        self.shared_components.insert_unique_unchecked(
            SharedComponentKey::new::<C, T>(),
            SharedComponentItem {
                component: Box::new(component),
                count: 0,
                auto_remove: false,
            },
        );

        self.channel.send(AddSharedComponent::new::<C, T>());

        Ok(())
    }

    pub fn remove_shared_component<C, T>(&mut self) -> Result<C, Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        if !self.has_shared_component::<C, T>() {
            return Err(Error::NoSuchComponent);
        }

        if self
            .shared_components
            .get(&SharedComponentKey::new::<C, T>())
            .unwrap()
            .count
            != 0
        {
            return Err(Error::ComponentInUsed);
        }

        let removed = *self
            .shared_components
            .remove(&SharedComponentKey::new::<C, T>())
            .unwrap()
            .component
            .downcast::<C>()
            .unwrap();

        self.channel.send(RemoveSharedComponent::new::<C, T>());

        Ok(removed)
    }

    pub fn iter_mut(&mut self) -> EntityComponentsIterMut {
        EntityComponentsIterMut::new(self)
    }

    pub fn iter(&mut self) -> EntityComponentsIter {
        EntityComponentsIter::new(self)
    }
}

struct EntityItem {
    version: usize,
    archetype: Archetype,
    chunk_index: usize,
}

impl EntityItem {
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

pub(super) struct SharedComponentItem {
    pub(super) component: Box<dyn Any>,
    count: usize,
    auto_remove: bool,
}

pub(super) struct ComponentItem {
    pub(super) component: Box<dyn Any>,
    pub(super) key: ComponentKey,
}

pub(super) struct ChunkItem {
    pub(super) entity_keys: Vec<EntityKey>,
    pub(super) components: Vec<ComponentItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CreateEntity(EntityKey);

impl CreateEntity {
    fn new(entity_key: EntityKey) -> Self {
        Self(entity_key)
    }

    pub fn entity_key(&self) -> &EntityKey {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveEntity(EntityKey);

impl RemoveEntity {
    fn new(entity_key: EntityKey) -> Self {
        Self(entity_key)
    }

    pub fn entity_key(&self) -> &EntityKey {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddComponent(EntityKey, ComponentKey);

impl AddComponent {
    fn new<C>(entity_key: EntityKey) -> Self
    where
        C: Component + 'static,
    {
        Self(entity_key, ComponentKey::new::<C>())
    }

    pub fn entity_key(&self) -> &EntityKey {
        &self.0
    }

    pub fn component_key(&self) -> &ComponentKey {
        &self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveComponent(EntityKey, ComponentKey);

impl RemoveComponent {
    fn new<C>(entity: EntityKey) -> Self
    where
        C: Component + 'static,
    {
        Self(entity, ComponentKey::new::<C>())
    }

    pub fn entity_key(&self) -> &EntityKey {
        &self.0
    }

    pub fn component_key(&self) -> &ComponentKey {
        &self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddSharedComponent(SharedComponentKey);

impl AddSharedComponent {
    fn new<C, T>() -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(SharedComponentKey::new::<C, T>())
    }

    pub fn shared_component_key(&self) -> &SharedComponentKey {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveSharedComponent(SharedComponentKey);

impl RemoveSharedComponent {
    fn new<C, T>() -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(SharedComponentKey::new::<C, T>())
    }

    pub fn shared_component_key(&self) -> &SharedComponentKey {
        &self.0
    }
}
