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

pub struct GeometryRenderEntity<'a> {
    entity: Rc<RefCell<Entity>>,
    material: *mut dyn Material,
    collected: &'a [Rc<RefCell<Entity>>],
    drawings: &'a [Rc<RefCell<Entity>>],
    drawing_index: usize,
}

impl<'a> GeometryRenderEntity<'a> {
    pub(crate) fn new(
        entity: Rc<RefCell<Entity>>,
        material: *mut dyn Material,
        collected: &'a [Rc<RefCell<Entity>>],
        drawings: &'a [Rc<RefCell<Entity>>],
        drawing_index: usize,
    ) -> Self {
        Self {
            entity,
            material,
            collected,
            drawings,
            drawing_index,
        }
    }

    #[inline]
    pub fn entity(&self) -> &Rc<RefCell<Entity>> {
        &self.entity
    }

    #[inline]
    pub fn material(&self) -> &dyn Material {
        unsafe { &*self.material }
    }

    #[inline]
    pub fn material_raw(&self) -> *mut dyn Material {
        self.material
    }

    #[inline]
    pub fn material_mut(&mut self) -> &mut dyn Material {
        unsafe { &mut *self.material }
    }

    #[inline]
    pub fn collected_entities(&self) -> &[Rc<RefCell<Entity>>] {
        self.collected
    }

    #[inline]
    pub fn drawing_entities(&self) -> &[Rc<RefCell<Entity>>] {
        self.drawings
    }

    #[inline]
    pub fn drawing_index(&self) -> usize {
        self.drawing_index
    }
}
