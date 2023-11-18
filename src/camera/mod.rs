pub mod perspective;

use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

pub trait Camera {
    fn direction(&self) -> Vec3;

    fn position(&self) -> Vec3;

    fn view_matrix(&self) -> Mat4;

    fn proj_matrix(&self) -> Mat4;

    fn view_proj_matrix(&self) -> Mat4;

    fn set_position(&mut self, position: Vec3);

    fn set_center(&mut self, center: Vec3);

    fn set_up(&mut self, up: Vec3);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
