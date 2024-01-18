use gl_matrix4rust::{vec3::Vec3, GLF32Borrowed};

use crate::render::webgl::pipeline::UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH;

/// Maximum directional lights.
pub const MAX_DIRECTIONAL_LIGHTS: usize = 12;

const UBO_LIGHTS_DIRECTIONAL_LIGHT_F32_LENGTH: usize =
    UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH as usize / 4;

/// Directional light.
/// Direction of a directional light should points from light to outside
/// and should be normalized.
pub struct DirectionalLight {
    enabled: bool,
    direction: Vec3<f32>,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
    specular_shininess: f32,

    ubo: [f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_F32_LENGTH],
    dirty: bool,
}
impl DirectionalLight {
    /// Constructs a new directional light.
    /// Position and direction of a directional light should be in world space.
    pub fn new(
        direction: Vec3<f32>,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
        specular_shininess: f32,
    ) -> Self {
        Self {
            enabled: true,
            direction: direction.normalize(),
            ambient,
            diffuse,
            specular,
            specular_shininess,

            ubo: [0.0; UBO_LIGHTS_DIRECTIONAL_LIGHT_F32_LENGTH],
            dirty: true,
        }
    }

    /// Returns `true` if this directional light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns directional light direction.
    pub fn direction(&self) -> Vec3<f32> {
        self.direction
    }

    /// Returns directional light ambient color.
    pub fn ambient(&self) -> Vec3<f32> {
        self.ambient
    }

    /// Returns directional light diffuse color.
    pub fn diffuse(&self) -> Vec3<f32> {
        self.diffuse
    }

    /// Returns directional light specular color.
    pub fn specular(&self) -> Vec3<f32> {
        self.specular
    }

    /// Returns directional light specular shininess.
    pub fn specular_shininess(&self) -> f32 {
        self.specular_shininess
    }

    /// Enables directional light.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.dirty = true;
    }

    /// Disables directional light.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.dirty = true;
    }

    /// Sets directional light direction.
    pub fn set_direction(&mut self, direction: Vec3<f32>) {
        self.direction = direction.normalize();
        self.dirty = true;
    }

    /// Sets directional light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
        self.dirty = true;
    }

    /// Sets directional light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
        self.dirty = true;
    }

    /// Sets directional light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
        self.dirty = true;
    }

    /// Sets directional light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f32) {
        self.specular_shininess = specular_shininess;
        self.dirty = true;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn ubo(&self) -> &[u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH as usize] {
        unsafe {
            std::mem::transmute::<
                &[f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_F32_LENGTH],
                &[u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH as usize],
            >(&self.ubo)
        }
    }

    /// Returns `true` if ubo of this directional light is dirty.
    pub fn ubo_dirty(&self) -> bool {
        self.dirty
    }

    /// Updates ubo data if this directional light is dirty.
    pub fn update_ubo(&mut self) {
        if !self.dirty {
            return;
        }

        self.ubo[0..3].copy_from_slice(self.direction.gl_f32_borrowed());
        self.ubo[3] = if self.enabled { 1.0 } else { 0.0 };
        self.ubo[4..7].copy_from_slice(self.ambient.gl_f32_borrowed());
        self.ubo[8..11].copy_from_slice(self.diffuse.gl_f32_borrowed());
        self.ubo[12..15].copy_from_slice(self.specular.gl_f32_borrowed());
        self.ubo[15] = self.specular_shininess;

        self.dirty = false;
    }
}
