pub mod cube;
// pub mod indexed_cube;
// pub mod plane;
// pub mod sphere;

use std::any::Any;

use crate::render::webgl::{
    draw::Draw,
    program::{AttributeValue, UniformValue},
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices<'a>(&'a self) -> Option<AttributeValue<'a>>;

    fn normals<'a>(&'a self) -> Option<AttributeValue<'a>>;

    fn texture_coordinates<'a>(&'a self) -> Option<AttributeValue<'a>>;

    fn attribute_value<'a>(&'a self, name: &str) -> Option<AttributeValue<'a>>;

    fn uniform_value<'a>(&'a self, name: &str) -> Option<UniformValue<'a>>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
