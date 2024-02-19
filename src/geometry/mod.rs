pub mod cube;
pub mod indexed_cube;
pub mod raw;
pub mod rectangle;
pub mod sphere;

use std::any::Any;

use crate::{
    bounding::BoundingVolume,
    readonly::Readonly,
    renderer::webgl::{
        attribute::AttributeValue,
        draw::{CullFace, Draw},
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn cull_face(&self) -> Option<CullFace>;

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>>;

    fn positions(&self) -> Readonly<'_, AttributeValue>;

    fn normals(&self) -> Option<Readonly<'_, AttributeValue>>;

    fn tangents(&self) -> Option<Readonly<'_, AttributeValue>>;
   
    fn bitangents(&self) -> Option<Readonly<'_, AttributeValue>>;

    fn texture_coordinates(&self) -> Option<Readonly<'_, AttributeValue>>;

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>>;

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>>;

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
