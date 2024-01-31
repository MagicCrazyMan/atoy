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
    ubo_dirty: bool,
}

impl AmbientLight {
    /// Constructs a new ambient light.
    pub fn new(color: Vec3<f32>) -> Self {
        Self {
            enabled: true,
            color,

            ubo: [0.0; UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH],
            ubo_dirty: true,
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
        self.ubo_dirty = true;
    }

    /// Disables ambient light.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.ubo_dirty = true;
    }

    /// Sets ambient light color.
    pub fn set_color(&mut self, color: Vec3<f32>) {
        self.color = color;
        self.ubo_dirty = true;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn ubo(&self) -> &[u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH as usize] {
        unsafe {
            std::mem::transmute::<
                &[f32; UBO_LIGHTS_AMBIENT_LIGHT_F32_LENGTH],
                &[u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH as usize],
            >(&self.ubo)
        }
    }

    /// Sets ubo of this ambient light to dirty.
    pub fn set_ubo_dirty(&mut self) {
        self.ubo_dirty = true;
    }

    /// Returns `true` if ubo of this ambient light is dirty.
    pub fn ubo_dirty(&self) -> bool {
        self.ubo_dirty
    }

    /// Updates ubo data if this ambient light is dirty.
    pub fn update_ubo(&mut self) {
        if !self.ubo_dirty {
            return;
        }

        self.ubo[0..3].copy_from_slice(self.color.gl_f32_borrowed());
        self.ubo[3] = if self.enabled { 1.0 } else { 0.0 };

        self.ubo_dirty = false;
    }
}
