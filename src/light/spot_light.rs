use gl_matrix4rust::vec3::Vec3;

/// Maximum spot lights.
pub const MAX_SPOT_LIGHTS: usize = 12;

/// Spot light. Position and direction of a spot light should be in world space.
pub struct SpotLight {
    enabled: bool,
    position: Vec3,
    direction: Vec3,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    specular_shininess: f64,
    inner_cutoff: f64,
    outer_cutoff: f64,
}

impl SpotLight {
    /// Constructs a new spot light.
    /// Position and direction of a spot light should be in world space.
    /// `inner_cutoff` and `outer_cutoff` are in radians,
    /// and `outer_cutoff` should be larger than `inner_cutoff`.
    pub fn new(
        position: Vec3,
        direction: Vec3,
        ambient: Vec3,
        diffuse: Vec3,
        specular: Vec3,
        specular_shininess: f64,
        inner_cutoff: f64,
        outer_cutoff: f64,
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
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Returns spot light ambient color.
    pub fn ambient(&self) -> Vec3 {
        self.ambient
    }

    /// Returns spot light diffuse color.
    pub fn diffuse(&self) -> Vec3 {
        self.diffuse
    }

    /// Returns spot light specular color.
    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    /// Returns spot light specular shininess.
    pub fn specular_shininess(&self) -> f64 {
        self.specular_shininess
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
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Sets spot light direction.
    pub fn set_direction(&mut self, direction: Vec3) {
        self.direction = direction.normalize();
    }

    /// Sets spot light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3) {
        self.ambient = ambient;
    }

    /// Sets spot light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3) {
        self.diffuse = diffuse;
    }

    /// Sets spot light specular color.
    pub fn set_specular(&mut self, specular: Vec3) {
        self.specular = specular;
    }

    /// Sets spot light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f64) {
        self.specular_shininess = specular_shininess;
    }

    /// Returns inner cutoff for smooth lighting, in radians.
    pub fn inner_cutoff(&self) -> f64 {
        self.inner_cutoff
    }

    /// Returns outer cutoff for smooth lighting, in radians.
    pub fn outer_cutoff(&self) -> f64 {
        self.outer_cutoff
    }

    /// Sets inner cutoff for smooth lighting, in radians.
    pub fn set_inner_cutoff(&mut self, inner_cutoff: f64) {
        self.inner_cutoff = inner_cutoff;
    }

    /// Sets outer cutoff for smooth lighting, in radians.
    pub fn set_outer_cutoff(&mut self, outer_cutoff: f64) {
        self.outer_cutoff = outer_cutoff.max(self.inner_cutoff);
    }

    /// Returns data in uniform buffer object alignment.
    ///
    /// `inner_cutoff` and `outer_cutoff` are transformed from radians to cosine values.
    pub fn gl_ubo(&self) -> [f32; 20] {
        [
            *self.direction.x() as f32,
            *self.direction.y() as f32,
            *self.direction.z() as f32,
            if self.enabled { 1.0 } else { 0.0 },
            *self.position.x() as f32,
            *self.position.y() as f32,
            *self.position.z() as f32,
            0.0,
            *self.ambient.x() as f32,
            *self.ambient.y() as f32,
            *self.ambient.z() as f32,
            self.inner_cutoff.cos() as f32,
            *self.diffuse.x() as f32,
            *self.diffuse.y() as f32,
            *self.diffuse.z() as f32,
            self.outer_cutoff.cos() as f32,
            *self.specular.x() as f32,
            *self.specular.y() as f32,
            *self.specular.z() as f32,
            self.specular_shininess as f32,
        ]
    }
}
