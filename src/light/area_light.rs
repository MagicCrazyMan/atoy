use gl_matrix4rust::vec3::{Vec3, AsVec3};

/// Maximum area lights.
pub const MAX_AREA_LIGHTS: usize = 12;

/// Area light. Position and direction of a area light should be in world space.
pub struct AreaLight {
    enabled: bool,
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
    offset: f64,
    inner_width: f64,
    inner_height: f64,
    outer_width: f64,
    outer_height: f64,
    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,
    specular_shininess: f64,
}

impl AreaLight {
    /// Constructs a new area light.
    /// Area light. Position and direction of a area light should be in world space.
    pub fn new(
        position: Vec3,
        direction: Vec3,
        up: Vec3,
        offset: f64,
        inner_width: f64,
        inner_height: f64,
        outer_width: f64,
        outer_height: f64,
        ambient: Vec3,
        diffuse: Vec3,
        specular: Vec3,
        specular_shininess: f64,
    ) -> Self {
        let direction = direction.normalize();
        let up = up.normalize();
        let right = direction.cross(&up).normalize();
        let up = right.cross(&direction).normalize();
        Self {
            enabled: true,
            position,
            direction,
            right,
            up,
            offset,
            inner_width,
            inner_height,
            outer_width: outer_width.max(inner_width),
            outer_height: outer_height.max(inner_height),
            ambient,
            diffuse,
            specular,
            specular_shininess,
        }
    }

    /// Returns `true` if this area light is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns area light position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Returns area light direction.
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Returns area light upward.
    pub fn up(&self) -> Vec3 {
        self.up
    }

    /// Returns area light rightward.
    pub fn right(&self) -> Vec3 {
        self.right
    }

    /// Returns area light offset.
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// Returns area light inner width for smooth lighting.
    pub fn inner_width(&self) -> f64 {
        self.inner_width
    }

    /// Returns area light inner height for smooth lighting.
    pub fn inner_height(&self) -> f64 {
        self.inner_height
    }

    /// Returns area light outer width for smooth lighting.
    pub fn outer_width(&self) -> f64 {
        self.outer_width
    }

    /// Returns area light outer height for smooth lighting
    pub fn outer_height(&self) -> f64 {
        self.outer_height
    }

    /// Returns area light ambient color.
    pub fn ambient(&self) -> Vec3 {
        self.ambient
    }

    /// Returns area light diffuse color.
    pub fn diffuse(&self) -> Vec3 {
        self.diffuse
    }

    /// Returns area light specular color.
    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    /// Returns area light specular shininess.
    pub fn specular_shininess(&self) -> f64 {
        self.specular_shininess
    }

    /// Enables area light.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables area light.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Sets area light position.
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// Sets area light direction.
    pub fn set_direction(&mut self, direction: Vec3) {
        let direction = direction.normalize();
        let right = direction.cross(&self.up).normalize();
        let up = right.cross(&direction).normalize();

        self.direction = direction;
        self.right = right;
        self.up = up;
    }

    /// Sets area light upward.
    pub fn set_up(&mut self, up: Vec3) {
        let up = up.normalize();
        let right = self.direction.cross(&up).normalize();
        let up = right.cross(&self.direction).normalize();

        self.right = right;
        self.up = up;
    }

    /// Sets area light offset.
    pub fn set_offset(&mut self, offset: f64) {
        self.offset = offset;
    }

    /// Sets area light inner width.
    pub fn set_inner_width(&mut self, inner_width: f64) {
        self.inner_width = inner_width;
    }

    /// Sets area light inner height.
    pub fn set_inner_height(&mut self, inner_height: f64) {
        self.inner_height = inner_height;
    }

    /// Sets area light outer width.
    pub fn set_outer_width(&mut self, outer_width: f64) {
        self.outer_width = outer_width.max(self.inner_width);
    }

    /// Sets area light outer height.
    pub fn set_outer_height(&mut self, outer_height: f64) {
        self.outer_height = outer_height.max(self.inner_height);
    }

    /// Sets area light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3) {
        self.ambient = ambient;
    }

    /// Sets area light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3) {
        self.diffuse = diffuse;
    }

    /// Sets area light specular color.
    pub fn set_specular(&mut self, specular: Vec3) {
        self.specular = specular;
    }

    /// Sets area light specular shininess.
    pub fn set_specular_shininess(&mut self, specular_shininess: f64) {
        self.specular_shininess = specular_shininess;
    }

    /// Returns data in uniform buffer object alignment.
    pub fn gl_ubo(&self) -> [f32; 28] {
        [
            self.direction.0[0] as f32,
            self.direction.0[1] as f32,
            self.direction.0[2] as f32,
            if self.enabled { 1.0 } else { 0.0 },
            self.up.0[0] as f32,
            self.up.0[1] as f32,
            self.up.0[2] as f32,
            self.inner_width as f32,
            self.right.0[0] as f32,
            self.right.0[1] as f32,
            self.right.0[2] as f32,
            self.inner_height as f32,
            self.position.0[0] as f32,
            self.position.0[1] as f32,
            self.position.0[2] as f32,
            self.offset as f32,
            self.ambient.0[0] as f32,
            self.ambient.0[1] as f32,
            self.ambient.0[2] as f32,
            self.outer_width as f32,
            self.diffuse.0[0] as f32,
            self.diffuse.0[1] as f32,
            self.diffuse.0[2] as f32,
            self.outer_height as f32,
            self.specular.0[0] as f32,
            self.specular.0[1] as f32,
            self.specular.0[2] as f32,
            self.specular_shininess as f32,
        ]
    }
}
