use gl_matrix4rust::{vec3::Vec3, GLF32Borrowed, GLF32};

use crate::render::webgl::pipeline::UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH;

/// Maximum spot lights.
pub const MAX_SPOT_LIGHTS: usize = 12;

const UBO_LIGHTS_SPOT_LIGHTS_F32_LENGTH: usize = UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH as usize / 4;

/// Spot light. Position and direction of a spot light should be in world space.
pub struct SpotLight {
    enabled: bool,
    position: Vec3,
    direction: Vec3<f32>,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
    specular_shininess: f32,
    inner_cutoff: f32,
    outer_cutoff: f32,

    ubo: [f32; UBO_LIGHTS_SPOT_LIGHTS_F32_LENGTH],
    dirty: bool,
}

impl SpotLight {
    /// Constructs a new spot light.
    /// Position and direction of a spot light should be in world space.
    /// `inner_cutoff` and `outer_cutoff` are in radians,
    /// and `outer_cutoff` should be larger than `inner_cutoff`.
    pub fn new(
        position: Vec3,
        direction: Vec3<f32>,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
        specular_shininess: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
    ) -> Self {
        Self {
            enabled: true,
            position,
            direction: direction.normalize(),
            ambient,
            diffuse,
            specular,
            specular_shininess,
            inner_cutoff: inner_cutoff,
            outer_cutoff: inner_cutoff.max(outer_cutoff),

            ubo: [0.0; UBO_LIGHTS_SPOT_LIGHTS_F32_LENGTH],
            dirty: true,
        }
    }

    /// Returns `true` if this spot light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns spot light position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Returns spot light direction.
    pub fn direction(&self) -> Vec3<f32> {
        self.direction
    }

    /// Returns spot light ambient color.
    pub fn ambient(&self) -> Vec3<f32> {
        self.ambient
    }

    /// Returns spot light diffuse color.
    pub fn diffuse(&self) -> Vec3<f32> {
        self.diffuse
    }

    /// Returns spot light specular color.
    pub fn specular(&self) -> Vec3<f32> {
        self.specular
    }

    /// Returns spot light specular shininess.
    pub fn specular_shininess(&self) -> f32 {
        self.specular_shininess
    }

    /// Enables spot light.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.dirty = true;
    }

    /// Disables spot light.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.dirty = true;
    }

    /// Sets spot light position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    /// Sets spot light direction.
    pub fn set_direction(&mut self, direction: Vec3<f32>) {
        self.direction = direction.normalize();
        self.dirty = true;
    }

    /// Sets spot light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
        self.dirty = true;
    }

    /// Sets spot light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
        self.dirty = true;
    }

    /// Sets spot light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
        self.dirty = true;
    }

    /// Sets spot light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f32) {
        self.specular_shininess = specular_shininess;
        self.dirty = true;
    }

    /// Returns inner cutoff for smooth lighting, in radians.
    pub fn inner_cutoff(&self) -> f32 {
        self.inner_cutoff
    }

    /// Returns outer cutoff for smooth lighting, in radians.
    pub fn outer_cutoff(&self) -> f32 {
        self.outer_cutoff
    }

    /// Sets inner cutoff for smooth lighting, in radians.
    pub fn set_inner_cutoff(&mut self, inner_cutoff: f32) {
        self.inner_cutoff = inner_cutoff;
        self.dirty = true;
    }

    /// Sets outer cutoff for smooth lighting, in radians.
    pub fn set_outer_cutoff(&mut self, outer_cutoff: f32) {
        self.outer_cutoff = outer_cutoff.max(self.inner_cutoff);
        self.dirty = true;
    }

    /// Returns data in uniform buffer object alignment.
    ///
    /// `inner_cutoff` and `outer_cutoff` are transformed from radians to cosine values.
    pub fn gl_ubo(&self) -> &[u8; UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH as usize] {
        unsafe {
            std::mem::transmute::<
                &[f32; UBO_LIGHTS_SPOT_LIGHTS_F32_LENGTH],
                &[u8; UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH as usize],
            >(&self.ubo)
        }
    }

    /// Returns `true` if this spot light is dirty.
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Updates ubo data if this spot light is dirty.
    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        self.ubo[0..3].copy_from_slice(self.direction.gl_f32_borrowed());
        self.ubo[3] = if self.enabled { 1.0 } else { 0.0 };
        self.ubo[4..7].copy_from_slice(&self.position.gl_f32());
        self.ubo[7] = 0.0;
        self.ubo[8..11].copy_from_slice(self.ambient.gl_f32_borrowed());
        self.ubo[11] = self.inner_cutoff.cos();
        self.ubo[12..15].copy_from_slice(self.diffuse.gl_f32_borrowed());
        self.ubo[15] = self.outer_cutoff.cos();
        self.ubo[16..19].copy_from_slice(self.specular.gl_f32_borrowed());
        self.ubo[19] = self.specular_shininess;

        self.dirty = false;
    }
}
