use std::any::Any;

use crate::{
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::ShaderSource,
            uniform::{UniformBinding, UniformValue, UniformBlockBinding, UniformBlockValue},
        },
    },
};

pub mod environment_mapping;
pub mod solid_color;
pub mod solid_color_instanced;
pub mod texture_mapping;
pub mod texture_mapping_instanced;
pub mod icon;
pub mod loader;

pub trait Material {
    fn name(&self) -> &'static str;

    /// Transparency of this material.
    /// 
    /// Unexpected render result may happens if you assign
    /// [`Transparency::Transparent`] or [`Transparency::Translucent`] to an opaque material.
    fn transparency(&self) -> Transparency;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>];

    fn attribute_value(&self, name: &str, entity: &BorrowedMut) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &BorrowedMut) -> Option<UniformValue>;

    #[allow(unused_variables)]
    fn uniform_block_value(&self, name: &str, entity: &BorrowedMut) -> Option<UniformBlockValue> {
        None
    }

    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_bounding_volume(&self) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn set_update_bounding_volume(&self, v: bool) {}

    fn update_matrices(&self) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn set_update_matrices(&self, v: bool) {}

    /// Preparation before entering drawing stage.
    ///
    /// Depending on [`MaterialPolicy`](crate::render::webgl::pipeline::policy::MaterialPolicy),
    /// `self` is not always extracted from entity. Thus, if you are not sure where the `self` from,
    /// do not borrow material from entity.
    #[allow(unused_variables)]
    fn prepare(&mut self, state: &State, entity: &BorrowedMut) {}

    #[allow(unused_variables)]
    fn before_draw(&mut self, state: &State, entity: &BorrowedMut) {}

    #[allow(unused_variables)]
    fn after_draw(&mut self, state: &State, entity: &BorrowedMut) {}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Transparency {
    Opaque,
    Transparent,
    Translucent(f32)
}

// pub struct MaterialRenderEntity<'a> {
//     entity: Strong,
//     geometry: *mut dyn Geometry,
//     collected: &'a [Strong],
//     drawings: &'a [Strong],
//     drawing_index: usize,
// }

// impl<'a> MaterialRenderEntity<'a> {
//     pub(crate) fn new(
//         entity: Strong,
//         geometry: *mut dyn Geometry,
//         collected: &'a [Strong],
//         drawings: &'a [Strong],
//         drawing_index: usize,
//     ) -> Self {
//         Self {
//             entity,
//             geometry,
//             collected,
//             drawings,
//             drawing_index,
//         }
//     }

//     #[inline]
//     pub fn entity(&self) -> &Strong {
//         &self.entity
//     }

//     #[inline]
//     pub fn geometry(&self) -> &dyn Geometry {
//         unsafe { &*self.geometry }
//     }

//     #[inline]
//     pub fn geometry_raw(&self) -> *mut dyn Geometry {
//         self.geometry
//     }

//     #[inline]
//     pub fn geometry_mut(&mut self) -> &mut dyn Geometry {
//         unsafe { &mut *self.geometry }
//     }

//     #[inline]
//     pub fn collected_entities(&self) -> &[Strong] {
//         self.collected
//     }

//     #[inline]
//     pub fn drawing_entities(&self) -> &[Strong] {
//         self.drawings
//     }

//     #[inline]
//     pub fn drawing_index(&self) -> usize {
//         self.drawing_index
//     }
// }
