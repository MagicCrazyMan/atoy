pub mod cube;
pub mod indexed_cube;
pub mod raw;
pub mod sphere;
pub mod plane;
// pub mod plane;

use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{
    entity::Entity,
    render::webgl::{attribute::AttributeValue, draw::Draw, uniform::UniformValue, pipeline::RenderState, RenderEntity},
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str, entity: &RenderEntity) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &RenderEntity) -> Option<UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    #[allow(unused_variables)]
    fn prepare(&mut self, state: &RenderState, entity: &Rc<RefCell<Entity>>) {}

    #[allow(unused_variables)]
    fn before_draw(&mut self, state: &RenderState, entity: &RenderEntity) {}

    #[allow(unused_variables)]
    fn after_draw(&mut self, state: &RenderState, entity: &RenderEntity) {}
}
