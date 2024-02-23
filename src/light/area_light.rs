use gl_matrix4rust::vec3::Vec3;

/// Area light. Position and direction of a area light should be in world space.
#[derive(Clone, Copy, PartialEq)]
pub struct AreaLight {
    enabled: bool,
    position: Vec3,
    direction: Vec3<f32>,
    up: Vec3<f32>,
    right: Vec3<f32>,
    offset: f32,
    inner_width: f32,
    inner_height: f32,
    outer_width: f32,
    outer_height: f32,
    ambient: Vec3<f32>,
    diffuse: Vec3<f32>,
    specular: Vec3<f32>,
}

impl AreaLight {
    /// Constructs a new area light.
    /// Area light. Position and direction of a area light should be in world space.
    pub fn new(
        position: Vec3,
        direction: Vec3<f32>,
        up: Vec3<f32>,
        offset: f32,
        inner_width: f32,
        inner_height: f32,
        outer_width: f32,
        outer_height: f32,
        ambient: Vec3<f32>,
        diffuse: Vec3<f32>,
        specular: Vec3<f32>,
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
    pub fn direction(&self) -> Vec3<f32> {
        self.direction
    }

    /// Returns area light upward.
    pub fn up(&self) -> Vec3<f32> {
        self.up
    }

    /// Returns area light rightward.
    pub fn right(&self) -> Vec3<f32> {
        self.right
    }

    /// Returns area light offset.
    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// Returns area light inner width for smooth lighting.
    pub fn inner_width(&self) -> f32 {
        self.inner_width
    }

    /// Returns area light inner height for smooth lighting.
    pub fn inner_height(&self) -> f32 {
        self.inner_height
    }

    /// Returns area light outer width for smooth lighting.
    pub fn outer_width(&self) -> f32 {
        self.outer_width
    }

    /// Returns area light outer height for smooth lighting
    pub fn outer_height(&self) -> f32 {
        self.outer_height
    }

    /// Returns area light ambient color.
    pub fn ambient(&self) -> Vec3<f32> {
        self.ambient
    }

    /// Returns area light diffuse color.
    pub fn diffuse(&self) -> Vec3<f32> {
        self.diffuse
    }

    /// Returns area light specular color.
    pub fn specular(&self) -> Vec3<f32> {
        self.specular
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
    pub fn set_direction(&mut self, direction: Vec3<f32>) {
        let direction = direction.normalize();
        let right = direction.cross(&self.up).normalize();
        let up = right.cross(&direction).normalize();

        self.direction = direction;
        self.right = right;
        self.up = up;
    }

    /// Sets area light upward.
    pub fn set_up(&mut self, up: Vec3<f32>) {
        let up = up.normalize();
        let right = self.direction.cross(&up).normalize();
        let up = right.cross(&self.direction).normalize();

        self.right = right;
        self.up = up;
    }

    /// Sets area light offset.
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    /// Sets area light inner width.
    pub fn set_inner_width(&mut self, inner_width: f32) {
        self.inner_width = inner_width;
    }

    /// Sets area light inner height.
    pub fn set_inner_height(&mut self, inner_height: f32) {
        self.inner_height = inner_height;
    }

    /// Sets area light outer width.
    pub fn set_outer_width(&mut self, outer_width: f32) {
        self.outer_width = outer_width.max(self.inner_width);
    }

    /// Sets area light outer height.
    pub fn set_outer_height(&mut self, outer_height: f32) {
        self.outer_height = outer_height.max(self.inner_height);
    }

    /// Sets area light ambient color.
    pub fn set_ambient(&mut self, ambient: Vec3<f32>) {
        self.ambient = ambient;
    }

    /// Sets area light diffuse color.
    pub fn set_diffuse(&mut self, diffuse: Vec3<f32>) {
        self.diffuse = diffuse;
    }

    /// Sets area light specular color.
    pub fn set_specular(&mut self, specular: Vec3<f32>) {
        self.specular = specular;
    }
}
