pub mod perspective;

use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::render::webgl::pipeline::RenderState;

pub trait Camera {
    fn position(&self) -> Vec3;

    fn direction(&self) -> Vec3;

    fn view_matrix(&self) -> Mat4;

    fn proj_matrix(&self) -> Mat4;

    fn view_proj_matrix(&self) -> Mat4;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn update_frame(&mut self, state: &RenderState);
}
