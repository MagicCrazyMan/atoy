use crate::{render::webgl::{
    attribute::{AttributeBinding, AttributeValue},
    pipeline::RenderState,
    program::ShaderSource,
    uniform::{UniformBinding, UniformValue},
    RenderingEntityState,
}, entity::Entity};

pub mod environment_mapping;
pub mod solid_color;
pub mod solid_color_instanced;
pub mod texture_mapping;
pub mod texture_mapping_instanced;

pub trait Material {
    fn name(&self) -> &'static str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>];

    fn attribute_value(&self, name: &str, state: &RenderingEntityState) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, state: &RenderingEntityState) -> Option<UniformValue>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    #[allow(unused_variables)]
    fn prepare(&mut self, state: &RenderState, entity: &Entity) {}
}
