use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{entity::Entity, geometry::Geometry, material::Material, render::webgl::RenderGroup};

/// Policies of preparation stage, developer could abort render procedure in this stage.
pub enum PreparationPolicy {
    Abort,
    Continue,
}

/// Material policy telling render program what material should be used of a entity.
pub enum MaterialPolicy {
    /// Uses material provides by entity.
    FollowEntity,
    /// Forces all entities render with a specified material.
    Overwrite(Option<Rc<RefCell<dyn Material>>>),
    /// Decides what material to use of each entity by a custom callback function.
    Custom(
        Box<
            dyn Fn(
                &HashMap<String, RenderGroup>,
                &Rc<RefCell<Entity>>,
            ) -> Option<Rc<RefCell<dyn Material>>>,
        >,
    ),
}

/// Geometry policy telling render program what geometry should be used of a entity.
pub enum GeometryPolicy {
    /// Uses geometry provides by entity.
    FollowEntity,
    /// Forces all entities render a specified geometry.
    Overwrite(Option<Rc<RefCell<dyn Geometry>>>),
    /// Decides what geometry to use of each entity by a custom callback function.
    Custom(
        Box<
            dyn Fn(
                &HashMap<String, RenderGroup>,
                &Rc<RefCell<Entity>>,
            ) -> Option<Rc<RefCell<dyn Geometry>>>,
        >,
    ),
}

/// Entity collect policy tells render program
/// whether an entity should be collect into render list.
pub enum CollectPolicy {
    CollectAll,
    Custom(
        Box<
            dyn Fn(
                &HashMap<String, RenderGroup>,
                &Rc<RefCell<Entity>>,
                &Rc<RefCell<dyn Geometry>>,
                &Rc<RefCell<dyn Material>>,
            ) -> bool,
        >,
    ),
}
