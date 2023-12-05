use gl_matrix4rust::mat4::Mat4;
use uuid::Uuid;

use super::Entity;

pub struct EntityCollection {
    id: Uuid,
    entities: Vec<Entity>,
    collections: Vec<EntityCollection>,
    update_matrices: bool,
    local_matrix: Mat4,
    model_matrix: Mat4,
}

impl EntityCollection {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            entities: Vec::new(),
            collections: Vec::new(),
            update_matrices: true,
            local_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.entities
    }

    pub fn add_entity(&mut self, entity: Entity) {
        entity.borrow_mut().0.update_matrices = true;
        self.entities.push(entity);
    }

    // pub fn remove_entity_by_index(&mut self, index: usize) -> Option<Rc<RefCell<Entity>>> {
    //     if index >= self.entities.len() {
    //         return None;
    //     }

    //     let entity = self.entities.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(entity)
    // }

    // pub fn remove_entity_by_id(&mut self, id: &Uuid) -> Option<Rc<RefCell<Entity>>> {
    //     let Some(index) = self.entities.iter().position(|entity| entity.borrow().id() == id) else {
    //         return None;
    //     };

    //     let entity = self.entities.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(entity)
    // }

    pub fn collections(&self) -> &Vec<EntityCollection> {
        &self.collections
    }

    pub fn collections_mut(&mut self) -> &mut Vec<EntityCollection> {
        &mut self.collections
    }

    // pub fn add_collection(self: &mut Box<Self>, collection: Self) {
    //     self.collections.push(collection);
    // }

    // pub fn remove_collection_by_index(&mut self, index: usize) -> Option<Self> {
    //     if index >= self.collections.len() {
    //         return None;
    //     }

    //     let collection = self.collections.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(collection)
    // }

    // pub fn remove_collection_by_id(&mut self, id: &Uuid) -> Option<Self> {
    //     let Some(index) = self.collections.iter().position(|entity| &entity.id == id) else {
    //         return None;
    //     };

    //     let collection = self.collections.remove(index);
    //     // self.entities_hash.remove(&entity.id);
    //     Some(collection)
    // }

    pub fn local_matrix(&self) -> &Mat4 {
        &self.local_matrix
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_local_matrix(&mut self, mat: Mat4) {
        self.local_matrix = mat;
        self.update_matrices = true;
    }

    /// Updates matrices of current frame and
    /// returns a boolean value indicating whether matrices changed.
    ///
    /// Only updates matrices when parent model matrix changed
    /// (`parent_model_matrix` is some) or local matrix changed.
    pub(crate) fn update_frame(&mut self, parent_model_matrix: Option<Mat4>) -> bool {
        match parent_model_matrix {
            Some(parent_model_matrix) => {
                self.model_matrix = parent_model_matrix * self.local_matrix;
                self.update_matrices = false;
                true
            }
            None => {
                if self.update_matrices {
                    self.model_matrix = self.local_matrix;
                    self.update_matrices = false;
                    true
                } else {
                    false
                }
            }
        }
    }
}
