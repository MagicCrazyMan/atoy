use gl_matrix4rust::vec3::Vec3;

/// Maximum point lights.
pub const MAX_POINT_LIGHTS: usize = 12;

/// Point light. Position of a point light should be in world space.
pub struct PointLight {
    enabled: bool,
    position: Vec3,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    specular_shininess: f64,
}

impl PointLight {
    /// Constructs a new point light.
    pub fn new(position: Vec3, ambient: Vec3, diffuse: Vec3, specular: Vec3, specular_shininess: f64) -> Self {
        Self {
            enabled: true,
            position,
            ambient,
            diffuse,
            specular,
            specular_shininess,
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
    pub fn ambient(&self) -> Vec3 {
        self.ambient
    }

    /// Returns point light diffuse color.
    pub fn diffuse(&self) -> Vec3 {
        self.diffuse
    }

    /// Returns point light specular color.
    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    /// Returns point light specular shininess.
    pub fn specular_shininess(&self) -> f64 {
        self.specular_shininess
    }

    /// Enables point light.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables point light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets point light position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Sets point light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3) {
        self.ambient = ambient;
    }

    /// Sets point light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3) {
        self.diffuse = diffuse;
    }

    /// Sets point light specular color.
    pub fn set_specular(&mut self, specular: Vec3) {
        self.specular = specular;
    }

    /// Sets point light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f64) {
        self.specular_shininess = specular_shininess;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn gl_ubo(&self) -> [f32; 16] {
        [
            self.position.0[0] as f32,
            self.position.0[1] as f32,
            self.position.0[2] as f32,
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
