use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    geometry::Geometry,
    material::Material,
    render::webgl::{attribute::AttributeValue, error::Error, uniform::UniformValue},
};

pub struct Entity {
    id: Uuid,
    local_matrix: Mat4,
    model_matrix: Mat4,
    normal_matrix: Mat4,
    model_view_matrix: Mat4,
    model_view_proj_matrix: Mat4,
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, UniformValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn Material>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            local_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
            normal_matrix: Mat4::new_identity(),
            model_view_matrix: Mat4::new_identity(),
            model_view_proj_matrix: Mat4::new_identity(),
            attributes: HashMap::new(),
            uniforms: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        match &self.geometry {
            Some(geometry) => Some(geometry.as_ref()),
            None => None,
        }
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match &mut self.geometry {
            Some(geometry) => Some(geometry.as_mut()),
            None => None,
        }
    }

    pub fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
        match &mut self.geometry {
            Some(geometry) => Some(geometry.as_mut()),
            None => None,
        }
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        self.geometry = match geometry {
            Some(geometry) => Some(Box::new(geometry)),
            None => None,
        }
    }

    pub fn material(&self) -> Option<&dyn Material> {
        match &self.material {
            Some(material) => Some(material.as_ref()),
            None => None,
        }
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        match &mut self.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn material_raw(&mut self) -> Option<*mut dyn Material> {
        match &mut self.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
        self.material = match material {
            Some(material) => Some(Box::new(material)),
            None => None,
        }
    }

    // pub(crate) fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
    //     match self.geometry.as_mut() {
    //         Some(geometry) => Some(geometry.as_mut()),
    //         None => None,
    //     }
    // }

    // pub(crate) fn material_raw(&mut self) -> Option<*mut dyn Material> {
    //     match self.material.as_mut() {
    //         Some(material) => Some(material.as_mut()),
    //         None => None,
    //     }
    // }

    // pub fn geometry(&self) -> Option<&Rc<RefCell<dyn Geometry>>> {
    //     self.geometry.as_ref()
    // }

    // pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
    //     self.geometry = match geometry {
    //         Some(geometry) => Some(Rc::new(RefCell::new(geometry))),
    //         None => None,
    //     }
    // }

    // pub fn material(&self) -> Option<&Rc<RefCell<dyn Material>>> {
    //     self.material.as_ref()
    // }

    // pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
    //     self.material = match material {
    //         Some(material) => Some(Rc::new(RefCell::new(material))),
    //         None => None,
    //     }
    // }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.attributes
    }

    pub fn attribute_values_mut(&mut self) -> &mut HashMap<String, AttributeValue> {
        &mut self.attributes
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.uniforms
    }

    pub fn uniform_values_mut(&mut self) -> &mut HashMap<String, UniformValue> {
        &mut self.uniforms
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.properties
    }

    pub fn local_matrix(&self) -> &Mat4 {
        &self.local_matrix
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn normal_matrix(&self) -> &Mat4 {
        &self.normal_matrix
    }

    pub fn model_view_matrix(&self) -> &Mat4 {
        &self.model_view_matrix
    }

    pub fn model_view_proj_matrix(&self) -> &Mat4 {
        &self.model_view_proj_matrix
    }

    pub fn set_local_matrix(&mut self, mat: Mat4) {
        self.local_matrix = mat;
    }

    pub(crate) fn update_frame_matrices(
        &mut self,
        parent_model_matrix: &Mat4,
        view_matrix: &Mat4,
        proj_matrix: &Mat4,
    ) -> Result<(), Error> {
        let model_matrix = *parent_model_matrix * self.local_matrix;
        let normal_matrix = model_matrix.invert()?.transpose();

        self.model_matrix = model_matrix;
        self.normal_matrix = normal_matrix;
        self.model_view_matrix = *view_matrix * model_matrix;
        self.model_view_proj_matrix = *proj_matrix * self.model_view_matrix;

        Ok(())
    }
}

pub struct EntityCollection {
    id: Uuid,
    entities: Vec<Rc<RefCell<Entity>>>,
    collections: Vec<EntityCollection>,
    local_matrix: Mat4,
    model_matrix: Mat4,
}

impl EntityCollection {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            entities: Vec::new(),
            // entities_hash: HashMap::new(),
            collections: Vec::new(),
            local_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn entities(&self) -> &[Rc<RefCell<Entity>>] {
        &self.entities
    }

    pub fn entities_mut(&mut self) -> &mut [Rc<RefCell<Entity>>] {
        &mut self.entities
    }

    pub fn add_entity(&mut self, entity: Entity) {
        // self.entities_hash.insert(entity.id, entity.as_mut());
        self.entities.push(Rc::new(RefCell::new(entity)));
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

    pub fn collections(&self) -> &[Self] {
        &self.collections
    }

    pub fn collections_mut(&mut self) -> &mut [Self] {
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
    }

    pub(crate) fn update_frame_matrices(&mut self, parent_model_matrix: &Mat4) {
        self.model_matrix = *parent_model_matrix * self.local_matrix;
    }
}

/// [`Entity`] and associated [`Material`] and [`Geometry`] for rendering.
/// Be aware, geometry and material may not extract from entity,
/// which depending on [`MaterialPolicy`] and [`GeometryPolicy`].
pub struct RenderEntity<'a> {
    entity: Rc<RefCell<Entity>>,
    geometry: *mut dyn Geometry,
    material: *mut dyn Material,
    collected: &'a [Rc<RefCell<Entity>>],
    filtered: &'a [Rc<RefCell<Entity>>],
    filtered_index: usize,
}

impl<'a> RenderEntity<'a> {
    pub(crate) fn new(
        entity: Rc<RefCell<Entity>>,
        geometry: *mut dyn Geometry,
        material: *mut dyn Material,
        collected: &'a [Rc<RefCell<Entity>>],
        filtered: &'a [Rc<RefCell<Entity>>],
        filtered_index: usize,
    ) -> Self {
        Self {
            entity,
            geometry,
            material,
            collected,
            filtered,
            filtered_index,
        }
    }

    #[inline]
    pub fn entity(&self) -> &Rc<RefCell<Entity>> {
        &self.entity
    }

    #[inline]
    pub fn geometry(&self) -> &dyn Geometry {
        unsafe { &*self.geometry }
    }

    #[inline]
    pub fn geometry_raw(&self) -> *mut dyn Geometry {
        self.geometry
    }

    #[inline]
    pub fn geometry_mut(&mut self) -> &mut dyn Geometry {
        unsafe { &mut *self.geometry }
    }

    #[inline]
    pub fn material(&self) -> &dyn Material {
        unsafe { &*self.material }
    }

    #[inline]
    pub fn material_raw(&self) -> *mut dyn Material {
        self.material
    }

    #[inline]
    pub fn material_mut(&mut self) -> &mut dyn Material {
        unsafe { &mut *self.material }
    }

    #[inline]
    pub fn collected(&self) -> &[Rc<RefCell<Entity>>] {
        self.collected
    }

    #[inline]
    pub fn filtered(&self) -> &[Rc<RefCell<Entity>>] {
        self.filtered
    }

    #[inline]
    pub fn filtered_index(&self) -> usize {
        self.filtered_index
    }
}
