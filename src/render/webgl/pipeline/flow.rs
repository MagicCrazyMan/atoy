use crate::{entity::Strong, geometry::Geometry, material::Material};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreparationFlow {
    Continue,
    Abort,
}

#[derive(Clone)]
pub enum BeforeDrawFlow {
    Skip,
    FollowCollectedEntities,
    Custom(Vec<Strong>),
}

#[derive(Clone)]
pub enum BeforeEachDrawFlow {
    Skip,
    FollowEntity,
    OverwriteMaterial(*mut dyn Material),
    OverwriteGeometry(*mut dyn Geometry),
    Overwrite(*mut dyn Geometry, *mut dyn Material),
}
