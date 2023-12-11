pub mod collection;

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    bounding::BoundingVolume,
    geometry::Geometry,
    material::Material,
    render::webgl::{
        attribute::AttributeValue,
        error::Error,
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub struct Borrowed<'a>(Ref<'a, Inner>);

impl<'a> Borrowed<'a> {
    pub fn id(&self) -> &Uuid {
        &self.0.id
    }

    pub fn bounding_volume(&self) -> Option<&BoundingVolume> {
        self.0.bounding_volume.as_ref()
    }

    pub fn local_matrix(&self) -> &Mat4 {
        &self.0.local_matrix
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.0.model_matrix
    }

    pub fn normal_matrix(&self) -> &Mat4 {
        &self.0.normal_matrix
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.0.geometry.as_ref().map(|geom| geom.as_ref())
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.0.material.as_ref().map(|geom| geom.as_ref())
    }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.0.attributes
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.0.uniforms
    }

    pub fn uniform_block_values(&self) -> &HashMap<String, UniformBlockValue> {
        &self.0.uniform_blocks
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.0.properties
    }
}

pub struct BorrowedMut<'a>(RefMut<'a, Inner>);

impl<'a> BorrowedMut<'a> {
    pub fn id(&self) -> &Uuid {
        &self.0.id
    }

    pub fn bounding_volume(&self) -> Option<&BoundingVolume> {
        self.0.bounding_volume.as_ref()
    }

    pub fn local_matrix(&self) -> &Mat4 {
        &self.0.local_matrix
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.0.model_matrix
    }

    pub fn normal_matrix(&self) -> &Mat4 {
        &self.0.normal_matrix
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.0.geometry.as_ref().map(|geom| geom.as_ref())
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        match geometry {
            Some(geometry) => self.0.geometry = Some(Box::new(geometry)),
            None => self.0.geometry = None,
        };
    }

    pub(crate) fn geometry_raw(&mut self) -> Option<*mut dyn Geometry> {
        self.0
            .geometry
            .as_mut()
            .map(|geom| geom.as_mut() as *mut dyn Geometry)
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.0.material.as_ref().map(|geom| geom.as_ref())
    }

    pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
        match material {
            Some(material) => self.0.material = Some(Box::new(material)),
            None => self.0.material = None,
        };
    }

    pub(crate) fn material_raw(&mut self) -> Option<*mut dyn Material> {
        self.0
            .material
            .as_mut()
            .map(|material| material.as_mut() as *mut dyn Material)
    }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.0.attributes
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.0.uniforms
    }

    pub fn uniform_block_values(&self) -> &HashMap<String, UniformBlockValue> {
        &self.0.uniform_blocks
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.0.properties
    }

    pub fn set_local_matrix(&mut self, local_matrix: Mat4) {
        self.0.local_matrix = local_matrix;
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match &mut self.0.geometry {
            Some(geometry) => Some(geometry.as_mut()),
            None => None,
        }
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        match &mut self.0.material {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn attribute_values_mut(&mut self) -> &mut HashMap<String, AttributeValue> {
        &mut self.0.attributes
    }

    pub fn uniform_values_mut(&mut self) -> &mut HashMap<String, UniformValue> {
        &mut self.0.uniforms
    }

    pub fn uniform_block_values_mut(&mut self) -> &mut HashMap<String, UniformBlockValue> {
        &mut self.0.uniform_blocks
    }

    pub fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.0.properties
    }

    /// Updates matrices of current frame.
    /// Only updates matrices when parent model matrix changed
    /// (`parent_model_matrix` is some) or local matrix changed.
    pub(crate) fn update_frame(&mut self, parent_model_matrix: Option<Mat4>) -> Result<(), Error> {
        enum Status {
            ModelMatrixChanged(Mat4),
            BoundingVolumeChanged,
            Unchanged,
        }

        let status = match parent_model_matrix {
            Some(parent_model_matrix) => {
                Status::ModelMatrixChanged(parent_model_matrix * self.0.local_matrix)
            }
            None => {
                let update_matrices = self.0.update_matrices
                    || self
                        .0
                        .geometry
                        .as_ref()
                        .map(|geom| geom.update_matrices())
                        .unwrap_or(false)
                    || self
                        .0
                        .material
                        .as_ref()
                        .map(|m| m.update_matrices())
                        .unwrap_or(false);
                let update_bounding_volume: bool = self.0.update_bounding_volume
                    || self
                        .0
                        .geometry
                        .as_ref()
                        .map(|geom| geom.update_bounding_volume())
                        .unwrap_or(false)
                    || self
                        .0
                        .material
                        .as_ref()
                        .map(|m| m.update_bounding_volume())
                        .unwrap_or(false);

                if update_matrices {
                    // no parent model matrix, use local matrix as model matrix
                    Status::ModelMatrixChanged(self.0.local_matrix)
                } else if update_bounding_volume {
                    Status::BoundingVolumeChanged
                } else {
                    Status::Unchanged
                }
            }
        };

        match status {
            Status::ModelMatrixChanged(model_matrix) => {
                let normal_matrix = model_matrix.invert()?.transpose();
                self.0.model_matrix = model_matrix;
                self.0.normal_matrix = normal_matrix;
                self.0.bounding_volume = self
                    .0
                    .geometry
                    .as_ref()
                    .and_then(|geom| geom.bounding_volume_native())
                    .map(|bounding| bounding.transform(&self.0.model_matrix))
                    .map(|kind| BoundingVolume::new(kind));
            }
            Status::BoundingVolumeChanged => {
                self.0.bounding_volume = self
                    .0
                    .geometry
                    .as_ref()
                    .and_then(|geom| geom.bounding_volume_native())
                    .map(|bounding| bounding.transform(&self.0.model_matrix))
                    .map(|kind| BoundingVolume::new(kind));
            }
            Status::Unchanged => {}
        };

        self.0.update_matrices = false;
        self.0.update_bounding_volume = false;
        self.0
            .geometry
            .as_mut()
            .map(|geom| geom.set_update_matrices(false));
        self.0
            .geometry
            .as_mut()
            .map(|geom| geom.set_update_bounding_volume(false));
        self.0
            .material
            .as_mut()
            .map(|material| material.set_update_matrices(false));
        self.0
            .material
            .as_mut()
            .map(|material| material.set_update_bounding_volume(false));

        Ok(())
    }
}

#[derive(Clone)]
pub struct Weak(std::rc::Weak<RefCell<Inner>>);

impl Weak {
    pub fn upgrade(&self) -> Option<Strong> {
        self.0.upgrade().map(|entity| Strong(entity))
    }
}

#[derive(Clone)]
pub struct Strong(Rc<RefCell<Inner>>);

impl Strong {
    pub fn borrow(&self) -> Borrowed {
        Borrowed(self.0.borrow())
    }

    pub fn borrow_mut(&self) -> BorrowedMut {
        BorrowedMut(self.0.borrow_mut())
    }

    pub fn downgrade(&self) -> Weak {
        Weak(Rc::downgrade(&self.0))
    }

    pub fn try_own(self) -> Result<Entity, Self> {
        if Rc::strong_count(&self.0) == 1 {
            Ok(Entity(self.0))
        } else {
            Err(self)
        }
    }
}

struct Inner {
    id: Uuid,
    update_matrices: bool,
    update_bounding_volume: bool,
    bounding_volume: Option<BoundingVolume>,
    local_matrix: Mat4,
    model_matrix: Mat4,
    normal_matrix: Mat4,
    // fields below are all sharable values
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, UniformValue>,
    uniform_blocks: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn Material>>,
}

pub struct Entity(Rc<RefCell<Inner>>);

impl Entity {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Inner {
            id: Uuid::new_v4(),
            update_matrices: true,
            update_bounding_volume: true,
            bounding_volume: None,
            local_matrix: Mat4::new_identity(),
            model_matrix: Mat4::new_identity(),
            normal_matrix: Mat4::new_identity(),
            attributes: HashMap::new(),
            uniforms: HashMap::new(),
            uniform_blocks: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
        })))
    }

    pub fn borrow(&self) -> Borrowed {
        Borrowed(self.0.borrow())
    }

    pub fn borrow_mut(&self) -> BorrowedMut {
        BorrowedMut(self.0.borrow_mut())
    }

    pub fn weak(&self) -> Weak {
        Weak(Rc::downgrade(&self.0))
    }

    pub fn strong(&self) -> Strong {
        Strong(Rc::clone(&self.0))
    }
}

// /// [`Entity`] and associated [`Material`] and [`Geometry`] for rendering.
// /// Be aware, geometry and material may not extract from entity,
// /// which depending on [`MaterialPolicy`] and [`GeometryPolicy`].
// pub struct RenderEntity<'a> {
//     entity: Strong,
//     geometry: *mut dyn Geometry,
//     material: *mut dyn Material,
//     collected: &'a [Strong],
//     drawings: &'a [Strong],
//     drawing_index: usize,
// }

// impl<'a> RenderEntity<'a> {
//     pub(crate) fn new(
//         entity: Strong,
//         geometry: *mut dyn Geometry,
//         material: *mut dyn Material,
//         collected: &'a [Strong],
//         drawings: &'a [Strong],
//         drawing_index: usize,
//     ) -> Self {
//         Self {
//             entity,
//             geometry,
//             material,
//             collected,
//             drawings,
//             drawing_index,
//         }
//     }

//     #[inline]
//     pub fn entity(&self) -> &Strong {
//         &self.entity
//     }

//     #[inline]
//     pub fn geometry(&self) -> &dyn Geometry {
//         unsafe { &*self.geometry }
//     }

//     #[inline]
//     pub fn geometry_raw(&self) -> *mut dyn Geometry {
//         self.geometry
//     }

//     #[inline]
//     pub fn geometry_mut(&mut self) -> &mut dyn Geometry {
//         unsafe { &mut *self.geometry }
//     }

//     #[inline]
//     pub fn material(&self) -> &dyn Material {
//         unsafe { &*self.material }
//     }

//     #[inline]
//     pub fn material_raw(&self) -> *mut dyn Material {
//         self.material
//     }

//     #[inline]
//     pub fn material_mut(&mut self) -> &mut dyn Material {
//         unsafe { &mut *self.material }
//     }

//     #[inline]
//     pub fn collected_entities(&self) -> &[Strong] {
//         self.collected
//     }

//     #[inline]
//     pub fn drawing_entities(&self) -> &[Strong] {
//         self.drawings
//     }

//     #[inline]
//     pub fn drawing_index(&self) -> usize {
//         self.drawing_index
//     }
// }
