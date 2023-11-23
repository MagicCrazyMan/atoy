use crate::{
    entity::Entity,
    geometry::Geometry,
    render::webgl::program::{
        AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
    },
    scene::Scene,
};

pub mod solid_color;
pub mod solid_color_instanced;
// pub mod solid_color_instanced;
// pub mod texture_mapping;
// pub mod texture_mapping_instanced;
// pub mod environment_mapping;

pub trait Material {
    fn name(&self) -> &'static str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>];

    fn attribute_value(&self, name: &str) -> Option<AttributeValue>;

    fn uniform_value<'a>(&'a self, name: &str) -> Option<UniformValue<'a>>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    #[allow(unused_variables)]
    fn prepare(&mut self, scene: &mut Scene, entity: &mut Entity, geometry: &mut dyn Geometry) {}

    #[allow(unused_variables)]
    fn pre_render(&mut self, scene: &mut Scene, entity: &mut Entity, geometry: &mut dyn Geometry) {}

    #[allow(unused_variables)]
    fn post_render(&mut self, scene: &mut Scene, entity: &mut Entity, geometry: &mut dyn Geometry) {}
}
