pub mod cube;
pub mod indexed_cube;
pub mod raw;
pub mod sphere;
pub mod multicube;

use std::any::Any;

use crate::{
    bounding::BoundingVolumeNative,
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{attribute::AttributeValue, draw::Draw, uniform::UniformValue},
    },
};

pub trait Geometry {
    fn draw(&self) -> Draw;

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative>;

    fn vertices(&self) -> Option<AttributeValue>;

    fn normals(&self) -> Option<AttributeValue>;

    fn texture_coordinates(&self) -> Option<AttributeValue>;

    fn attribute_value(&self, name: &str, entity: &BorrowedMut) -> Option<AttributeValue>;

    fn uniform_value(&self, name: &str, entity: &BorrowedMut) -> Option<UniformValue>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_bounding_volume(&self) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn set_update_bounding_volume(&mut self, v: bool) {}

    fn update_matrices(&self) -> bool {
        false
    }

    #[allow(unused_variables)]
    fn set_update_matrices(&mut self, v: bool) {}

    #[allow(unused_variables)]
    fn prepare(&mut self, state: &State, entity: &BorrowedMut) {}

    #[allow(unused_variables)]
    fn before_draw(&mut self, state: &State, entity: &BorrowedMut) {}

    #[allow(unused_variables)]
    fn after_draw(&mut self, state: &State, entity: &BorrowedMut) {}
}

// pub struct GeometryRenderEntity<'a> {
//     entity: Strong,
//     material: *mut dyn Material,
//     collected: &'a [Strong],
//     drawings: &'a [Strong],
//     drawing_index: usize,
// }

// impl<'a> GeometryRenderEntity<'a> {
//     pub(crate) fn new(
//         entity: Strong,
//         material: *mut dyn Material,
//         collected: &'a [Strong],
//         drawings: &'a [Strong],
//         drawing_index: usize,
//     ) -> Self {
//         Self {
//             entity,
//             material,
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
//     pub fn material(&self) -> &dyn Material {
//         unsafe { &*self.material }
//     }

//     #[inline]
//     pub fn material_raw(&self) -> *mut dyn Material {
//         self.material
//     }

//     #[inline]
//     pub fn material_mut(&mut self) -> &mut dyn Material {
//         unsafe { &mut *self.material }
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
