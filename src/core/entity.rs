use std::{any::Any, ops::Range};

use super::webgl::{
    attribute::{AttributeValue, IndicesDataType},
    buffer::BufferData,
    draw::DrawMode,
    uniform::{UniformBlockValue, UniformValue},
};

pub trait EntityComponent {
    fn attribute(&self, name: &str) -> Option<AttributeValue>;

    fn uniform(&self, name: &str) -> Option<UniformValue>;

    fn uniform_block(&self, name: &str) -> Option<UniformBlockValue>;

    fn property(&self, name: &str) -> Option<&dyn Any>;
}

pub trait Entity {
    type Component: ?Sized;

    fn draw_mode(&self) -> DrawMode;

    fn draw_range(&self) -> Range<usize>;

    fn components(&self) -> &[&Self::Component];
}

pub trait IndexedEntity: Entity {
    fn indices(&self) -> BufferData;

    fn indices_data_type(&self) -> IndicesDataType;

    fn indices_range(&self) -> Option<Range<usize>>;
}

pub trait InstancedEntity: Entity {
    fn instance_count(&self) -> usize;
}

pub struct Property {}
