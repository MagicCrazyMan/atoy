pub mod perspective;

use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::{frustum::ViewingFrustum, render::pp::State};

pub trait Camera {
    fn position(&self) -> Vec3;

    fn center(&self) -> Vec3;

    fn up(&self) -> Vec3;

    fn aspect(&self) -> f64;

    fn near(&self) -> f64;

    fn far(&self) -> Option<f64>;

    fn view_matrix(&self) -> Mat4;

    fn proj_matrix(&self) -> Mat4;

    fn view_proj_matrix(&self) -> Mat4;

    fn viewing_frustum(&self) -> ViewingFrustum;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_frame(&mut self, state: &State);
}
