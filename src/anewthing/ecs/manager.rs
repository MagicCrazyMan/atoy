use std::any::TypeId;

use hashbrown::HashMap;

use super::{
    archetype::Archetype,
    component::{Component, ComponentSet},
    entity::Entity,
    error::Error,
};

pub struct EntityManager {
    entities: Vec<EntityItem>,
    chunks: HashMap<Archetype, ChunkTable>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            chunks: HashMap::new(),
        }
    }

    fn get_or_create_chunk<'a: 'b, 'b>(&'a mut self, archetype: Archetype) -> &'b mut ChunkTable {
        self.chunks.entry(archetype).or_insert_with(|| ChunkTable {
            components: Vec::new(),
        })
    }

    fn verify_entity(&self, entity: Entity) -> Result<(), Error> {
        let Entity { index, version } = entity;
        let Some(item) = self.entities.get(index) else {
            return Err(Error::NoSuchEntity);
        };
        if item.version != version {
            return Err(Error::NoSuchEntity);
        }
        Ok(())
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
        let item = EntityItem {
            version: 0,
            archetype,
            chunk_index,
        };
        self.entities.push(item);

        Ok(Entity::new(self.entities.len() - 1, 0))
    }

    pub fn add_component<C>(&mut self, entity: Entity, component: C) -> Result<Entity, Error>
    where
        C: Component + 'static,
    {
        self.verify_entity(entity)?;

        let index = entity.index;

        let old_version = entity.version;
        let old_archetype = &self.entities[index].archetype;
        let old_chunk = self.chunks.get_mut(old_archetype).unwrap();
        let old_chunk_index = self.entities[index].chunk_index;
        let old_size = old_archetype.components_per_entity();

        let new_version = old_version.wrapping_add(1);
        let new_archetype = old_archetype.add_component::<C>()?;
        let new_size = new_archetype.components_per_entity();

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
            let index = self
                .entities
                .iter()
                .position(|item| {
                    &item.archetype == old_archetype && item.chunk_index == last_chunk_index
                })
                .unwrap();
            self.entities[index].chunk_index = old_chunk_index;
            self.entities[index].version = self.entities[index].version.wrapping_add(1);

            components
        };

        let new_chunk = self.get_or_create_chunk(new_archetype.clone());
        let new_chunk_index = new_chunk.components.len() / new_size;
        new_chunk.components.extend(components);

        self.entities[index] = EntityItem {
            version: new_version,
            archetype: new_archetype,
            chunk_index: new_chunk_index,
        };
        Ok(Entity {
            index,
            version: new_version,
        })
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Result<Entity, Error>
    where
        C: Component + 'static,
    {
        self.verify_entity(entity)?;

        let type_id = TypeId::of::<C>();
        let index = entity.index;

        let old_version = entity.version;
        let old_archetype = &self.entities[index].archetype;
        let old_chunk = self.chunks.get_mut(old_archetype).unwrap();
        let old_chunk_index = self.entities[index].chunk_index;
        let old_size = old_archetype.components_per_entity();

        let new_version = old_version.wrapping_add(1);
        let new_archetype = old_archetype.remove_component::<C>()?;
        let new_size = new_archetype.components_per_entity();

        let components = if old_chunk.components.len() == old_size {
            let components = old_chunk
                .components
                .drain(..)
                .filter(|(i, _)| i != &type_id)
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
                .filter(|(i, _)| i != &type_id)
                .collect::<Vec<_>>();

            // last entity should be updated
            let last_chunk_index = old_chunk.components.len() / old_size - 1;
            let index = self
                .entities
                .iter()
                .position(|item| {
                    &item.archetype == old_archetype && item.chunk_index == last_chunk_index
                })
                .unwrap();
            self.entities[index].chunk_index = old_chunk_index;
            self.entities[index].version = self.entities[index].version.wrapping_add(1);

            components
        };

        let new_chunk = self.get_or_create_chunk(new_archetype.clone());
        let new_chunk_index = new_chunk.components.len() / new_size;
        new_chunk.components.extend(components);

        self.entities[index] = EntityItem {
            version: new_version,
            archetype: new_archetype,
            chunk_index: new_chunk_index,
        };
        Ok(Entity {
            index,
            version: new_version,
        })
    }

    // fn swap_archetype(&mut self)
}

struct EntityItem {
    version: usize,
    archetype: Archetype,
    chunk_index: usize,
}

struct ChunkTable {
    components: Vec<(TypeId, Box<dyn Component>)>,
}
