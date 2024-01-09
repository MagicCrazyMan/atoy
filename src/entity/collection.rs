use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    event::EventAgency,
};

use super::Entity;

struct Inner {
    id: Uuid,
    entities: Vec<Entity>,
    collections: Vec<EntityCollection>,
    model_matrix: Mat4,
}

pub(super) struct Runtime {
    pub(super) dirty: bool,
    parent: Option<EntityCollectionWeak>,
    bounding: Option<CullingBoundingVolume>,
    pub(super) compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
    enable_bounding: bool,
    changed_event: EventAgency<Event>,
    delegated_events: HashMap<Uuid, Uuid>,
}

/// An [`Entity`] and `EntityCollection` container.
/// Model matrix of an `EntityCollection` effects all entities and sub-collections.
pub struct EntityCollection {
    marker: Rc<()>,
    inner: *mut Inner,
    runtime: *mut Runtime,
}

impl EntityCollection {
    /// Constructs a new entity collection.
    pub fn new() -> Self {
        let inner = Inner {
            id: Uuid::new_v4(),
            entities: Vec::new(),
            collections: Vec::new(),
            model_matrix: Mat4::new_identity(),
        };
        let runtime = Runtime {
            dirty: true,
            parent: None,
            bounding: None,
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
            enable_bounding: true,
            changed_event: EventAgency::new(),
            delegated_events: HashMap::new(),
        };

        Self {
            marker: Rc::new(()),
            inner: Box::leak(Box::new(inner)),
            runtime: Box::leak(Box::new(runtime)),
        }
    }

    #[inline]
    fn inner(&self) -> &Inner {
        unsafe { &*self.inner }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut Inner {
        unsafe { &mut *self.inner }
    }

    #[inline]
    pub(super) fn runtime(&self) -> &Runtime {
        unsafe { &*self.runtime }
    }

    #[inline]
    fn runtime_mut(&mut self) -> &mut Runtime {
        unsafe { &mut *self.runtime }
    }

    fn undelegate_entity_event(&mut self, entity: &Entity) {
        let Some(listener) = self.runtime_mut().delegated_events.remove(entity.id()) else {
            return;
        };
        entity.runtime().changed_event.off(&listener);
    }

    fn delegate_entity_event(&mut self, entity: &Entity) {
        let me = self.weak();
        let listener = entity.runtime().changed_event.on(move |_| {
            let Some(me) = me.upgrade() else {
                return;
            };

            me.runtime()
                .changed_event
                .raise(Event::new(EventKind::EntityChanged, &me));
        });
        self.runtime_mut()
            .delegated_events
            .insert(*entity.id(), listener);
    }

    fn undelegate_collection_event(&mut self, collection: &EntityCollection) {
        let Some(listener) = self.runtime_mut().delegated_events.remove(collection.id()) else {
            return;
        };
        collection.runtime().changed_event.off(&listener);
    }

    fn delegate_collection_event(&mut self, collection: &EntityCollection) {
        let me = self.weak();
        let listener = collection.runtime().changed_event.on(move |e| {
            let Some(me) = me.upgrade() else {
                return;
            };

            me.runtime()
                .changed_event
                .raise(Event::new(e.kind(), e.entity_collection()));
        });
        self.runtime_mut()
            .delegated_events
            .insert(*collection.id(), listener);
    }

    pub fn weak(&self) -> EntityCollectionWeak {
        EntityCollectionWeak {
            marker: Rc::downgrade(&self.marker),
            inner: self.inner,
            runtime: self.runtime,
        }
    }

    /// Returns collection id.
    pub fn id(&self) -> &Uuid {
        &self.inner().id
    }

    /// Returns `true` if this collection enable bounding volume.
    pub fn bounding_enabled(&self) -> bool {
        self.runtime().enable_bounding
    }

    /// Enables bounding volume for this collection.
    pub fn enable_bounding(&mut self) {
        self.runtime_mut().enable_bounding = true;
        self.runtime_mut().dirty = true;
    }

    /// Disables bounding volume for this collection.
    pub fn disable_bounding(&mut self) {
        self.runtime_mut().enable_bounding = false;
        self.runtime_mut().dirty = true;
    }

    /// Returns culling bounding volume.
    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.runtime().bounding.as_ref()
    }

    /// Returns entities in this collection.
    pub fn entities(&self) -> &[Entity] {
        &self.inner().entities
    }

    /// Returns mutable entities in this collection.
    pub fn entities_mut(&mut self) -> &mut [Entity] {
        &mut self.inner_mut().entities
    }

    /// Adds a new entity to this collection.
    pub fn add_entity(&mut self, mut entity: Entity) {
        if entity.runtime().collection.is_some() {
            panic!("share entity between multiple entity collection is not allowed");
        }

        entity.runtime_mut().dirty = true;
        entity.runtime_mut().collection = Some(self.weak());
        self.delegate_entity_event(&entity);
        self.inner_mut().entities.push(entity);

        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddEntity, self));
    }

    /// Returns an entity from this collection by index.
    pub fn get_entity_by_index(&mut self, index: usize) -> Option<&Entity> {
        self.inner().entities.get(index)
    }

    /// Returns a mutable entity from this collection by index.
    pub fn get_mut_entity_by_index(&mut self, index: usize) -> Option<&mut Entity> {
        self.inner_mut().entities.get_mut(index)
    }

    /// Removes an entity from this collection by index.
    pub fn remove_entity_by_index(&mut self, index: usize) -> Option<Entity> {
        if index > self.inner().entities.len() - 1 {
            return None;
        }

        let mut entity = self.inner_mut().entities.remove(index);
        entity.runtime_mut().dirty = true;
        entity.runtime_mut().collection = None;
        self.undelegate_entity_event(&entity);
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::RemoveEntity, self));
        // self.entities_hash.remove(&entity.id);
        Some(entity)
    }

    // pub fn remove_entity_by_id(&mut self, id: &Uuid) -> Option<Rc<RefCell<Entity>>> {
    //     let Some(index) = self.entities.iter().position(|entity| entity.borrow().id() == id) else {
    //         return None;
    //     };

    //     let entity = self.entities.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(entity)
    // }

    /// Returns sub-collections in this collection.
    pub fn collections(&self) -> &[Self] {
        &self.inner().collections
    }

    /// Returns mutable sub-collections in this collection.
    pub fn collections_mut(&mut self) -> &mut [Self] {
        &mut self.inner_mut().collections
    }

    /// Adds a new sub-collection to this collection.
    pub fn add_collection(&mut self, mut collection: Self) {
        if collection.runtime().parent.is_some() {
            panic!("share entity collection between multiple entity collection is not allowed");
        }

        collection.runtime_mut().dirty = true;
        collection.runtime_mut().parent = Some(self.weak());
        self.delegate_collection_event(&collection);
        self.inner_mut().collections.push(collection);

        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddEntityCollection, self));
    }

    /// Returns a sub-collection from this collection by index.
    pub fn get_collection_by_index(&mut self, index: usize) -> Option<&Self> {
        self.inner().collections.get(index)
    }

    /// Returns a mutable sub-collection from this collection by index.
    pub fn get_mut_collection_by_index(&mut self, index: usize) -> Option<&mut Self> {
        self.inner_mut().collections.get_mut(index)
    }

    /// Removes a sub-collection from this collection by index.
    pub fn remove_collection_by_index(&mut self, index: usize) -> Option<Self> {
        if index > self.inner().collections.len() - 1 {
            return None;
        }

        let mut collection = self.inner_mut().collections.remove(index);
        collection.runtime_mut().dirty = true;
        collection.runtime_mut().parent = None;
        self.undelegate_collection_event(&collection);
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::RemoveEntityCollection, self));
        // self.entities_hash.remove(&entity.id);
        Some(collection)
    }

    // pub fn remove_collection_by_id(&mut self, id: &Uuid) -> Option<Self> {
    //     let Some(index) = self.collections.iter().position(|entity| &entity.id == id) else {
    //         return None;
    //     };

    //     let collection = self.collections.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(collection)
    // }

    /// Returns model matrix of this collection.
    pub fn model_matrix(&self) -> &Mat4 {
        &self.inner().model_matrix
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.runtime().compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.runtime().compose_normal_matrix
    }

    /// Sets model matrix for this collection.
    pub fn set_model_matrix(&mut self, mat: Mat4) {
        self.inner_mut().model_matrix = mat;
        self.runtime_mut().dirty = true;

        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ModelMatrix, self));
    }

    pub fn update(&mut self) {
        if !self.runtime().dirty
            && !self
                .runtime()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade())
                .map(|parent| parent.runtime().dirty)
                .unwrap_or(false)
        {
            return;
        }

        self.update_matrix();
        self.update_bounding();
        self.runtime_mut().dirty = false;
    }

    fn update_matrix(&mut self) {
        let compose_model_matrix = match self
            .runtime()
            .parent
            .as_ref()
            .and_then(|parent| parent.upgrade())
            .map(|parent| parent.runtime().compose_model_matrix)
        {
            Some(parent_model_matrix) => parent_model_matrix * self.inner().model_matrix,
            None => self.inner().model_matrix,
        };
        self.runtime_mut().compose_model_matrix = compose_model_matrix;
        self.runtime_mut().compose_normal_matrix = self
            .runtime()
            .compose_model_matrix
            .invert()
            .expect("matrix with zero determinant is not allowed")
            .transpose();
        self.inner_mut().entities.iter_mut().for_each(|entity| {
            entity.update();
        });
        self.inner_mut()
            .collections
            .iter_mut()
            .for_each(|collection| {
                collection.update();
            });
    }

    /// Creates a large bounding volume that contains all entities in this collection.
    fn update_bounding(&mut self) {
        if !self.runtime().enable_bounding {
            return;
        }

        let collection_boundings = self
            .inner()
            .collections
            .iter()
            .filter_map(|collection| collection.bounding().map(|bounding| bounding.bounding()));
        let entity_boundings = self
            .inner()
            .entities
            .iter()
            .filter_map(|entity| entity.bounding().map(|bounding| bounding.bounding()));
        let boundings = collection_boundings.chain(entity_boundings);
        self.runtime_mut().bounding =
            merge_bounding_volumes(boundings).map(|bounding| CullingBoundingVolume::new(bounding));
    }

    pub fn changed_event(&self) -> &EventAgency<Event> {
        &self.runtime().changed_event
    }
}

impl Drop for EntityCollection {
    fn drop(&mut self) {
        if Rc::strong_count(&self.marker) == 1 {
            unsafe { drop(Box::from_raw(self.inner)) }
            unsafe { drop(Box::from_raw(self.runtime)) }
        }
    }
}

pub struct EntityCollectionWeak {
    marker: Weak<()>,
    inner: *mut Inner,
    runtime: *mut Runtime,
}

impl EntityCollectionWeak {
    pub fn upgrade(&self) -> Option<EntityCollection> {
        self.marker.upgrade().map(|marker| EntityCollection {
            marker,
            inner: self.inner,
            runtime: self.runtime,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    EntityChanged,
    ModelMatrix,
    AddEntity,
    AddEntityCollection,
    RemoveEntity,
    RemoveEntityCollection,
}

pub struct Event(EventKind, *const EntityCollection);

impl Event {
    fn new(kind: EventKind, collection: &EntityCollection) -> Self {
        Self(kind, collection)
    }

    pub fn kind(&self) -> EventKind {
        self.0
    }

    pub fn entity_collection(&self) -> &EntityCollection {
        unsafe { &*self.1 }
    }
}
