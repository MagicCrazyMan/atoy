use std::any::Any;

use gl_matrix4rust::vec3::Vec3;

pub const MAX_DIFFUSE_LIGHTS: usize = 12;

pub trait Diffuse {
    /// Returns diffuse light color.
    fn color(&self) -> Vec3;

    /// Returns diffuse light position.
    fn position(&self) -> Vec3;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SimpleDiffuseLight {
    position: Vec3,
    color: Vec3,
}

impl SimpleDiffuseLight {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Self { position, color }
    }
}

impl Diffuse for SimpleDiffuseLight {
    fn color(&self) -> Vec3 {
        self.color
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
