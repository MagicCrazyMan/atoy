pub mod orthogonal;
pub mod perspective;
pub mod universal;

use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::{frustum::ViewFrustum, readonly::Readonly};

pub trait Camera {
    fn position(&self) -> Readonly<'_, Vec3>;

    fn view_matrix(&self) -> Readonly<'_, Mat4>;

    fn proj_matrix(&self) -> Readonly<'_, Mat4>;

    fn view_proj_matrix(&self) -> Readonly<'_, Mat4>;

    fn view_frustum(&self) -> ViewFrustum;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
