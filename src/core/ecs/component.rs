use std::any::TypeId;

use gl_matrix4rust::{mat4::Mat4, quat::Quat, vec3::Vec3};
use proc::{AsAny, Component};

use crate::core::AsAny;

pub trait Component {}

// #[derive(AsAny, Component)]
// pub struct Transformation {
//     translation: Vec3<f64>,
//     rotation: Quat<f64>,
//     scale: Vec3<f64>,

//     model_matrix: Mat4<f64>,
// }

// impl Transformation {
//     pub fn new() -> Self {
//         Self {
//             translation: Vec3::<f64>::new_zero(),
//             rotation: Quat::<f64>::new_identity(),
//             scale: Vec3::<f64>::new(1.0, 1.0, 1.0),

//             model_matrix: Mat4::<f64>::new_identity(),
//         }
//     }

//     pub fn with_translation_rotation_scale(
//         translation: Vec3<f64>,
//         rotation: Quat<f64>,
//         scale: Vec3<f64>,
//     ) -> Self {
//         Self {
//             model_matrix: Mat4::<f64>::from_rotation_translation_scale(
//                 &rotation,
//                 &translation,
//                 &scale,
//             ),

//             translation,
//             rotation,
//             scale,
//         }
//     }

//     pub fn translation(&self) -> &Vec3<f64> {
//         &self.translation
//     }

//     pub fn rotation(&self) -> &Quat<f64> {
//         &self.rotation
//     }

//     pub fn scale(&self) -> &Vec3<f64> {
//         &self.scale
//     }

//     pub fn set_translation(&mut self, translation: Vec3<f64>) {
//         self.translation = translation;
//         self.update_model_matrix();
//     }

//     pub fn set_rotation(&mut self, rotation: Quat<f64>) {
//         self.rotation = rotation;
//         self.update_model_matrix();
//     }

//     pub fn set_scale(&mut self, scale: Vec3<f64>) {
//         self.scale = scale;
//         self.update_model_matrix();
//     }

//     pub fn set_translation_rotation_scale(
//         &mut self,
//         translation: Vec3<f64>,
//         rotation: Quat<f64>,
//         scale: Vec3<f64>,
//     ) {
//         self.translation = translation;
//         self.rotation = rotation;
//         self.scale = scale;
//         self.update_model_matrix();
//     }

//     pub fn model_matrix(&self) -> &Mat4<f64> {
//         &self.model_matrix
//     }

//     fn update_model_matrix(&mut self) {
//         self.model_matrix = Mat4::<f64>::from_rotation_translation_scale(
//             &self.rotation,
//             &self.translation,
//             &self.scale,
//         );
//     }
// }
