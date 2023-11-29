use std::{cell::RefCell, rc::Rc};

use crate::{entity::Entity, geometry::Geometry, material::Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreparationFlow {
    Continue,
    Abort,
}

#[derive(Clone)]
pub enum BeforeDrawFlow {
    Skip,
    FollowCollectedEntities,
    Custom(Vec<Rc<RefCell<Entity>>>),
}

#[derive(Clone)]
pub enum BeforeEachDrawFlow {
    Skip,
    FollowEntity,
    OverwriteMaterial(*mut dyn Material),
    OverwriteGeometry(*mut dyn Geometry),
    Overwrite(*mut dyn Geometry, *mut dyn Material),
}
