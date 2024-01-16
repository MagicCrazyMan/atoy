use gl_matrix4rust::vec3::Vec3;

/// Ambient light.
#[derive(Clone, Copy)]
pub struct AmbientLight {
    enabled: bool,
    color: Vec3,
}

impl AmbientLight {
    /// Constructs a new ambient light.
    pub fn new(color: Vec3) -> Self {
        Self {
            enabled: true,
            color,
        }
    }

    /// Returns ambient light color.
    pub fn color(&self) -> Vec3 {
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
    pub fn set_color(&mut self, color: Vec3) {
        self.color = color;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn gl_ubo(&self) -> [f32; 4] {
        [
            *self.color.x() as f32,
            *self.color.y() as f32,
            *self.color.z() as f32,
            if self.enabled { 1.0 } else { 0.0 },
        ]
    }
}
