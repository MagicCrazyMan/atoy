use gl_matrix4rust::vec3::Vec3;

/// Point light. Position of a point light should be in world space.
#[derive(Clone, Copy, PartialEq)]
pub struct PointLight {
    enabled: bool,
    position: Vec3,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
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
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
    }

    /// Sets point light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
    }

    /// Sets point light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
    }
}
