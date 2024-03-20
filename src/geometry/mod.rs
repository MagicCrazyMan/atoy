pub mod cube;
pub mod indexed_cube;
pub mod rectangle;
pub mod sphere;

use std::any::Any;

use crate::{
    bounding::BoundingVolume,
    clock::Tick,
    message::Receiver,
    renderer::webgl::{
        attribute::AttributeValue,
        draw::{CullFace, Draw},
        uniform::{UniformBlockValue, UniformValue},
    },
    value::Readonly,
};

pub trait Geometry {
    fn draw(&self) -> Draw<'_>;

    fn cull_face(&self) -> Option<CullFace>;

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>>;

    fn positions(&self) -> AttributeValue<'_>;

    fn normals(&self) -> Option<AttributeValue<'_>>;

    fn tangents(&self) -> Option<AttributeValue<'_>>;

    fn bitangents(&self) -> Option<AttributeValue<'_>>;

    fn texture_coordinates(&self) -> Option<AttributeValue<'_>>;

    fn attribute_value(&self, name: &str) -> Option<AttributeValue<'_>>;

    fn uniform_value(&self, name: &str) -> Option<UniformValue<'_>>;

    fn uniform_block_value(&self, name: &str) -> Option<UniformBlockValue<'_>>;

    fn tick(&mut self, tick: &Tick);

    fn changed(&self) -> Receiver<GeometryMessage>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryMessage {
    Changed,
    BoundingVolumeChanged,
    PositionsChanged,
    TextureCoordinatesChanged,
    NormalsChanged,
    TangentsChanged,
    BitangentsChanged,
    VertexArrayObjectChanged,
}
