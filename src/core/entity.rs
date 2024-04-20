use std::ops::Range;

use super::webgl::{
    attribute::{Attribute, IndicesDataType},
    buffer::BufferData,
    draw::DrawMode, uniform::Uniform,
};

pub trait EntityComponent {
    fn attributes(&self, name: &str) -> &Attribute;

    fn uniforms(&self, name: &str) -> &Uniform;

    fn properties(&self, name: &str) -> &Property;
}

pub trait Entity {
    fn draw_mode(&self) -> DrawMode;

    fn draw_range(&self) -> Range<usize>;

    fn components(&self) -> &[&dyn EntityComponent];
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
