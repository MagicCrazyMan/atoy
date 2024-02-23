use gl_matrix4rust::vec3::Vec3;

/// Directional light.
/// Direction of a directional light should points from light to outside
/// and should be normalized.
#[derive(Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    enabled: bool,
    direction: Vec3<f32>,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
}
impl DirectionalLight {
    /// Constructs a new directional light.
    /// Position and direction of a directional light should be in world space.
    pub fn new(
        direction: Vec3<f32>,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
    ) -> Self {
        Self {
            enabled: true,
            direction: direction.normalize(),
            ambient,
            diffuse,
            specular,
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

    /// Enables directional light.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables directional light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets directional light direction.
    pub fn set_direction(&mut self, direction: Vec3<f32>) {
        self.direction = direction.normalize();
    }

    /// Sets directional light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
    }

    /// Sets directional light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
    }

    /// Sets directional light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
    }
}
