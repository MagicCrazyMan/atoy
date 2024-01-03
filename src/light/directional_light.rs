use gl_matrix4rust::vec3::{Vec3, AsVec3};

/// Maximum directional lights.
pub const MAX_DIRECTIONAL_LIGHTS: usize = 12;

/// Directional light.
/// Direction of a directional light should points from light to outside
/// and should be normalized.
pub struct DirectionalLight {
    enabled: bool,
    direction: Vec3,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    specular_shininess: f64,
}
impl DirectionalLight {
    /// Constructs a new directional light.
    /// Position and direction of a directional light should be in world space.
    pub fn new(direction: Vec3, ambient: Vec3, diffuse: Vec3, specular: Vec3, specular_shininess: f64) -> Self {
        Self {
            enabled: true,
            direction: direction.normalize(),
            ambient,
            diffuse,
            specular,
            specular_shininess,
        }
    }

    /// Returns `true` if this directional light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns directional light direction.
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Returns directional light ambient color.
    pub fn ambient(&self) -> Vec3 {
        self.ambient
    }

    /// Returns directional light diffuse color.
    pub fn diffuse(&self) -> Vec3 {
        self.diffuse
    }

    /// Returns directional light specular color.
    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    /// Returns directional light specular shininess.
    pub fn specular_shininess(&self) -> f64 {
        self.specular_shininess
    }

    /// Enables directional light.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables directional light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets directional light direction.
    pub fn set_direction(&mut self, direction: Vec3) {
        self.direction = direction.normalize();
    }

    /// Sets directional light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3) {
        self.ambient = ambient;
    }

    /// Sets directional light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3) {
        self.diffuse = diffuse;
    }

    /// Sets directional light specular color.
    pub fn set_specular(&mut self, specular: Vec3) {
        self.specular = specular;
    }

    /// Sets directional light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f64) {
        self.specular_shininess = specular_shininess;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn gl_ubo(&self) -> [f32; 16] {
        [
            self.direction.0[0] as f32,
            self.direction.0[1] as f32,
            self.direction.0[2] as f32,
            if self.enabled { 1.0 } else { 0.0 },
            self.ambient.0[0] as f32,
            self.ambient.0[1] as f32,
            self.ambient.0[2] as f32,
            0.0,
            self.diffuse.0[0] as f32,
            self.diffuse.0[1] as f32,
            self.diffuse.0[2] as f32,
            0.0,
            self.specular.0[0] as f32,
            self.specular.0[1] as f32,
            self.specular.0[2] as f32,
            self.specular_shininess as f32,
        ]
    }
}
