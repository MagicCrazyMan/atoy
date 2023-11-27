pub mod cube;
pub mod indexed_cube;
pub mod raw;
pub mod sphere;
// pub mod plane;

use std::any::Any;

use crate::{
    entity::Entity,
    render::webgl::{attribute::AttributeValue, draw::Draw, uniform::UniformValue},
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str, state: &Entity) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, state: &Entity) -> Option<UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
