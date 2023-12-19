use std::any::Any;

use gl_matrix4rust::vec3::Vec3;

pub trait Diffuse {
    /// Returns diffuse light color.
    fn color(&self) -> Vec3;

    /// Returns diffuse light position.
    fn position(&self) -> Vec3;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
