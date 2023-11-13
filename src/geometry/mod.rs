pub mod cube;

use std::{any::Any, borrow::Cow, collections::HashMap};

use crate::render::webgl::{
    draw::Draw,
    program::{AttributeValue, UniformValue},
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices<'a>(&'a self) -> Option<Cow<'a, AttributeValue>>;

    fn normals<'a>(&'a self) -> Option<Cow<'a, AttributeValue>>;

    fn texture_coordinates<'a>(&'a self) -> Option<Cow<'a, AttributeValue>>;

    fn attribute_values(&self) -> &HashMap<String, AttributeValue>;

    fn uniform_values(&self) -> &HashMap<String, UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
