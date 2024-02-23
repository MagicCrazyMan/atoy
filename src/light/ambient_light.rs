use gl_matrix4rust::vec3::Vec3;

/// Ambient light.
#[derive(Clone, Copy, PartialEq)]
pub struct AmbientLight {
    enabled: bool,
    color: Vec3<f32>,
}

impl AmbientLight {
    /// Constructs a new ambient light.
    pub fn new(color: Vec3<f32>) -> Self {
        Self {
            enabled: true,
            color,
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
    }

    /// Disables ambient light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets ambient light color.
    pub fn set_color(&mut self, color: Vec3<f32>) {
        self.color = color;
    }
}
