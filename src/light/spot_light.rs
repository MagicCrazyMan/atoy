use gl_matrix4rust::vec3::Vec3;

/// Spot light. Position and direction of a spot light should be in world space.
#[derive(Clone, Copy, PartialEq)]
pub struct SpotLight {
    enabled: bool,
    position: Vec3<f64>,
    direction: Vec3<f32>,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
    inner_cutoff: f32,
    outer_cutoff: f32,
}

impl SpotLight {
    /// Constructs a new spot light.
    /// Position and direction of a spot light should be in world space.
    /// `inner_cutoff` and `outer_cutoff` are in radians,
    /// and `outer_cutoff` should be larger than `inner_cutoff`.
    pub fn new(
        position: Vec3<f64>,
        direction: Vec3<f32>,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
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
            inner_cutoff: inner_cutoff,
            outer_cutoff: inner_cutoff.max(outer_cutoff),
        }
    }

    /// Returns `true` if this spot light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns spot light position.
    pub fn position(&self) -> Vec3<f64> {
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

    /// Enables spot light.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables spot light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets spot light position.
    pub fn set_position(&mut self, position: Vec3<f64>) {
        self.position = position;
    }

    /// Sets spot light direction.
    pub fn set_direction(&mut self, direction: Vec3<f32>) {
        self.direction = direction.normalize();
    }

    /// Sets spot light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
    }

    /// Sets spot light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
    }

    /// Sets spot light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
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
    }

    /// Sets outer cutoff for smooth lighting, in radians.
    pub fn set_outer_cutoff(&mut self, outer_cutoff: f32) {
        self.outer_cutoff = outer_cutoff.max(self.inner_cutoff);
    }
}
