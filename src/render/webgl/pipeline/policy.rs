use crate::{entity::Entity, geometry::Geometry, material::Material};

/// Material policy telling render program what material should be used of a entity.
pub enum MaterialPolicy {
    /// Uses material provides by entity.
    FollowEntity,
    /// Forces all entities render with a specified material.
    Overwrite(Option<Box<dyn Material>>),
    /// Decides what material to use of each entity by a custom callback function.
    Custom(Box<dyn Fn(&Entity) -> Option<Box<dyn Material>>>),
}

// pub trait MP {
//     fn name(&self) -> &str;

//     fn policy<S>(
//         &mut self,
//         state: &mut RenderState<S>,
//         entity: &mut Entity,
//     ) -> Option<&mut dyn Material>
//     where
//         S: RenderStuff;
// }

/// Geometry policy telling render program what geometry should be used of a entity.
pub enum GeometryPolicy {
    /// Uses geometry provides by entity.
    FollowEntity,
    /// Forces all entities render a specified geometry.
    Overwrite(Option<Box<dyn Geometry>>),
    /// Decides what geometry to use of each entity by a custom callback function.
    Custom(Box<dyn Fn(&Entity) -> Option<Box<dyn Geometry>>>),
}

pub enum ErrorPolicy {}
