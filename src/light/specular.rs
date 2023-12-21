use std::any::Any;

use gl_matrix4rust::vec3::Vec3;


pub const MAX_SPECULAR_LIGHTS: usize = 12;


pub trait Specular {
    /// Returns specular light color.
    fn color(&self) -> Vec3;

    /// Returns specular light position.
    fn position(&self) -> Vec3;

    /// Returns specular light shininess.
    fn shininess(&self) -> f32;

    /// Returns specular light attenuations.
    fn attenuations(&self) -> Vec3;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SimpleSpecularLight {
    position: Vec3,
    color: Vec3,
    shininess: f32,
    attenuations: Vec3,
}

impl SimpleSpecularLight {
    pub fn new(position: Vec3, color: Vec3, shininess: f32, attenuations: Vec3) -> Self {
        Self {
            position,
            color,
            shininess,
            attenuations,
        }
    }
}

impl Specular for SimpleSpecularLight {
    fn color(&self) -> Vec3 {
        self.color
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn shininess(&self) -> f32 {
        self.shininess
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
