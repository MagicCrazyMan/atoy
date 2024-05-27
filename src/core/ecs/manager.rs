use std::{
    any::TypeId,
    cell::{RefCell, RefMut},
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::core::{app::AppConfig, carrier::Carrier, Rrc};

use super::{
    archetype::{Archetype, AsArchetype},
    component::Component,
    entity::Entity,
    iter::{ArchetypeIter, Iter},
    AddComponent, AddEntity, RemoveComponent, RemoveEntity, ReplaceComponent,
};

pub struct EntityManager {
    pub(super) id: Uuid,

    pub(super) archetypes: Rrc<HashMap<Archetype, HashMap<Uuid, Rrc<Entity>>>>,
    pub(super) entities: Rrc<HashMap<Uuid, Rrc<Entity>>>,

    add_entity: Carrier<AddEntity>,
    remove_entity: Carrier<RemoveEntity>,
    add_component: Carrier<AddComponent>,
    remove_component: Carrier<RemoveComponent>,
    replace_component: Carrier<ReplaceComponent>,
}

impl EntityManager {
    pub fn new(app_config: &AppConfig) -> Self {
        Self {
            id: Uuid::new_v4(),

            archetypes: Rc::new(RefCell::new(HashMap::new())),
            entities: Rc::new(RefCell::new(HashMap::new())),

            add_entity: app_config.on_add_entity().clone(),
            remove_entity: app_config.on_remove_entity().clone(),
            add_component: app_config.on_add_component().clone(),
            remove_component: app_config.on_remove_component().clone(),
            replace_component: app_config.on_replace_component().clone(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    fn chunk_or_create(&self, archetype: Archetype) -> RefMut<'_, HashMap<Uuid, Rrc<Entity>>> {
        RefMut::map(
            self.archetypes.borrow_mut(),
            |archetypes| match archetypes.entry(archetype) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(HashMap::new()),
            },
        )
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
        let entity = Entity::new(components);
        let archetype = Archetype::new(component_types);

        let id = entity.id;
        let entity = Rc::new(RefCell::new(entity));
        self.entities
            .borrow_mut()
            .insert_unique_unchecked(id, Rc::clone(&entity));
        self.chunk_or_create(archetype)
            .insert(id, Rc::clone(&entity));

        self.add_entity.send(&mut AddEntity {});

        entity
    }

    pub fn remove_entity(&mut self, id: &Uuid) {
        let Some(entity) = self.entities.borrow_mut().remove(id) else {
            return;
        };
        let archetype = entity.borrow().archetype();

        self.archetypes
            .borrow_mut()
            .get_mut(&archetype)
            .unwrap()
            .remove(id);

        self.remove_entity.send(&mut RemoveEntity {});
    }

    pub fn remove_component<T>(&self, id: &Uuid)
    where
        T: Component + 'static,
    {
        let Some(entity) = self.entity(id) else {
            return;
        };
        let id = entity.borrow().id;

        let old_archetype = entity.borrow().archetype();
        entity.borrow_mut().components.remove(&TypeId::of::<T>());
        let new_archetype = entity.borrow().archetype();

        if old_archetype != new_archetype {
            self.chunk_or_create(old_archetype).remove(&id);
            self.chunk_or_create(new_archetype).insert(id, entity);
            self.remove_component.send(&mut RemoveComponent {});
        }
    }

    pub fn add_component<T>(&self, id: &Uuid, component: T)
    where
        T: Component + 'static,
    {
        let Some(entity) = self.entity(id) else {
            return;
        };
        let id = entity.borrow().id;

        let old_archetype = entity.borrow().archetype();
        entity
            .borrow_mut()
            .components
            .insert(TypeId::of::<T>(), Box::new(component));
        let new_archetype = entity.borrow().archetype();

        if old_archetype == new_archetype {
            self.replace_component.send(&mut ReplaceComponent {});
        } else {
            self.chunk_or_create(old_archetype).remove(&id);
            self.chunk_or_create(new_archetype).insert(id, entity);
            self.add_component.send(&mut AddComponent {});
        }
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
