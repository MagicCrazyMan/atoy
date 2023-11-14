pub mod cube;

use std::any::Any;

use crate::render::webgl::{
    draw::Draw,
    program::{AttributeValue, UniformValue},
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<&AttributeValue>;

    fn normals(&self) -> Option<&AttributeValue>;

    fn texture_coordinates(&self) -> Option<&AttributeValue>;

    fn attribute_value(&self, name: &str) -> Option<&AttributeValue>;

    fn uniform_value(&self, name: &str) -> Option<&UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}


pub trait GeometryA {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<Box<dyn AsRef<[u8]>>>;

    fn normals(&self) -> Option<&AttributeValue>;

    fn texture_coordinates(&self) -> Option<&AttributeValue>;

    fn attribute_value(&self, name: &str) -> Option<&AttributeValue>;

    fn uniform_value(&self, name: &str) -> Option<&UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
