use std::ops::Range;

use super::webgl::{
    attribute::{Attribute, IndicesDataType},
    buffer::BufferData,
    draw::DrawMode,
};

pub trait Entity {
    fn draw_mode(&self) -> DrawMode;

    fn draw_range(&self) -> Range<usize>;

    fn attributes(&self) -> &[Attribute];

    fn uniforms(&self) -> &[Uniform];

    fn properties(&self) -> &[Property];
}

pub trait IndexedEntity: Entity {
    fn indices(&self) -> BufferData;

    fn indices_data_type(&self) -> IndicesDataType;

    fn indices_range(&self) -> Option<Range<usize>>;
}

pub trait InstancedEntity: Entity {
    fn instance_count(&self) -> usize;
}

pub struct Uniform {}

pub struct Property {}
