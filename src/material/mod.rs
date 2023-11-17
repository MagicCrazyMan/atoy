use crate::{
    entity::Entity,
    geometry::Geometry,
    ncor::Ncor,
    render::webgl::program::{
        AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
    },
    scene::Scene,
};

pub mod solid_color;
pub mod solid_color_instanced;
pub mod texture_mapping;
pub mod texture_mapping_instanced;
pub mod environment_mapping;

pub trait WebGLMaterial {
    fn name(&self) -> &str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn sources(&self) -> &[ShaderSource];

    fn attribute_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, AttributeValue>>;

    fn uniform_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, UniformValue>>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    #[allow(unused_variables)]
    fn prepare(&mut self, scene: &Scene, entity: &Entity, geometry: &dyn Geometry) {}

    #[allow(unused_variables)]
    fn pre_render(&mut self, scene: &Scene, entity: &Entity, geometry: &dyn Geometry) {}

    #[allow(unused_variables)]
    fn post_render(&mut self, scene: &Scene, entity: &Entity, geometry: &dyn Geometry) {}
}
