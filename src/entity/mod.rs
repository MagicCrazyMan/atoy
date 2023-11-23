use std::{any::Any, collections::HashMap};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;
use wasm_bindgen_test::console_log;

use crate::{
    geometry::Geometry,
    material::Material,
    render::webgl::{
        error::Error,
        program::{AttributeValue, UniformValue},
    },
};

pub struct EntityData {
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
    parent: Option<*mut EntityData>,
    children: Vec<Entity>,
}

impl EntityData {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        match &self.geometry {
            Some(geometry) => Some(geometry.as_ref()),
            None => None,
        }
    }

    pub(crate) fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
        match &mut self.geometry {
            Some(geometry) => Some(geometry.as_mut()),
            None => None,
        }
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
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

    pub(crate) fn material_raw(&mut self) -> Option<*mut dyn Material> {
        match &mut self.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
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

    pub fn attribute_value(&self, name: &str) -> Option<&AttributeValue> {
        self.attributes.get(name)
    }

    pub fn set_attribute_value<K: Into<String>>(&mut self, name: K, value: AttributeValue) {
        self.attributes.insert(name.into(), value);
    }

    pub fn uniform_value(&self, name: &str) -> Option<UniformValue> {
        self.uniforms.get(name).cloned()
    }

    pub fn set_uniform_value<K: Into<String>>(
        &mut self,
        name: K,
        value: UniformValue,
    ) -> Option<UniformValue> {
        self.uniforms.insert(name.into(), value)
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.properties
    }


    pub fn property<'a>(&'a self, key: &str) -> Option<&'a Box<dyn Any>> {
        self.properties.get(key)
    }

    pub fn set_property<K: Into<String>, V: 'static>(&mut self, key: K, value: V) {
        self.properties.insert(key.into(), Box::new(value));
    }

    pub fn parent(&self) -> Option<&EntityData> {
        match self.parent {
            Some(parent) => unsafe { Some(&*parent) },
            None => None,
        }
    }

    pub fn parent_mut(&self) -> Option<&mut EntityData> {
        match self.parent {
            Some(parent) => unsafe { Some(&mut *parent) },
            None => None,
        }
    }

    pub fn add_child(&mut self, mut entity: Entity) {
        entity.0.parent = Some(&mut *self);
        self.children.push(entity);
    }

    pub fn add_children<I: IntoIterator<Item = Entity>>(&mut self, entities: I) {
        for entity in entities {
            self.add_child(entity);
        }
    }

    pub fn remove_child_by_index(&mut self, index: usize) -> Option<Entity> {
        if index > self.children.len() - 1 {
            return None;
        }

        let mut entity = self.children.remove(index);
        entity.0.parent = None;
        Some(entity)
    }

    pub fn remove_child_by_id(&mut self, id: &Uuid) -> Option<Entity> {
        let Some(index) = self.children.iter().position(|entity| &entity.0.id == id) else {
            return None;
        };

        let mut entity = self.children.remove(index);
        entity.0.parent = None;
        Some(entity)
    }

    pub fn child_by_index(&self, index: usize) -> Option<&EntityData> {
        match self.children.get(index) {
            Some(child) => Some(child.0.as_ref()),
            None => None,
        }
    }

    pub fn child_mut_by_index(&mut self, index: usize) -> Option<&mut EntityData> {
        match self.children.get_mut(index) {
            Some(child) => Some(child.0.as_mut()),
            None => None,
        }
    }

    pub fn child_by_id(&self, id: &Uuid) -> Option<&EntityData> {
        match self.children.iter().find(|entity| &entity.0.id == id) {
            Some(child) => Some(child.0.as_ref()),
            None => None,
        }
    }

    pub fn child_mut_by_id(&mut self, id: &Uuid) -> Option<&mut EntityData> {
        match self.children.iter_mut().find(|entity| &entity.0.id == id) {
            Some(child) => Some(child.0.as_mut()),
            None => None,
        }
    }

    pub fn children(&self) -> &[Entity] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Entity] {
        &mut self.children
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
        parent_model_matrix: Option<*const Mat4>,
        view_matrix: &Mat4,
        proj_matrix: &Mat4,
    ) -> Result<(), Error> {
        let (parent_model_matrix, view_matrix, proj_matrix) = unsafe {
            (
                match parent_model_matrix {
                    Some(mat) => Some(&*mat),
                    None => None,
                },
                view_matrix,
                proj_matrix,
            )
        };

        let model_matrix = match parent_model_matrix {
            Some(parent_model_matrix) => *parent_model_matrix * self.local_matrix,
            None => self.local_matrix,
        };
        let normal_matrix = model_matrix.invert()?.transpose();

        self.model_matrix = model_matrix;
        self.normal_matrix = normal_matrix;
        self.model_view_matrix = *view_matrix * self.model_matrix;
        self.model_view_proj_matrix = *proj_matrix * self.model_view_matrix;

        Ok(())
    }
}

pub struct Entity(Box<EntityData>);

impl Entity {
    pub fn new() -> Self {
        Self(Box::new(EntityData {
            id: Uuid::new_v4(),
            local_matrix: Mat4::new_identity(),
            normal_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
            model_view_matrix: Mat4::new_identity(),
            model_view_proj_matrix: Mat4::new_identity(),
            attributes: HashMap::new(),
            uniforms: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
            parent: None,
            children: Vec::new(),
        }))
    }

    pub fn id(&self) -> &Uuid {
        self.0.id()
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.0.geometry()
    }

    pub(crate) fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
        self.0.geometry_raw()
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        self.0.geometry_mut()
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        self.0.set_geometry(geometry)
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.0.material()
    }

    pub(crate) fn material_raw(&mut self) -> Option<*mut dyn Material> {
        self.0.material_raw()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        self.0.material_mut()
    }

    pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
        self.0.set_material(material)
    }

    pub fn attribute_value(&self, name: &str) -> Option<AttributeValue> {
        self.0.attribute_value(name).cloned()
    }

    pub fn set_attribute_value<K: Into<String>>(&mut self, name: K, value: AttributeValue) {
        self.0.set_attribute_value(name, value)
    }

    pub fn uniform_value(&self, name: &str) -> Option<UniformValue> {
        self.0.uniform_value(name)
    }

    pub fn set_uniform_value<K: Into<String>>(
        &mut self,
        name: K,
        value: UniformValue,
    ) -> Option<UniformValue> {
        self.0.set_uniform_value(name, value)
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        self.0.properties()
    }

    pub fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        self.0.properties_mut()
    }

    pub fn property<'a>(&'a self, key: &str) -> Option<&'a Box<dyn Any>> {
        self.0.property(key)
    }

    pub fn set_property<K: Into<String>, V: 'static>(&mut self, key: K, value: V) {
        self.0.set_property(key, value)
    }

    pub fn parent(&self) -> Option<&EntityData> {
        self.0.parent()
    }

    pub fn parent_mut(&self) -> Option<&mut EntityData> {
        self.0.parent_mut()
    }

    pub fn add_child(&mut self, entity: Entity) {
        self.0.add_child(entity)
    }

    pub fn add_children<I: IntoIterator<Item = Entity>>(&mut self, entities: I) {
        self.0.add_children(entities)
    }

    pub fn remove_child_by_index(&mut self, index: usize) -> Option<Entity> {
        self.0.remove_child_by_index(index)
    }

    pub fn remove_child_by_id(&mut self, id: &Uuid) -> Option<Entity> {
        self.0.remove_child_by_id(id)
    }

    pub fn child_by_index(&self, index: usize) -> Option<&EntityData> {
        self.0.child_by_index(index)
    }

    pub fn child_mut_by_index(&mut self, index: usize) -> Option<&mut EntityData> {
        self.0.child_mut_by_index(index)
    }

    pub fn child_by_id(&self, id: &Uuid) -> Option<&EntityData> {
        self.0.child_by_id(id)
    }

    pub fn child_mut_by_id(&mut self, id: &Uuid) -> Option<&mut EntityData> {
        self.0.child_mut_by_id(id)
    }

    pub fn children(&self) -> &[Entity] {
        self.0.children()
    }

    pub fn children_mut(&mut self) -> &mut [Entity] {
        self.0.children_mut()
    }

    pub fn local_matrix(&self) -> &Mat4 {
        self.0.local_matrix()
    }

    pub fn model_matrix(&self) -> &Mat4 {
        self.0.model_matrix()
    }

    pub fn normal_matrix(&self) -> &Mat4 {
        self.0.normal_matrix()
    }

    pub fn model_view_matrix(&self) -> &Mat4 {
        self.0.model_view_matrix()
    }

    pub fn model_view_proj_matrix(&self) -> &Mat4 {
        self.0.model_view_proj_matrix()
    }

    pub fn set_local_matrix(&mut self, mat: Mat4) {
        self.0.set_local_matrix(mat)
    }

    pub(crate) fn update_frame_matrices(
        &mut self,
        parent_model_matrix: Option<*const Mat4>,
        view_matrix: &Mat4,
        proj_matrix: &Mat4,
    ) -> Result<(), Error> {
        self.0
            .update_frame_matrices(parent_model_matrix, view_matrix, proj_matrix)
    }
}

// pub struct EntityBuilder {
//     model_matrix: Mat4,
//     geometry: Option<Box<dyn Geometry>>,
//     material: Option<Box<dyn WebGLMaterial>>,
// }

// impl EntityBuilder {
//     pub fn new() -> Self {
//         EntityBuilder {
//             model_matrix: Mat4::new_identity(),
//             geometry: None,
//             material: None,
//         }
//     }

//     pub fn model_matrix(mut self, mat: Mat4) -> Self {
//         self.model_matrix = mat;
//         self
//     }

//     pub fn geometry<G: Geometry + 'static>(mut self, geometry: G) -> Self {
//         self.geometry = Some(Box::new(geometry));
//         self
//     }

//     pub fn no_geometry(mut self) -> Self {
//         self.geometry = None;
//         self
//     }

//     pub fn material<M: WebGLMaterial + 'static>(mut self, material: M) -> Self {
//         self.material = Some(Box::new(material));
//         self
//     }

//     pub fn no_material(mut self) -> Self {
//         self.material = None;
//         self
//     }

//     // pub fn build(self) -> Entity {
//     //     Entity {
//     //         id: Uuid::new_v4(),
//     //         m: self.model_matrix,
//     //         cn: Mat4::new_identity(),
//     //         cm: Mat4::new_identity(),
//     //         cmv: Mat4::new_identity(),
//     //         cmvp: Mat4::new_identity(),
//     //         geometry: self.geometry,
//     //         material: self.material,
//     //         parent: None,
//     //         children: Vec::new(),
//     //     }
//     // }

//     pub fn build_boxed(self) -> Box<Entity> {
//         Box::new(Entity {
//             id: Uuid::new_v4(),
//             local_matrix: self.model_matrix,
//             normal_matrix: Mat4::new_identity(),
//             model_matrix: Mat4::new_identity(),
//             model_view_matrix: Mat4::new_identity(),
//             model_view_proj_matrix: Mat4::new_identity(),
//             geometry: self.geometry,
//             material: self.material,
//             parent: None,
//             children: Vec::new(),
//         })
//     }
// }
