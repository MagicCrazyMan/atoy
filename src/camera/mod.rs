pub mod orthogonal;
pub mod perspective;
pub mod universal;

use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::frustum::ViewFrustum;

pub trait Camera {
    fn position(&self) -> Vec3<f64>;

    fn view_matrix(&self) -> Mat4<f64>;

    fn proj_matrix(&self) -> Mat4<f64>;

    fn view_proj_matrix(&self) -> Mat4<f64>;

    fn view_frustum(&self) -> ViewFrustum;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
