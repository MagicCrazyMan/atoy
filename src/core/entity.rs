use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use gl_matrix4rust::mat4::Mat4;
use uuid::Uuid;

use super::{
    bounding::BoundingVolume, operator::entity_collector::EntityFiltering,
    transparency::EntityTransparency, versioning::Versioning, web::webgl::entity::WebGlEntity,
    AsAny,
};

pub trait Component: AsAny {
    // fn property(&self, name: &str) -> Option<&dyn Any>;
}

pub trait Entity: EntityMatrices + EntityBoundingVolume + Versioning + AsAny {
    fn as_filtering(&self) -> Option<&dyn EntityFiltering>;

    fn as_transparency(&self) -> Option<&dyn EntityTransparency>;

    fn as_webgl(&self) -> Option<&dyn WebGlEntity>;

    fn as_webgl_mut(&mut self) -> Option<&mut dyn WebGlEntity>;

    // fn components(&self) -> &[&dyn Component];

    // fn components_mut(&self) -> &[&mut dyn Component];
}

pub trait EntityMatrices {
    fn local_matrix(&self) -> &Mat4<f64>;

    // fn model_matrix(&self) -> &Mat4<f64>;

    // fn set_model_matrix(&mut self, mat: Mat4<f64>);
}

pub trait EntityBoundingVolume {
    fn bounding_volume(&self) -> &BoundingVolume;
}

pub type EntityShared = Rc<RefCell<dyn Entity>>;
pub type EntitySharedWeak = Weak<RefCell<dyn Entity>>;

pub struct Collection {
    id: Uuid,
    entities: Vec<EntityShared>,
    version: usize,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            entities: Vec::new(),
            version: 0,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn entities(&self) -> &Vec<EntityShared> {
        &self.entities
    }
}

impl Versioning for Collection {
    fn version(&self) -> usize {
        self.version
    }

    fn set_version(&mut self, version: usize) {
        self.version = version;
    }
}
