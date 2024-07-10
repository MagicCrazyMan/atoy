use std::any::TypeId;

use hashbrown::{hash_map::Entry, HashMap};

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

    fn get_or_create_chunk(&mut self, archetype: Archetype) -> &mut ChunkTable {
        self.chunks.entry(archetype).or_insert_with(|| ChunkTable {
            components: Vec::new(),
        })
    }

    fn entity_item_mut(&mut self, entity: Entity) -> Result<&mut EntityItem, Error> {
        let Entity { index, version } = entity;
        let Some(item) = self.entities.get_mut(index) else {
            return Err(Error::NoSuchEntity);
        };
        if item.version != version {
            return Err(Error::NoSuchEntity);
        }
        Ok(item)
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
        let Entity { index, version } = entity;
        let Some(item) = self.entities.get_mut(index) else {
            return Err(Error::NoSuchEntity);
        };
        if item.version != version {
            return Err(Error::NoSuchEntity);
        }

        let EntityItem {
            archetype,
            chunk_index,
            ..
        } = item;
        let chunk = self.chunks.get_mut(archetype).unwrap();
        let size = archetype.components_per_entity();

        let new_archetype = item.archetype.add_component::<C>()?;

        if chunk.components.len() == size {
            let mut components = chunk
                .components
                .splice(.., [])
                .chain([(TypeId::of::<C>(), Box::new(component) as Box<dyn Component>)])
                .collect::<Vec<_>>();
            components.sort_by(|(a, _), (b, _)| a.cmp(b));

            let new_chunk = self.get_or_create_chunk(new_archetype.clone());
            let new_chunk_index = new_chunk.components.len() / size;
            new_chunk.components.extend(components);

            let new_version = item.version.wrapping_add(1);
            item.version = new_version;
            item.archetype = new_archetype.clone();
            item.chunk_index = new_chunk_index;
            Ok(Entity {
                index,
                version: new_version,
            })
        } else {
            for i in 0..size {
                let from_index = *chunk_index * size + i;
                let to_index = chunk.components.len() - size + i;
                chunk.components.swap(from_index, to_index);
            }

            todo!()
        }
    }

    pub fn remove_component<C>(&mut self, entity: Entity) -> Result<Entity, Error>
    where
        C: Component + 'static,
    {
    }

    // fn swap_archetype(&mut self)
}

struct EntityItem {
    version: usize,
    archetype: Archetype,
    chunk_index: usize,
}

struct EntityTable {
    entities: Vec<EntityItem>,
}

struct ChunkTable {
    components: Vec<(TypeId, Box<dyn Component>)>,
}
