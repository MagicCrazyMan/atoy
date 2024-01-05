use std::ptr::NonNull;

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    event::EventAgency,
};

use super::{Entity, EntityRaw};

/// An [`Entity`] and `EntityCollection` container.
/// Model matrix of an  `EntityCollection` effects all entities and sub-collections.
pub struct EntityCollection {
    id: Uuid,
    entities: Vec<Entity>,
    collections: Vec<EntityCollection>,
    model_matrix: Mat4,
    event: EventAgency<Event>,
    dirty: bool,
    collection: *const EntityCollection,
    bounding: Option<CullingBoundingVolume>,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
    enable_bounding: bool,
}

impl EntityCollection {
    /// Constructs a new entity collection.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            entities: Vec::new(),
            collections: Vec::new(),
            model_matrix: Mat4::new_identity(),
            event: EventAgency::new(),
            dirty: true,
            bounding: None,
            collection: std::ptr::null(),
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
            enable_bounding: true,
        }
    }

    /// Returns collection id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns `true` if this collection enable bounding volume.
    pub fn bounding_enabled(&self) -> bool {
        self.enable_bounding
    }

    /// Enables bounding volume for this collection.
    pub fn enable_bounding(&mut self) {
        self.enable_bounding = true;
        self.dirty = true;
    }

    /// Disables bounding volume for this collection.
    pub fn disable_bounding(&mut self) {
        self.enable_bounding = false;
        self.dirty = true;
    }

    /// Returns culling bounding volume.
    pub fn bounding(&mut self) -> Option<&mut CullingBoundingVolume> {
        self.bounding.as_mut()
    }

    /// Returns event agency of this collection.
    pub fn event(&self) -> &EventAgency<Event> {
        &self.event
    }

    /// Returns entities in this collection.
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Returns mutable entities in this collection.
    pub fn entities_mut(&mut self) -> &mut [Entity] {
        &mut self.entities
    }

    /// Adds a new entity to this collection.
    pub fn add_entity(&mut self, entity_raw: EntityRaw) {
        self.entities.push(Entity::new(entity_raw, self));

        let index = self.entities.len() - 1;
        self.event.raise(Event::AddEntity(unsafe {
            NonNull::new_unchecked(self.entities.get_mut(index).unwrap())
        }));
    }

    /// Returns an entity from this collection by index.
    pub fn get_entity_by_index(&mut self, index: usize) -> Option<&Entity> {
        self.entities.get(index)
    }

    /// Returns a mutable entity from this collection by index.
    pub fn get_mut_entity_by_index(&mut self, index: usize) -> Option<&mut Entity> {
        self.entities.get_mut(index)
    }

    /// Removes an entity from this collection by index.
    pub fn remove_entity_by_index(&mut self, index: usize) -> Option<EntityRaw> {
        if index > self.entities.len() - 1 {
            return None;
        }

        let mut entity_raw = self.entities.remove(index).raw;
        self.event.raise(Event::RemoveEntity(unsafe {
            NonNull::new_unchecked(&mut entity_raw)
        }));
        // self.entities_hash.remove(&entity.id);
        Some(entity_raw)
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
        &self.collections
    }

    /// Returns mutable sub-collections in this collection.
    pub fn collections_mut(&mut self) -> &mut [Self] {
        &mut self.collections
    }

    /// Adds a new sub-collection to this collection.
    pub fn add_collection(&mut self, mut collection: Self) {
        collection.collection = self;
        collection.dirty = true;
        self.collections.push(collection);

        let index = self.collections.len() - 1;
        self.event.raise(Event::AddCollection(unsafe {
            NonNull::new_unchecked(self.collections.get_mut(index).unwrap())
        }));
    }

    /// Returns a sub-collection from this collection by index.
    pub fn get_collection_by_index(&mut self, index: usize) -> Option<&EntityCollection> {
        self.collections.get(index)
    }

    /// Returns a mutable sub-collection from this collection by index.
    pub fn get_mut_collection_by_index(&mut self, index: usize) -> Option<&mut EntityCollection> {
        self.collections.get_mut(index)
    }

    /// Removes a sub-collection from this collection by index.
    pub fn remove_collection_by_index(&mut self, index: usize) -> Option<Self> {
        if index > self.collections.len() - 1 {
            return None;
        }

        let mut collection = self.collections.remove(index);
        collection.collection = std::ptr::null();
        collection.dirty = true;
        self.event.raise(Event::RemoveCollection(unsafe {
            NonNull::new_unchecked(&mut collection)
        }));
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
        &self.model_matrix
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    /// Sets model matrix for this collection.
    pub fn set_model_matrix(&mut self, mat: Mat4) {
        self.model_matrix = mat;
        self.dirty = true;
        self.event.raise(Event::SetModelMatrix(unsafe {
            NonNull::new_unchecked(&mut self.model_matrix)
        }))
    }

    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        self.update_matrix();
        self.update_bounding();
        self.dirty = false;
    }

    fn update_matrix(&mut self) {
        if self.collection.is_null() {
            self.compose_model_matrix = self.model_matrix;
        } else {
            unsafe {
                self.compose_model_matrix =
                    *(*self.collection).compose_model_matrix() * self.model_matrix;
            }
        }
        self.compose_normal_matrix = self
            .compose_model_matrix
            .invert()
            .expect("matrix with zero determinant is not allowed")
            .transpose();
    }

    /// Creates a large bounding volume that contains all entities in this collection.
    fn update_bounding(&mut self) {
        if !self.enable_bounding {
            return;
        }

        let collection_boundings = self.collections.iter().filter_map(|collection| {
            collection
                .bounding
                .as_ref()
                .map(|bounding| bounding.bounding())
        });
        let entity_boundings = self
            .entities
            .iter()
            .filter_map(|entity| entity.bounding().map(|bounding| bounding.bounding()));
        let boundings = collection_boundings.chain(entity_boundings);
        self.bounding =
            merge_bounding_volumes(boundings).map(|bounding| CullingBoundingVolume::new(bounding));
    }
}

#[derive(Clone)]
pub enum Event {
    SetModelMatrix(NonNull<Mat4>),
    AddEntity(NonNull<Entity>),
    AddCollection(NonNull<EntityCollection>),
    RemoveEntity(NonNull<EntityRaw>),
    RemoveCollection(NonNull<EntityCollection>),
}
