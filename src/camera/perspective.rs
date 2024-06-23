use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::{frustum::ViewFrustum, plane::Plane};

use super::Camera;

pub struct PerspectiveCamera {
    position: Vec3<f64>,
    center: Vec3<f64>,
    up: Vec3<f64>,
    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,
    view: Mat4<f64>,
    proj: Mat4<f64>,
    view_proj: Mat4<f64>,
    frustum: ViewFrustum,
}

impl PerspectiveCamera {
    pub fn new(
        position: Vec3<f64>,
        center: Vec3<f64>,
        up: Vec3<f64>,
        fovy: f64,
        aspect: f64,
        near: f64,
        far: Option<f64>,
    ) -> Self {
        let view = Mat4::<f64>::from_look_at(&position, &center, &up);
        let proj = Mat4::<f64>::from_perspective(fovy, aspect, near, far);
        let up = up.normalize();
        let frustum = frustum(position, center, up, fovy, aspect, near, far);
        Self {
            position,
            center,
            up,
            fovy,
            aspect,
            near,
            far,
            view,
            proj,
            view_proj: proj * view,
            frustum,
        }
    }

    fn update_view(&mut self) {
        self.view = Mat4::<f64>::from_look_at(&self.position, &self.center, &self.up);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::<f64>::from_perspective(self.fovy, self.aspect, self.near, self.far);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_frustum(&mut self) {
        self.frustum = frustum(
            self.position,
            self.center,
            self.up,
            self.fovy,
            self.aspect,
            self.near,
            self.far,
        );
    }

    pub fn center(&self) -> Vec3<f64> {
        self.center
    }

    pub fn up(&self) -> Vec3<f64> {
        self.up
    }

    pub fn aspect(&self) -> f64 {
        self.aspect
    }

    pub fn near(&self) -> f64 {
        self.near
    }

    pub fn far(&self) -> Option<f64> {
        self.far
    }

    pub fn fovy(&self) -> f64 {
        self.fovy
    }

    pub fn set_fovy(&mut self, fovy: f64) {
        self.fovy = fovy;
        self.update_proj();
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.aspect = aspect;
        self.update_proj();
    }

    pub fn set_near(&mut self, near: f64) {
        self.near = near;
        self.update_proj();
    }

    pub fn set_far(&mut self, far: Option<f64>) {
        self.far = far;
        self.update_proj();
    }

    pub fn set_position(&mut self, position: Vec3<f64>) {
        self.position = position;
        self.update_view();
    }

    pub fn set_center(&mut self, center: Vec3<f64>) {
        self.center = center;
        self.update_view();
    }

    pub fn set_up(&mut self, up: Vec3<f64>) {
        self.up = up.normalize();
        self.update_view();
    }
}

impl Camera for PerspectiveCamera {
    fn position(&self) -> Vec3<f64> {
        self.position
    }

    fn view_matrix(&self) -> Mat4<f64> {
        self.view
    }

    fn proj_matrix(&self) -> Mat4<f64> {
        self.proj
    }

    fn view_proj_matrix(&self) -> Mat4<f64> {
        self.view_proj
    }

    fn view_frustum(&self) -> ViewFrustum {
        self.frustum
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for PerspectiveCamera {
    fn default() -> Self {
        Self::new(
            Vec3::<f64>::new(0.0, 0.0, 2.0),
            Vec3::<f64>::new_zero(),
            Vec3::<f64>::new(0.0, 1.0, 0.0),
            60.0f64.to_radians(),
            1.0,
            0.5,
            None,
        )
    }
}

pub(super) fn frustum(
    position: Vec3<f64>,
    center: Vec3<f64>,
    up: Vec3<f64>,
    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,
) -> ViewFrustum {
    let nz = (position - center).normalize();
    let x = up.cross(&nz).normalize();
    let y = nz.cross(&x).normalize();
    let z = -nz;

    let p = position + z * near;
    let hh = (fovy / 2.0).tan() * near;
    let hw = aspect * hh;

    let top = {
        let pop = p + y * hh;
        let d = (pop - position).normalize();
        Plane::new(pop, x.cross(&d).normalize())
    };
    let bottom = {
        let pop = p + y * -hh;
        let d = (pop - position).normalize();
        Plane::new(pop, d.cross(&x).normalize())
    };
    let left = {
        let pop = p + x * -hw;
        let d = (pop - position).normalize();
        Plane::new(pop, y.cross(&d).normalize())
    };
    let right = {
        let pop = p + x * hw;
        let d = (pop - position).normalize();
        Plane::new(pop, d.cross(&y).normalize())
    };
    let near = { Plane::new(p, nz) };
    let far = match far {
        Some(far) => Some(Plane::new(position + z * far, z)),
        None => None,
    };

    ViewFrustum::new(left, right, top, bottom, near, far)
}
