use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity,
    geometry::Geometry,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
    scene::Scene,
};

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

    fn attribute_value(&self, name: &str, entity: &Entity) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &Entity) -> Option<UniformValue>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    #[allow(unused_variables)]
    fn prepare(
        &mut self,
        gl: &WebGl2RenderingContext,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
    ) {
    }

    #[allow(unused_variables)]
    fn pre_render(
        &mut self,
        gl: &WebGl2RenderingContext,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
    ) {
    }

    #[allow(unused_variables)]
    fn post_render(
        &mut self,
        gl: &WebGl2RenderingContext,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
    ) {
    }
}
