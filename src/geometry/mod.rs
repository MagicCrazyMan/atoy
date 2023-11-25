pub mod cube;
pub mod indexed_cube;
pub mod sphere;
pub mod raw;
// pub mod plane;

use std::any::Any;

use crate::render::webgl::{attribute::AttributeValue, draw::Draw, uniform::UniformValue, EntityRenderState};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str, state: &EntityRenderState) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, state: &EntityRenderState) -> Option<UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
