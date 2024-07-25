use std::any::TypeId;

use hashbrown::HashMap;

use crate::anewthing::channel::Channel;

use super::{
    archetype::Archetype,
    component::{Component, ComponentSet},
    entity::Entity,
    error::Error,
};

pub struct EntityManager {
    channel: Channel,
    entities: HashMap<Entity, EntityTable>,
    chunks: HashMap<Archetype, ChunkTable>,
}

impl EntityManager {
    pub fn new(channel: Channel) -> Self {
        Self {
            channel,
            entities: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    fn get_or_create_chunk<'a: 'b, 'b>(&'a mut self, archetype: Archetype) -> &'b mut ChunkTable {
        self.chunks.entry(archetype).or_insert_with(|| ChunkTable {
            components: Vec::new(),
        })
    }

    pub fn create_entity(&mut self, components: ComponentSet) -> Result<Entity, Error> {
        let archetype = components.archetype();
        let size = archetype.components_per_entity();
        if size == 0 {
            return Err(Error::EmptyComponents);
        }

        let chunk = self.get_or_create_chunk(archetype.clone());
        let total = chunk.components.len();
        let chunk_index = total / size;
        chunk.components.extend(components.0);
        let item = EntityTable {
            version: 0,
            archetype,
            chunk_index,
        };
        let (entity, _) = self.entities.insert_unique_unchecked(Entity::new(), item);

        Ok(*entity)
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        let old = self.entities.get_mut(&entity).ok_or(Error::NoSuchEntity)?;
        let old_version = old.version;
        let old_archetype = old.archetype.clone();
        let old_chunk_index = old.chunk_index;
        let old_size = old_archetype.components_per_entity();
        let old_chunk = self.chunks.get_mut(&old_archetype).unwrap();

        let components = if old_chunk.components.len() == old_size {
            let mut components = old_chunk
                .components
                .drain(..)
                .chain([(TypeId::of::<C>(), Box::new(component) as Box<dyn Component>)])
                .collect::<Vec<_>>();
            components.sort_by(|(a, _), (b, _)| a.cmp(b));
            components
        } else {
            let from_components_index = old_chunk_index * old_size;
            let last_components_index = old_chunk.components.len() - old_size;
            for i in 0..old_size {
                old_chunk
                    .components
                    .swap(from_components_index + i, last_components_index + i);
            }
            let mut components = old_chunk
                .components
                .splice(last_components_index.., [])
                .chain([(TypeId::of::<C>(), Box::new(component) as Box<dyn Component>)])
                .collect::<Vec<_>>();
            components.sort_by(|(a, _), (b, _)| a.cmp(b));

            // last entity should be updated
            let last_chunk_index = old_chunk.components.len() / old_size - 1;
            let swap_id = self
                .entities
                .iter()
                .find(|(_, item)| {
                    item.archetype == old_archetype && item.chunk_index == last_chunk_index
                })
                .map(|(id, _)| *id)
                .unwrap();
            let swap = self.entities.get_mut(&swap_id).unwrap();
            swap.chunk_index = old_chunk_index;
            swap.version = swap.version.wrapping_add(1);

            components
        };

        let new_version = old_version.wrapping_add(1);
        let new_archetype = old_archetype.add_component::<C>()?;
        let new_size = new_archetype.components_per_entity();
        let new_chunk = self.get_or_create_chunk(new_archetype.clone());
        let new_chunk_index = new_chunk.components.len() / new_size;
        new_chunk.components.extend(components);
        self.entities.insert(
            entity,
            EntityTable {
                version: new_version,
                archetype: new_archetype,
                chunk_index: new_chunk_index,
            },
        );

        self.channel.send(AddComponent::new::<C>(entity));

        Ok(())
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        let old = self.entities.get_mut(&entity).ok_or(Error::NoSuchEntity)?;
        let old_version = old.version;
        let old_archetype = old.archetype.clone();
        let old_chunk_index = old.chunk_index;
        let old_size = old_archetype.components_per_entity();
        let old_chunk = self.chunks.get_mut(&old_archetype).unwrap();

        let component_type = TypeId::of::<C>();
        let components = if old_chunk.components.len() == old_size {
            let components = old_chunk
                .components
                .drain(..)
                .filter(|(i, _)| i != &component_type)
                .collect::<Vec<_>>();
            // resorts is unnecessary
            components
        } else {
            let from_components_index = old_chunk_index * old_size;
            let last_components_index = old_chunk.components.len() - old_size;
            for i in 0..old_size {
                old_chunk
                    .components
                    .swap(from_components_index + i, last_components_index + i);
            }
            let components = old_chunk
                .components
                .splice(last_components_index.., [])
                .filter(|(id, _)| id != &component_type)
                .collect::<Vec<_>>();

            // last entity should be updated
            let last_chunk_index = old_chunk.components.len() / old_size - 1;
            let swap_id = self
                .entities
                .iter()
                .find(|(_, item)| {
                    item.archetype == old_archetype && item.chunk_index == last_chunk_index
                })
                .map(|(id, _)| *id)
                .unwrap();
            let swap = self.entities.get_mut(&swap_id).unwrap();
            swap.chunk_index = old_chunk_index;
            swap.version = swap.version.wrapping_add(1);

            components
        };

        let new_version = old_version.wrapping_add(1);
        let new_archetype = old_archetype.remove_component::<C>()?;
        let new_size = new_archetype.components_per_entity();
        let new_chunk = self.get_or_create_chunk(new_archetype.clone());
        let new_chunk_index = new_chunk.components.len() / new_size;
        new_chunk.components.extend(components);
        self.entities.insert(
            entity,
            EntityTable {
                version: new_version,
                archetype: new_archetype,
                chunk_index: new_chunk_index,
            },
        );

        self.channel.send(RemoveComponent::new::<C>(entity));

        Ok(())
    }

    // fn swap_archetype(&mut self)
}

struct EntityTable {
    version: usize,
    archetype: Archetype,
    chunk_index: usize,
}

struct ChunkTable {
    components: Vec<(TypeId, Box<dyn Component>)>,
}

#[derive(Clone, Copy)]
pub struct AddComponent(Entity, TypeId);

impl AddComponent {
    fn new<C>(entity: Entity) -> Self
    where
        C: Component + 'static,
    {
        Self(entity, TypeId::of::<C>())
    }

    pub fn entity(&self) -> &Entity {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}

#[derive(Clone, Copy)]
pub struct RemoveComponent(Entity, TypeId);

impl RemoveComponent {
    fn new<C>(entity: Entity) -> Self
    where
        C: Component + 'static,
    {
        Self(entity, TypeId::of::<C>())
    }

    pub fn entity(&self) -> &Entity {
        &self.0
    }

    pub fn component_type(&self) -> &TypeId {
        &self.1
    }
}
