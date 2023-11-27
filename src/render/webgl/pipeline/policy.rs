use std::{cell::RefCell, rc::Rc};

use crate::{entity::Entity, geometry::Geometry, material::Material};

use super::{RenderState, RenderStuff};

/// Material policy telling render program what material should be used of a entity.
pub enum MaterialPolicy<'a, S>
where
    S: RenderStuff,
{
    /// Uses material provides by entity.
    FollowEntity,
    /// Forces all entities render with a specified material.
    Overwrite(Option<Rc<RefCell<dyn Material>>>),
    /// Decides what material to use of each entity by a custom callback function.
    Custom(&'a mut dyn Fn(&RenderState<S>, &Entity) -> Option<Rc<RefCell<dyn Material>>>),
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
pub enum GeometryPolicy<S>
where
    S: RenderStuff,
{
    /// Uses geometry provides by entity.
    FollowEntity,
    /// Forces all entities render a specified geometry.
    Overwrite(Option<Box<dyn Geometry>>),
    /// Decides what geometry to use of each entity by a custom callback function.
    Custom(Box<dyn Fn(&RenderState<S>, &Entity) -> Option<Box<dyn Geometry>>>),
}

pub enum ErrorPolicy {}
