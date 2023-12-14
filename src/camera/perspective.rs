use std::any::Any;

use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};

use crate::{frustum::ViewFrustum, plane::Plane, render::pp::State};

use super::Camera;

pub struct PerspectiveCamera {
    position: Vec3,
    center: Vec3,
    up: Vec3,
    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,
    view: Mat4,
    proj: Mat4,
    view_proj: Mat4,
    frustum: ViewFrustum,
}

impl PerspectiveCamera {
    pub fn new<V1, V2, V3>(
        position: V1,
        center: V2,
        up: V3,
        fovy: f64,
        aspect: f64,
        near: f64,
        far: Option<f64>,
    ) -> Self
    where
        V1: AsVec3<f64>,
        V2: AsVec3<f64>,
        V3: AsVec3<f64>,
    {
        let view = Mat4::from_look_at(&position, &center, &up);
        let proj = Mat4::from_perspective(fovy, aspect, near, far);
        let position = Vec3::from_as_vec3(position);
        let center = Vec3::from_as_vec3(center);
        let up = Vec3::from_as_vec3(up).normalize();
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
        self.view = Mat4::from_look_at(&self.position, &self.center, &self.up);
        self.view_proj = self.proj * self.view;
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

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
        self.view_proj = self.proj * self.view;
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

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn fovy(&self) -> f64 {
        self.fovy
    }

    pub fn aspect(&self) -> f64 {
        self.aspect
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

    pub fn set_position<V>(&mut self, position: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        self.position.copy(position);
        self.update_view();
    }

    pub fn set_center<V>(&mut self, center: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        self.center.copy(center);
        self.update_view();
    }

    pub fn set_up<V>(&mut self, up: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        self.up = self.up.copy(up).normalize();
        self.update_view();
    }
}

impl Camera for PerspectiveCamera {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn center(&self) -> Vec3 {
        self.center
    }

    fn up(&self) -> Vec3 {
        self.up
    }

    fn aspect(&self) -> f64 {
        self.aspect
    }

    fn near(&self) -> f64 {
        self.near
    }

    fn far(&self) -> Option<f64> {
        self.far
    }

    fn view_matrix(&self) -> Mat4 {
        self.view
    }

    fn proj_matrix(&self) -> Mat4 {
        self.proj
    }

    fn view_proj_matrix(&self) -> Mat4 {
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

    fn update_frame(&mut self, state: &State) {
        let aspect = state.canvas().width() as f64 / state.canvas().height() as f64;
        if aspect != self.aspect {
            self.set_aspect(aspect);
        }
    }
}

fn frustum(
    position: Vec3,
    center: Vec3,
    up: Vec3,
    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,
) -> ViewFrustum {
    let z = (position - center).normalize();
    let x = up.cross(&z).normalize();
    let y = z.cross(&x).normalize();
    let nz = z.negate();

    let p = position + nz * near;
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
    let near = { Plane::new(p, z) };
    let far = match far {
        Some(far) => Some(Plane::new(position + nz * far, nz)),
        None => None,
    };

    ViewFrustum::new(left, right, top, bottom, near, far)
}
