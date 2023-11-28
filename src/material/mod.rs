use std::{cell::RefCell, rc::Rc, any::Any};

use crate::{
    entity::Entity,
    geometry::Geometry,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        pipeline::RenderState,
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
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

    fn attribute_value(&self, name: &str, entity: &MaterialRenderEntity) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &MaterialRenderEntity) -> Option<UniformValue>;

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Preparation before entering drawing stage.
    ///
    /// Depending on [`MaterialPolicy`](crate::render::webgl::pipeline::policy::MaterialPolicy),
    /// `self` is not always extracted from entity. Thus, if you are not sure where the `self` from,
    /// do not borrow material from entity.
    #[allow(unused_variables)]
    fn prepare(&mut self, state: &RenderState, entity: &Rc<RefCell<Entity>>) {}

    #[allow(unused_variables)]
    fn before_draw(&mut self, state: &RenderState, entity: &MaterialRenderEntity) {}

    #[allow(unused_variables)]
    fn after_draw(&mut self, state: &RenderState, entity: &MaterialRenderEntity) {}
}

pub struct MaterialRenderEntity {
    entity: Rc<RefCell<Entity>>,
    geometry: Rc<RefCell<dyn Geometry>>,
}

impl MaterialRenderEntity {
    pub(crate) fn new(entity: Rc<RefCell<Entity>>, geometry: Rc<RefCell<dyn Geometry>>) -> Self {
        Self { entity, geometry }
    }

    #[inline]
    pub fn entity(&self) -> &Rc<RefCell<Entity>> {
        &self.entity
    }

    #[inline]
    pub fn geometry(&self) -> &Rc<RefCell<dyn Geometry>> {
        &self.geometry
    }
}
