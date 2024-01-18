use gl_matrix4rust::{vec3::Vec3, GLF32Borrowed};

use crate::render::webgl::pipeline::UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH;

const UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH: usize =
    UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH as usize / 4;

/// Ambient light.
#[derive(Clone, Copy)]
pub struct AmbientLight {
    enabled: bool,
    color: Vec3<f32>,

    ubo: [f32; UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH],
    dirty: bool,
}

impl AmbientLight {
    /// Constructs a new ambient light.
    pub fn new(color: Vec3<f32>) -> Self {
        Self {
            enabled: true,
            color,

            ubo: [0.0; UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH],
            dirty: true,
        }
    }

    /// Returns ambient light color.
    pub fn color(&self) -> Vec3<f32> {
        self.color
    }

    /// Returns `true` if this ambient light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Enables ambient light.
    pub fn enable(&mut self) {
        self.enabled = true;
        self.dirty = true;
    }

    /// Disables ambient light.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.dirty = true;
    }

    /// Sets ambient light color.
    pub fn set_color(&mut self, color: Vec3<f32>) {
        self.color = color;
        self.dirty = true;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn gl_ubo(&self) -> &[u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH as usize] {
        unsafe {
             std::mem::transmute::<&[f32; UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH], &[u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH as usize]>(&self.ubo)
        }
    }

    /// Returns `true` if this ambient light is dirty.
    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Updates ubo data if this ambient light is dirty.
    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        self.ubo[0..3].copy_from_slice(self.color.gl_f32_borrowed());
        self.ubo[3] = if self.enabled { 1.0 } else { 0.0 };

        self.dirty = false;
    }
}
