pub mod cube;
pub mod indexed_cube;
pub mod raw;
pub mod rectangle;
pub mod sphere;

use std::any::Any;

use crate::{
    bounding::BoundingVolume,
    event::EventAgency,
    render::webgl::{
        attribute::AttributeValue,
        draw::Draw,
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn bounding_volume(&self) -> Option<BoundingVolume>;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str) -> Option<UniformValue>;

    fn uniform_block_value(&self, name: &str) -> Option<UniformBlockValue>;

    fn changed_event(&self) -> &EventAgency<()>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
