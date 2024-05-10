use std::ops::Range;

use crate::core::{
    clock::Tick,
    entity::{Component, Entity},
};

use super::{
    attribute::{AttributeValue, IndicesDataType},
    buffer::BufferData,
    context::Context,
    draw::DrawMode,
    uniform::{UniformBlockValue, UniformValue},
    WebGl,
};

pub trait WebGlComponent: Component<WebGl> {
    fn attribute(&self, name: &str) -> Option<AttributeValue>;

    fn uniform(&self, name: &str) -> Option<UniformValue>;

    fn uniform_block(&self, name: &str) -> Option<UniformBlockValue>;

    fn tick(&mut self, ticking: &Tick);

    fn pre_render(&mut self, context: &Context);

    fn post_render(&mut self, context: &Context);
}

pub trait WebGlEntity: Entity<WebGl> {
    fn draw_mode(&self) -> DrawMode;

    fn draw_range(&self) -> Range<usize>;

    fn tick(&mut self, ticking: &Tick);

    fn pre_render(&mut self, context: &Context);

    fn post_render(&mut self, context: &Context);

    fn as_indexed_entity(&self) -> Option<&dyn WebGlIndexedEntity>;

    fn as_instanced_entity(&self) -> Option<&dyn WebGlInstancedEntity>;
}

pub trait WebGlIndexedEntity: WebGlEntity {
    fn indices(&self) -> BufferData;

    fn indices_data_type(&self) -> IndicesDataType;

    fn indices_range(&self) -> Option<Range<usize>>;
}

pub trait WebGlInstancedEntity: WebGlEntity {
    fn instance_count(&self) -> usize;
}
