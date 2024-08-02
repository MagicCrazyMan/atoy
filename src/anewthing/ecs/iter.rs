use std::{any::Any, marker::PhantomData};

use hashbrown::HashMap;

use crate::anewthing::ecs::manager::ComponentItem;

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, SharedComponentKey},
    entity::EntityKey,
    manager::{ChunkItem, EntityManager, SharedComponentItem},
};

pub struct EntityComponentsMut<'a> {
    entity_key: *const EntityKey,
    archetype: *const Archetype,
    components: HashMap<ComponentKey, *mut Box<dyn Any>>,
    shared_components: HashMap<SharedComponentKey, *mut Box<dyn Any>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> EntityComponentsMut<'a> {
    pub fn entity_key(&self) -> &EntityKey {
        unsafe { &*self.entity_key }
    }

    pub fn archetype(&self) -> &Archetype {
        unsafe { &*self.archetype }
    }

    /// Returns a component with a specific component type.
    pub fn component<C>(&mut self) -> Option<&mut C>
    where
        C: Component + 'static,
    {
        unsafe {
            Some(
                self.components
                    .get_mut(&ComponentKey::new::<C>())?
                    .as_mut()
                    .unwrap()
                    .downcast_mut::<C>()
                    .unwrap(),
            )
        }
    }

    /// Returns a component with a specific component type.
    /// Panic if no such component.
    pub fn component_unchecked<C>(&mut self) -> &mut C
    where
        C: Component + 'static,
    {
        unsafe {
            self.components
                .get_mut(&ComponentKey::new::<C>())
                .unwrap()
                .as_mut()
                .unwrap()
                .downcast_mut::<C>()
                .unwrap()
        }
    }

    /// Returns a shared component with a specific component type.
    pub fn shared_component<C, T>(&mut self) -> Option<&mut C>
    where
        C: Component + 'static,
        T: 'static,
    {
        unsafe {
            Some(
                self.shared_components
                    .get_mut(&SharedComponentKey::new::<C, T>())?
                    .as_mut()
                    .unwrap()
                    .downcast_mut::<C>()
                    .unwrap(),
            )
        }
    }

    /// Returns a shared component with a specific component type.
    /// Panic if no such component.
    pub fn shared_component_unchecked<C, T>(&mut self) -> &mut C
    where
        C: Component + 'static,
        T: 'static,
    {
        unsafe {
            self.shared_components
                .get_mut(&SharedComponentKey::new::<C, T>())
                .unwrap()
                .as_mut()
                .unwrap()
                .downcast_mut::<C>()
                .unwrap()
        }
    }
}

pub struct EntityComponentsIterMut<'a> {
    shared_components: &'a mut HashMap<SharedComponentKey, SharedComponentItem>,
    chunks: hashbrown::hash_map::IterMut<'a, Archetype, ChunkItem>,
    chunk: Option<(&'a Archetype, &'a mut ChunkItem)>,
    chunk_index: usize,
}

impl<'a> EntityComponentsIterMut<'a> {
    pub(super) fn new(manager: &'a mut EntityManager) -> Self {
        Self {
            shared_components: &mut manager.shared_components,
            chunks: manager.chunks.iter_mut(),
            chunk: None,
            chunk_index: 0,
        }
    }
}

impl<'a> Iterator for EntityComponentsIterMut<'a> {
    type Item = EntityComponentsMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk.is_none() {
            self.chunk = self.chunks.next();
            if self.chunk.is_none() {
                return None;
            }
        }

        let (archetype, chunk_item) = self.chunk.as_mut().unwrap();

        let entity_key: *const EntityKey = &chunk_item.entity_keys[self.chunk_index];

        let components_len = archetype.components_len();
        let components_start_index = self.chunk_index * components_len;
        let components_end_index = components_start_index + components_len;
        let components = chunk_item.components[components_start_index..components_end_index]
            .iter_mut()
            .map(|ComponentItem { key, component }| (*key, component as *mut _))
            .collect::<HashMap<_, _>>();

        let mut shared_components = HashMap::with_capacity(archetype.shared_components_len());
        for key in archetype.shared_component_keys() {
            let shared_component: *mut _ =
                &mut self.shared_components.get_mut(key).unwrap().component;
            shared_components.insert(*key, shared_component);
        }

        let archetype = *archetype as *const _;

        self.chunk_index += 1;
        if self.chunk_index >= chunk_item.entity_keys.len() {
            self.chunk = None;
            self.chunk_index = 0;
        }

        Some(EntityComponentsMut {
            entity_key,
            archetype,
            components,
            shared_components,
            _lifetime: PhantomData,
        })
    }
}

pub struct EntityComponents<'a> {
    entity_key: &'a EntityKey,
    archetype: &'a Archetype,
    components: HashMap<ComponentKey, &'a Box<dyn Any>>,
    shared_components: HashMap<SharedComponentKey, &'a Box<dyn Any>>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> EntityComponents<'a> {
    pub fn entity_key(&self) -> &EntityKey {
        &self.entity_key
    }

    pub fn archetype(&self) -> &Archetype {
        &self.archetype
    }

    /// Returns a component with a specific component type.
    pub fn component<C>(&self) -> Option<&C>
    where
        C: Component + 'static,
    {
        Some(
            self.components
                .get(&ComponentKey::new::<C>())?
                .downcast_ref::<C>()
                .unwrap(),
        )
    }

    /// Returns a component with a specific component type.
    /// Panic if no such component.
    pub fn component_unchecked<C>(&self) -> &C
    where
        C: Component + 'static,
    {
        self.components
            .get(&ComponentKey::new::<C>())
            .unwrap()
            .downcast_ref::<C>()
            .unwrap()
    }

    /// Returns a shared component with a specific component type.
    pub fn shared_component<C, T>(&self) -> Option<&C>
    where
        C: Component + 'static,
        T: 'static,
    {
        Some(
            self.shared_components
                .get(&SharedComponentKey::new::<C, T>())?
                .downcast_ref::<C>()
                .unwrap(),
        )
    }

    /// Returns a shared component with a specific component type.
    /// Panic if no such component.
    pub fn shared_component_unchecked<C, T>(&self) -> &C
    where
        C: Component + 'static,
        T: 'static,
    {
        self.shared_components
            .get(&SharedComponentKey::new::<C, T>())
            .unwrap()
            .downcast_ref::<C>()
            .unwrap()
    }
}

pub struct EntityComponentsIter<'a> {
    shared_components: &'a HashMap<SharedComponentKey, SharedComponentItem>,
    chunks: hashbrown::hash_map::Iter<'a, Archetype, ChunkItem>,
    chunk: Option<(&'a Archetype, &'a ChunkItem)>,
    chunk_index: usize,
}

impl<'a> EntityComponentsIter<'a> {
    pub(super) fn new(manager: &'a EntityManager) -> Self {
        Self {
            shared_components: &manager.shared_components,
            chunks: manager.chunks.iter(),
            chunk: None,
            chunk_index: 0,
        }
    }
}

impl<'a> Iterator for EntityComponentsIter<'a> {
    type Item = EntityComponents<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chunk.is_none() {
            self.chunk = self.chunks.next();
            if self.chunk.is_none() {
                return None;
            }
        }

        let (archetype, chunk_item) = self.chunk.unwrap();

        let entity_key = &chunk_item.entity_keys[self.chunk_index];

        let components_len = archetype.components_len();
        let components_start_index = self.chunk_index * components_len;
        let components_end_index = components_start_index + components_len;
        let components = chunk_item.components[components_start_index..components_end_index]
            .iter()
            .map(|ComponentItem { key, component }| (*key, component))
            .collect::<HashMap<_, _>>();

        let mut shared_components = HashMap::with_capacity(archetype.shared_components_len());
        for key in archetype.shared_component_keys() {
            let shared_component = &self.shared_components.get(key).unwrap().component;
            shared_components.insert(*key, shared_component);
        }

        self.chunk_index += 1;
        if self.chunk_index >= chunk_item.entity_keys.len() {
            self.chunk = None;
            self.chunk_index = 0;
        }

        Some(EntityComponents {
            entity_key,
            archetype,
            components,
            shared_components,
            _lifetime: PhantomData,
        })
    }
}
