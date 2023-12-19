use std::any::Any;

use gl_matrix4rust::vec3::Vec3;

/// A trait for defining an ambient light.
pub trait Ambient {
    /// Returns ambient light color.
    fn color(&self) -> Vec3;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Global ambient light.
#[derive(Clone, Copy)]
pub struct SimpleAmbientLight(Vec3);

impl SimpleAmbientLight {
    /// Constructs a new ambient light.
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Vec3::from_values(r, g, b))
    }
}

impl Ambient for SimpleAmbientLight {
    fn color(&self) -> Vec3 {
        self.0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
