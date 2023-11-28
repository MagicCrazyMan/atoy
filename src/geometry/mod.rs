pub mod cube;
pub mod indexed_cube;
pub mod plane;
pub mod raw;
pub mod sphere;
// pub mod plane;

use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{
    entity::Entity,
    material::Material,
    render::webgl::{
        attribute::AttributeValue, draw::Draw, pipeline::RenderState, uniform::UniformValue,
    },
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str, entity: &GeometryRenderEntity) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &GeometryRenderEntity) -> Option<UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    #[allow(unused_variables)]
    fn prepare(&mut self, state: &RenderState, entity: &Rc<RefCell<Entity>>) {}

    #[allow(unused_variables)]
    fn before_draw(&mut self, state: &RenderState, entity: &GeometryRenderEntity) {}

    #[allow(unused_variables)]
    fn after_draw(&mut self, state: &RenderState, entity: &GeometryRenderEntity) {}
}

pub struct GeometryRenderEntity {
    entity: Rc<RefCell<Entity>>,
    material: Rc<RefCell<dyn Material>>,
}

impl GeometryRenderEntity {
    pub(crate) fn new(entity: Rc<RefCell<Entity>>, material: Rc<RefCell<dyn Material>>) -> Self {
        Self { entity, material }
    }

    #[inline]
    pub fn entity(&self) -> &Rc<RefCell<Entity>> {
        &self.entity
    }

    #[inline]
    pub fn material(&self) -> &Rc<RefCell<dyn Material>> {
        &self.material
    }
}
