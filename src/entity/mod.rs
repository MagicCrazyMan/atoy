use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    geometry::Geometry,
    material::WebGLMaterial,
    render::webgl::{
        error::Error,
        program::{AttributeValue, UniformValue},
    },
};

pub struct Entity {
    id: Uuid,
    local_matrix: Mat4,
    model_matrix: Mat4,
    normal_matrix: Mat4,
    model_view_matrix: Mat4,
    model_view_proj_matrix: Mat4,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn WebGLMaterial>>,
    parent: Option<*mut Entity>,
    children: Vec<EntityNode>,
}

impl Entity {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        match &self.geometry {
            Some(geometry) => Some(geometry.as_ref()),
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

    pub fn material(&self) -> Option<&dyn WebGLMaterial> {
        match &self.material {
            Some(material) => Some(material.as_ref()),
            None => None,
        }
    }

    pub fn material_raw(&mut self) -> Option<*mut dyn WebGLMaterial> {
        match &mut self.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn WebGLMaterial> {
        match &mut self.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn set_material<M: WebGLMaterial + 'static>(&mut self, material: Option<M>) {
        self.material = match material {
            Some(material) => Some(Box::new(material)),
            None => None,
        }
    }

    #[allow(unused_variables)]
    pub fn attribute_value<'a>(&self, name: &str) -> Option<AttributeValue<'a>> {
        None
    }

    #[allow(unused_variables)]
    pub fn uniform_value<'a>(&self, name: &str) -> Option<UniformValue<'a>> {
        None
    }

    // pub fn parent_raw(&self) -> Option<*mut EntityNode> {
    //     self.parent
    // }

    pub fn parent(&self) -> Option<&Entity> {
        match self.parent {
            Some(parent) => unsafe { Some(&*parent) },
            None => None,
        }
    }

    pub fn parent_mut(&self) -> Option<&mut Entity> {
        match self.parent {
            Some(parent) => unsafe { Some(&mut *parent) },
            None => None,
        }
    }

    pub fn add_child_boxed(&mut self, mut entity: EntityNode) {
        entity.0.parent = Some(&mut *self);
        self.children.push(entity);
    }

    pub fn add_children_boxed<I: IntoIterator<Item = EntityNode>>(&mut self, entities: I) {
        for entity in entities {
            self.add_child_boxed(entity);
        }
    }

    pub fn remove_child_by_index(&mut self, index: usize) -> Option<EntityNode> {
        if index > self.children.len() - 1 {
            return None;
        }

        let mut entity = self.children.remove(index);
        entity.0.parent = None;
        Some(entity)
    }

    pub fn remove_child_by_id(&mut self, id: &Uuid) -> Option<EntityNode> {
        let Some(index) = self.children.iter().position(|entity| &entity.0.id == id) else {
            return None;
        };

        let mut entity = self.children.remove(index);
        entity.0.parent = None;
        Some(entity)
    }

    pub fn child_by_index(&self, index: usize) -> Option<&Entity> {
        match self.children.get(index) {
            Some(child) => Some(child.0.as_ref()),
            None => None,
        }
    }

    pub fn child_mut_by_index(&mut self, index: usize) -> Option<&mut Entity> {
        match self.children.get_mut(index) {
            Some(child) => Some(child.0.as_mut()),
            None => None,
        }
    }

    pub fn child_by_id(&self, id: &Uuid) -> Option<&Entity> {
        match self.children.iter().find(|entity| &entity.0.id == id) {
            Some(child) => Some(child.0.as_ref()),
            None => None,
        }
    }

    pub fn child_mut_by_id(&mut self, id: &Uuid) -> Option<&mut Entity> {
        match self.children.iter_mut().find(|entity| &entity.0.id == id) {
            Some(child) => Some(child.0.as_mut()),
            None => None,
        }
    }

    pub fn children(&self) -> &[EntityNode] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [EntityNode] {
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

pub struct EntityNode(Box<Entity>);

impl EntityNode {
    pub fn new() -> Self {
        Self(Box::new(Entity {
            id: Uuid::new_v4(),
            local_matrix: Mat4::new_identity(),
            normal_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
            model_view_matrix: Mat4::new_identity(),
            model_view_proj_matrix: Mat4::new_identity(),
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

    pub fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
        self.0.geometry_raw()
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        self.0.set_geometry(geometry)
    }

    pub fn material(&self) -> Option<&dyn WebGLMaterial> {
        self.0.material()
    }

    pub fn material_raw(&mut self) -> Option<*mut dyn WebGLMaterial> {
        self.0.material_raw()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn WebGLMaterial> {
        self.0.material_mut()
    }

    pub fn set_material<M: WebGLMaterial + 'static>(&mut self, material: Option<M>) {
        self.0.set_material(material)
    }

    #[allow(unused_variables)]
    pub fn attribute_value<'a>(&self, name: &str) -> Option<AttributeValue<'a>> {
        self.0.attribute_value(name)
    }

    #[allow(unused_variables)]
    pub fn uniform_value<'a>(&self, name: &str) -> Option<UniformValue<'a>> {
        self.0.uniform_value(name)
    }

    // pub fn parent_raw(&self) -> Option<*mut EntityNode> {
    //     self.parent
    // }

    pub fn parent(&self) -> Option<&Entity> {
        self.0.parent()
    }

    pub fn parent_mut(&self) -> Option<&mut Entity> {
        self.0.parent_mut()
    }

    pub fn add_child_boxed(&mut self, mut entity: EntityNode) {
        self.0.add_child_boxed(entity)
    }

    pub fn add_children_boxed<I: IntoIterator<Item = EntityNode>>(&mut self, entities: I) {
        self.0.add_children_boxed(entities)
    }

    pub fn remove_child_by_index(&mut self, index: usize) -> Option<EntityNode> {
        self.0.remove_child_by_index(index)
    }

    pub fn remove_child_by_id(&mut self, id: &Uuid) -> Option<EntityNode> {
        self.0.remove_child_by_id(id)
    }

    pub fn child_by_index(&self, index: usize) -> Option<&Entity> {
        self.0.child_by_index(index)
    }

    pub fn child_mut_by_index(&mut self, index: usize) -> Option<&mut Entity> {
        self.0.child_mut_by_index(index)
    }

    pub fn child_by_id(&self, id: &Uuid) -> Option<&Entity> {
        self.0.child_by_id(id)
    }

    pub fn child_mut_by_id(&mut self, id: &Uuid) -> Option<&mut Entity> {
        self.0.child_mut_by_id(id)
    }

    pub fn children(&self) -> &[EntityNode] {
        self.0.children()
    }

    pub fn children_mut(&mut self) -> &mut [EntityNode] {
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
