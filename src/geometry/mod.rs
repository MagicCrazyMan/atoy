pub mod cube;
pub mod indexed_cube;
pub mod plane;

use std::any::Any;

use crate::{
    ncor::Ncor,
    render::webgl::{
        draw::Draw,
        program::{AttributeValue, UniformValue},
    },
};

pub trait Geometry {
    fn draw<'a>(&'a self) -> Draw<'a>;

    fn vertices<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>>;

    fn normals<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>>;

    fn texture_coordinates<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>>;

    fn attribute_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, AttributeValue>>;

    fn uniform_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, UniformValue>>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
