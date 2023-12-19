pub mod ambient;
pub mod diffuse;
pub mod pointlight;

use gl_matrix4rust::vec3::Vec3;

pub trait Light {
    /// Light position.
    fn position(&self) -> Vec3;

    /// Ambient light color.
    fn ambient_color(&self) -> Vec3;

    /// Diffuse light color.
    fn diffuse_color(&self) -> Vec3;

    /// Specular light color
    fn specular_color(&self) -> Vec3;

    /// Specular light exponent.
    fn specular_exponent(&self) -> f64;

    /// Attenuation.
    fn attenuation(&self) -> Option<Attenuation>;
}

pub enum Attenuation {}
