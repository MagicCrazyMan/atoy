use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::core::{carrier::Carrier, Rrc};

use super::{
    archetype::{Archetype, AsArchetype},
    component::Component,
    entity::Entity,
    iter::{ArchetypeIter, Iter},
};

#[derive(Clone)]
pub struct AddEntity {
    pub entity_id: Uuid,
}

#[derive(Clone)]
pub struct RemoveEntity {
    pub entity_id: Uuid,
}

#[derive(Clone)]
pub struct UpdateComponent {
    pub entity_id: Uuid,
}

#[derive(Clone)]
pub struct AddComponent {
    pub entity_id: Uuid,
    pub old_archetype: Archetype,
    pub new_archetype: Archetype,
}

#[derive(Clone)]
pub struct RemoveComponent {
    pub entity_id: Uuid,
    pub old_archetype: Archetype,
    pub new_archetype: Archetype,
}

#[derive(Clone)]
pub struct ReplaceComponent {
    pub entity_id: Uuid,
}

pub struct EntityManager {
    pub(super) archetypes: Rrc<HashMap<Archetype, HashMap<Uuid, Rrc<Entity>>>>,
    pub(super) entities: Rrc<HashMap<Uuid, Rrc<Entity>>>,

    add_entity: Carrier<AddEntity>,
    remove_entity: Carrier<RemoveEntity>,
    update_component: Carrier<UpdateComponent>,
    add_component: Carrier<AddComponent>,
    remove_component: Carrier<RemoveComponent>,
    replace_component: Carrier<ReplaceComponent>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            archetypes: Rc::new(RefCell::new(HashMap::new())),
            entities: Rc::new(RefCell::new(HashMap::new())),

            add_entity: Carrier::new(),
            remove_entity: Carrier::new(),
            update_component: Carrier::new(),
            add_component: Carrier::new(),
            remove_component: Carrier::new(),
            replace_component: Carrier::new(),
        }
    }
}
