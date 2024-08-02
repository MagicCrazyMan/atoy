use std::any::{Any, TypeId};

use hashbrown::HashMap;

use crate::anewthing::{channel::Channel, key::Key};

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, ComponentSet, SharedComponentKey},
    entity::EntityKey,
    error::Error,
};

pub struct EntityManager {
    channel: Channel,
    entities: HashMap<EntityKey, EntityItem>,
    chunks: HashMap<Archetype, ChunkItem>,
    shared_components: HashMap<SharedComponentKey, SharedComponentItem>,
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
        let chunk_size = archetype.len();
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

    pub fn has_entity(&self, entity_key: &EntityKey) -> bool {
        self.entities.contains_key(entity_key)
    }

    pub fn create_entity(&mut self, components: ComponentSet) -> Result<EntityKey, Error> {
        let archetype = components.archetype();
        let size = archetype.len();
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

    pub fn has_shared_component<C>(&self, key: &Key) -> bool
    where
        C: Component + 'static,
    {
        self.shared_components
            .contains_key(&SharedComponentKey::new::<C>(key.clone()))
    }

    pub fn add_shared_component<C>(&mut self, key: Key, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        if self.has_shared_component::<C>(&key) {
            return Err(Error::DuplicateComponent);
        }

        self.shared_components.insert_unique_unchecked(
            SharedComponentKey::new::<C>(key),
            SharedComponentItem {
                component: Box::new(component),
                count: 0,
                auto_remove: false,
            },
        );

        Ok(())
    }

    pub fn remove_shared_component<C>(&mut self, key: &Key) -> Result<C, Error>
    where
        C: Component + 'static,
    {
        if !self.has_shared_component::<C>(key) {
            return Err(Error::NoSuchComponent);
        }

        let removed = *self
            .shared_components
            .remove(&SharedComponentKey::new::<C>(key.clone()))
            .unwrap()
            .component
            .downcast::<C>()
            .unwrap();

        Ok(removed)
    }

    // pub fn query(&mut self) {
    //     let archetypes = self.chunks.keys().filter(|ar| )
    // }
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

struct SharedComponentItem {
    component: Box<dyn Any>,
    count: usize,
    auto_remove: bool,
}

struct ComponentItem {
    component: Box<dyn Any>,
    key: ComponentKey,
}

struct ChunkItem {
    entity_keys: Vec<EntityKey>,
    components: Vec<ComponentItem>,
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
pub struct AddComponent(EntityKey, TypeId);

impl AddComponent {
    fn new<C>(entity_key: EntityKey) -> Self
    where
        C: Component + 'static,
    {
        Self(entity_key, TypeId::of::<C>())
    }

    pub fn entity_id(&self) -> &EntityKey {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoveComponent(EntityKey, TypeId);

impl RemoveComponent {
    fn new<C>(entity: EntityKey) -> Self
    where
        C: Component + 'static,
    {
        Self(entity, TypeId::of::<C>())
    }

    pub fn entity_key(&self) -> &EntityKey {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}
