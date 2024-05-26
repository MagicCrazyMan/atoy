use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::core::{
    channel::{MessageChannel, Sender},
    Rrc,
};

use super::{
    archetype::{Archetype, AsArchetype, Chunk},
    component::Component,
    entity::Entity,
    iter::{ArchetypeIter, Iter},
    message::Message,
};

pub struct EntityManager {
    pub(super) id: Uuid,

    pub(super) chunks: Rrc<HashMap<Archetype, Chunk>>,
    pub(super) entities: Rrc<HashMap<Uuid, Rrc<Entity>>>,

    pub(super) sender: Sender<Message>,
}

impl EntityManager {
    pub fn new(channel: MessageChannel) -> Self {
        Self {
            id: Uuid::new_v4(),

            chunks: Rc::new(RefCell::new(HashMap::new())),
            entities: Rc::new(RefCell::new(HashMap::new())),

            sender: channel.sender(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    fn chunk_or_create(&self, archetype: Archetype) -> RefMut<'_, Chunk> {
        RefMut::map(self.chunks.borrow_mut(), |chunks| {
            match chunks.entry(archetype) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(Chunk::new(self.sender.clone())),
            }
        })
    }

    pub fn entity(&self, id: &Uuid) -> Option<Rrc<Entity>> {
        self.entities.borrow().get(id).cloned()
    }

    pub fn create_empty_entity(&self) -> Rrc<Entity> {
        Self::create_entity(&self, [])
    }

    pub fn create_entity<I>(&self, components: I) -> Rrc<Entity>
    where
        I: IntoIterator<Item = Box<dyn Component>>,
    {
        let (components, component_types) = components.into_iter().fold(
            (HashMap::new(), Vec::new()),
            |(mut components, mut component_types), component| {
                let component_type = component.component_type_instanced();
                components.insert(component_type, component);
                component_types.push(component_type);

                (components, component_types)
            },
        );
        let entity = Rc::new(RefCell::new(Entity::new(self, components)));
        let archetype = Archetype::new(component_types);

        self.entities
            .borrow_mut()
            .insert_unique_unchecked(entity.borrow().id, Rc::clone(&entity));
        self.chunk_or_create(archetype)
            .add_entity_unchecked(Rc::clone(&entity));
        entity
    }

    pub fn remove_entity(&mut self, entity_id: &Uuid) {
        let Some(entity) = self.entities.borrow_mut().remove(entity_id) else {
            return;
        };
        let archetype = entity.borrow().archetype();
        self.chunks
            .borrow_mut()
            .get_mut(&archetype)
            .unwrap()
            .remove_entity(entity_id);
    }

    pub fn entities(&self) -> Iter {
        Iter::new(self)
    }

    pub fn entities_of_archetype<I>(&self) -> Option<ArchetypeIter>
    where
        I: AsArchetype + 'static,
    {
        ArchetypeIter::new(self, I::as_archetype())
    }
}
