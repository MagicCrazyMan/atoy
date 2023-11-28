use std::{cell::RefCell, rc::Rc, collections::HashMap, any::Any};

use gl_matrix4rust::mat4::Mat4;
use uuid::Uuid;

use crate::{
    entity::Entity,
    geometry::Geometry,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        pipeline::RenderState,
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

pub mod environment_mapping;
pub mod solid_color;
pub mod solid_color_instanced;
pub mod texture_mapping;
pub mod texture_mapping_instanced;

pub trait Material {
    fn name(&self) -> &'static str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>];

    fn attribute_value(&self, name: &str, entity: &Rc<RefCell<Entity>>) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &Rc<RefCell<Entity>>) -> Option<UniformValue>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    /// Preparation before entering drawing stage.
    /// 
    /// Depending on [`MaterialPolicy`](crate::render::webgl::pipeline::policy::MaterialPolicy),
    /// `self` is not always extracted from entity. Thus, if you are not sure where the `self` from,
    /// do not borrow material from entity.
    #[allow(unused_variables)]
    fn prepare(&mut self, state: &RenderState, entity: &Rc<RefCell<Entity>>) {}
}
