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

/// A entity node in Scene Graph.
pub struct Entity {
    id: Uuid,
    local_matrix: Mat4,
    normal_matrix: Mat4,
    model_matrix: Mat4,
    model_view_matrix: Mat4,
    model_view_proj_matrix: Mat4,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn WebGLMaterial>>,
    parent: Option<*mut Entity>,
    children: Vec<Box<Entity>>,
}

impl Entity {
    /// Constructs a new empty entity node.
    // pub fn new() -> Self {
    //     Self {
    //         id: Uuid::new_v4(),
    //         name: None,
    //         matrices: EntityMatrices::new(),
    //         geometry: None,
    //         parent: None,
    //         children: Vec::new(),
    //     }
    // }

    /// Constructs a new empty entity node and boxes it.
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self {
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
        })
    }

    /// Constructs a new entity node using [`EntityBuilder`].
    pub fn builder() -> EntityBuilder {
        EntityBuilder::new()
    }
}

impl Entity {
    pub fn id(&self) -> Uuid {
        self.id
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

    pub fn parent_raw(&self) -> Option<*mut Entity> {
        self.parent
    }

    pub fn parent(&self) -> Option<&Self> {
        self.parent.map(|parent| unsafe { &*parent })
    }

    // pub fn parent_mut(&mut self) -> Option<&mut Self> {
    //     match &self.parent {
    //         Some(parent) => unsafe {
    //             let parent = &mut **parent;
    //             Some(parent)
    //         },
    //         None => None,
    //     }
    // }

    pub fn set_parent(self: &mut Box<Self>, parent: Option<*mut Self>) {
        // removes self from original parent
        let self_entity = match &self.parent {
            Some(parent) => unsafe {
                let parent = &mut **parent;
                let index = parent.children.iter().position(|child| child.id == self.id);
                match index {
                    Some(index) => Some(parent.children.remove(index)),
                    None => None,
                }
            },
            None => None,
        };

        // appends self into new parent if parent exists
        match (parent, self_entity) {
            (Some(parent), Some(self_entity)) => unsafe {
                let parent = &mut *parent;
                parent.children.push(self_entity);
            },
            _ => {}
        }

        // sets self's parent
        self.parent = parent
    }

    pub fn children(&self) -> &Vec<Box<Self>> {
        &self.children
    }

    pub(crate) fn children_mut(&mut self) -> &mut Vec<Box<Entity>> {
        &mut self.children
    }

    // pub fn child_by_index(&self, index: usize) -> Option<&Self> {
    //     self.children.get(index).map(|child| child.as_ref())
    // }

    // pub fn add_child(self: &mut Box<Self>, mut child: Self) {
    //     child.parent = Some(self.as_mut());
    //     self.children.push(Box::new(child));
    // }

    pub fn add_child_boxed(self: &mut Box<Self>, mut child: Box<Self>) {
        child.parent = Some(self.as_mut());
        self.children.push(child);
    }

    // pub fn remove_child_by_index(&mut self, index: usize) -> Option<Box<Self>> {
    //     if index > self.children.len() - 1 {
    //         return None;
    //     }

    //     let mut child = self.children.remove(index);
    //     child.parent = None;
    //     Some(child)
    // }

    // pub fn remove_child_by_id(&mut self, id: Uuid) -> Option<Entity> {}

    // fn remove_child
}

impl Entity {
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
        view_matrix: *const Mat4,
        proj_matrix: *const Mat4,
    ) -> Result<(), Error> {
        let (parent_model_matrix, view_matrix, proj_matrix) = unsafe {
            (
                match parent_model_matrix {
                    Some(mat) => Some(&*mat),
                    None => None,
                },
                &*view_matrix,
                &*proj_matrix,
            )
        };

        let model_matrix = match parent_model_matrix {
            Some(parent_model_matrix) => *parent_model_matrix * self.local_matrix,
            None => self.local_matrix,
        };
        let mut normal_matrix = model_matrix.clone();
        normal_matrix.invert()?.transpose();

        self.model_matrix = model_matrix;
        self.normal_matrix = normal_matrix;
        self.model_view_matrix = *view_matrix * self.model_matrix;
        self.model_view_proj_matrix = *proj_matrix * self.model_view_matrix;

        Ok(())
    }
}

pub struct EntityBuilder {
    model_matrix: Mat4,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn WebGLMaterial>>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        EntityBuilder {
            model_matrix: Mat4::new_identity(),
            geometry: None,
            material: None,
        }
    }

    pub fn model_matrix(mut self, mat: Mat4) -> Self {
        self.model_matrix = mat;
        self
    }

    pub fn geometry<G: Geometry + 'static>(mut self, geometry: G) -> Self {
        self.geometry = Some(Box::new(geometry));
        self
    }

    pub fn no_geometry(mut self) -> Self {
        self.geometry = None;
        self
    }

    pub fn material<M: WebGLMaterial + 'static>(mut self, material: M) -> Self {
        self.material = Some(Box::new(material));
        self
    }

    pub fn no_material(mut self) -> Self {
        self.material = None;
        self
    }

    // pub fn build(self) -> Entity {
    //     Entity {
    //         id: Uuid::new_v4(),
    //         m: self.model_matrix,
    //         cn: Mat4::new_identity(),
    //         cm: Mat4::new_identity(),
    //         cmv: Mat4::new_identity(),
    //         cmvp: Mat4::new_identity(),
    //         geometry: self.geometry,
    //         material: self.material,
    //         parent: None,
    //         children: Vec::new(),
    //     }
    // }

    pub fn build_boxed(self) -> Box<Entity> {
        Box::new(Entity {
            id: Uuid::new_v4(),
            local_matrix: self.model_matrix,
            normal_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
            model_view_matrix: Mat4::new_identity(),
            model_view_proj_matrix: Mat4::new_identity(),
            geometry: self.geometry,
            material: self.material,
            parent: None,
            children: Vec::new(),
        })
    }
}

// #[cfg(test)]
// mod test {
//     use gl_matrix4rust::mat4::Mat4;

//     use crate::geometry::cube::Cube;

//     use super::{Entity, EntityBuilder};

//     #[test]
//     fn test_builder() {
//         let entity = EntityBuilder::new().no_geometry().no_name().build();

//         assert_eq!(entity.children.len(), 0);
//         assert_eq!(entity.geometry.is_none(), true);
//         assert_eq!(entity.parent.is_none(), true);
//         assert_eq!(entity.name.is_none(), true);
//         assert_eq!(
//             entity.matrices.model.raw(),
//             &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
//         );
//     }

//     #[test]
//     fn test_builder_with_params() {
//         let root = EntityBuilder::new()
//             .name("Root")
//             .geometry(Cube::new())
//             .model_matrix(Mat4::from_values(
//                 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
//                 15.0,
//             ))
//             .build_boxed();

//         assert_eq!(root.name(), Some("Root"));
//         assert_eq!(root.children.len(), 0);
//         assert_eq!(root.geometry.is_some(), true);
//         assert_eq!(root.parent.is_none(), true);
//         assert_eq!(
//             root.matrices.model.raw(),
//             &[
//                 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
//                 15.0,
//             ]
//         );
//     }

//     #[test]
//     fn test_add_child() {
//         let mut root = Entity::new_boxed();
//         let root_id = root.id();

//         let mut child0 = Entity::new_boxed();
//         let child0_id = child0.id();

//         let child1 = Entity::new_boxed();
//         let child1_id = child1.id();

//         let grandchild0 = Entity::new_boxed();
//         let grandchild0_id = grandchild0.id();

//         let grandchild1 = Entity::new_boxed();
//         let grandchild1_id = grandchild1.id();

//         child0.add_child_boxed(grandchild0);
//         child0.add_child_boxed(grandchild1);
//         root.add_child_boxed(child0);
//         root.add_child_boxed(child1);

//         let child0 = root.child_by_index(0).unwrap();
//         assert_eq!(child0.id(), child0_id);
//         assert_eq!(child0.parent().unwrap().id(), root_id);

//         let child1 = root.child_by_index(1).unwrap();
//         assert_eq!(child1.id(), child1_id);
//         assert_eq!(child1.parent().unwrap().id(), root_id);

//         let grandchild0 = child0.child_by_index(0).unwrap();
//         assert_eq!(grandchild0.id(), grandchild0_id);
//         assert_eq!(grandchild0.parent().unwrap().id(), child0_id);

//         let grandchild1 = child0.child_by_index(1).unwrap();
//         assert_eq!(grandchild1.id(), grandchild1_id);
//         assert_eq!(grandchild1.parent().unwrap().id(), child0_id);
//     }

//     #[test]
//     fn test_remove_child() {
//         let mut root = Entity::new_boxed();
//         let root_id = root.id();

//         let mut child = Entity::new_boxed();
//         let child_id = child.id();

//         let grandchild = Entity::new_boxed();
//         let grandchild_id = grandchild.id();

//         child.add_child_boxed(grandchild);
//         root.add_child_boxed(child);
//     }

//     #[test]
//     fn test_change_parent() {
//         // let root = Entity::new();

//         // let child = Entity
//     }
// }
