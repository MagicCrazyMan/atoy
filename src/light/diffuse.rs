use std::any::Any;

use gl_matrix4rust::vec3::Vec3;

pub const MAX_DIFFUSE_LIGHTS: usize = 12;
pub const DIFFUSE_LIGHTS_UNIFORM_BLOCK_STRUCT_BYTES_SIZE_PER_LIGHT: usize = 16 + 16 + 16 + 16;
pub const DIFFUSE_LIGHTS_UNIFORM_BLOCK_BYTES_SIZE: usize =
    16 + DIFFUSE_LIGHTS_UNIFORM_BLOCK_STRUCT_BYTES_SIZE_PER_LIGHT * MAX_DIFFUSE_LIGHTS;

pub trait Diffuse {
    /// Returns diffuse light color.
    fn color(&self) -> Vec3;

    /// Returns diffuse light position.
    fn position(&self) -> Vec3;

    /// Returns diffuse light position.
    fn attenuations(&self) -> Vec3;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SimpleDiffuseLight {
    position: Vec3,
    color: Vec3,
    attenuations: Vec3,
}

impl SimpleDiffuseLight {
    pub fn new(position: Vec3, color: Vec3, attenuations: Vec3) -> Self {
        Self {
            position,
            color,
            attenuations,
        }
    }
}

impl Diffuse for SimpleDiffuseLight {
    fn color(&self) -> Vec3 {
        self.color
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn attenuations(&self) -> Vec3 {
        self.attenuations
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
