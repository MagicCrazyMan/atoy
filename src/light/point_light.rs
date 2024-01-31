use gl_matrix4rust::{vec3::Vec3, GLF32Borrowed, GLF32};

use crate::render::webgl::pipeline::UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH;

/// Maximum point lights.
pub const MAX_POINT_LIGHTS: usize = 40;
pub const POINT_LIGHTS_COUNT_DEFINE: &'static str = "POINT_LIGHTS_COUNT";

const UBO_LIGHTS_POINT_LIGHTS_F32_LENGTH: usize = UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH as usize / 4;

/// Point light. Position of a point light should be in world space.
pub struct PointLight {
    enabled: bool,
    position: Vec3,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,

    ubo: [f32; UBO_LIGHTS_POINT_LIGHTS_F32_LENGTH],
    ubo_dirty: bool,
}

impl PointLight {
    /// Constructs a new point light.
    pub fn new(
        position: Vec3,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
    ) -> Self {
        Self {
            enabled: true,
            position,
            ambient,
            diffuse,
            specular,

            ubo: [0.0; UBO_LIGHTS_POINT_LIGHTS_F32_LENGTH],
            ubo_dirty: true,
        }
    }

    /// Returns `true` if this point light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns point light position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Returns point light ambient color.
    pub fn ambient(&self) -> Vec3<f32> {
        self.ambient
    }

    /// Returns point light diffuse color.
    pub fn diffuse(&self) -> Vec3<f32> {
        self.diffuse
    }

    /// Returns point light specular color.
    pub fn specular(&self) -> Vec3<f32> {
        self.specular
    }

    /// Enables point light.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.ubo_dirty = true;
    }

    /// Disables point light.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.ubo_dirty = true;
    }

    /// Sets point light position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.ubo_dirty = true;
    }

    /// Sets point light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
        self.ubo_dirty = true;
    }

    /// Sets point light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
        self.ubo_dirty = true;
    }

    /// Sets point light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
        self.ubo_dirty = true;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn ubo(&self) -> &[u8; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH as usize] {
        unsafe {
            std::mem::transmute::<
                &[f32; UBO_LIGHTS_POINT_LIGHTS_F32_LENGTH],
                &[u8; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH as usize],
            >(&self.ubo)
        }
    }

    /// Sets ubo of this point light to dirty.
    pub fn set_ubo_dirty(&mut self) {
        self.ubo_dirty = true;
    }

    /// Returns `true` if ubo of this point light is dirty.
    pub fn ubo_dirty(&self) -> bool {
        self.ubo_dirty
    }

    /// Updates ubo data if this point light is dirty.
    pub fn update_ubo(&mut self) {
        if !self.ubo_dirty {
            return;
        }

        self.ubo[0..3].copy_from_slice(&self.position.gl_f32());
        self.ubo[3] = if self.enabled { 1.0 } else { 0.0 };
        self.ubo[4..7].copy_from_slice(self.ambient.gl_f32_borrowed());
        self.ubo[8..11].copy_from_slice(self.diffuse.gl_f32_borrowed());
        self.ubo[12..15].copy_from_slice(self.specular.gl_f32_borrowed());

        self.ubo_dirty = false;
    }
}
