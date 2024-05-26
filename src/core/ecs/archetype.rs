use std::any::TypeId;

use hashbrown::HashMap;
use smallvec::SmallVec;
use uuid::Uuid;

use crate::core::{channel::Sender, Rrc};

use super::{component::Component, entity::Entity, message::Message};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(SmallVec<[TypeId; 3]>);

impl Archetype {
    pub fn new<I>(component_types: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        let mut component_types: SmallVec<[TypeId; 3]> = component_types.into_iter().collect();
        component_types.sort();
        component_types.dedup();
        Self(component_types)
    }
}

pub trait AsArchetype {
    fn as_archetype() -> Archetype;
}

impl<A0> AsArchetype for A0
where
    A0: Component + 'static,
{
    fn as_archetype() -> Archetype {
        Archetype::new([TypeId::of::<A0>()])
    }
}

macro_rules! as_archetype {
    ($($ct: tt),+) => {
        impl<$($ct,)+> AsArchetype for ($($ct,)+)
        where
            $(
                $ct: Component + 'static,
            )+
        {
            fn as_archetype() -> Archetype {
                Archetype::new([
                    $(
                        TypeId::of::<$ct>(),
                    )+
                ])
            }
        }
    };
}

as_archetype!(A0);
as_archetype!(A0, A1);
as_archetype!(A0, A1, A2);
as_archetype!(A0, A1, A2, A3);

pub(super) struct Chunk {
    pub(super) entities: HashMap<Uuid, Rrc<Entity>>,
    sender: Sender<Message>,
}

impl Chunk {
    pub(super) fn new(sender: Sender<Message>) -> Self {
        Self {
            entities: HashMap::new(),
            sender,
        }
    }

    pub(super) fn add_entity_unchecked(&mut self, entity: Rrc<Entity>) {
        let id = entity.borrow().id;
        self.entities.insert_unique_unchecked(id, entity);
        self.sender.send(Message::AddEntity { entity_id: id });
    }

    pub(super) fn remove_entity(&mut self, id: &Uuid) -> Option<Rrc<Entity>> {
        let removed = self.entities.remove(id)?;
        self.sender.send(Message::RemoveEntity {
            entity_id: removed.borrow().id,
        });
        Some(removed)
    }
}
