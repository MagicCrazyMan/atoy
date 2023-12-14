use std::any::Any;

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};
use log::info;

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
        self.update_frustum();
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
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
        info!("left   {:?}", self.frustum.left());
        info!("right  {:?}", self.frustum.right());
        info!("top    {:?}", self.frustum.top());
        info!("bottom {:?}", self.frustum.bottom());
        info!("near   {:?}", self.frustum.near());
        info!("far    {:?}", self.frustum.far());

        let a = self.a();
        info!("left   {:?}", a.left());
        info!("right  {:?}", a.right());
        info!("top    {:?}", a.top());
        info!("bottom {:?}", a.bottom());
        info!("near   {:?}", a.near());
        info!("far    {:?}", a.far());
    }

    fn a(&self) -> ViewFrustum {
        let x = Vec3::from_values(self.view.m00(), self.view.m10(), self.view.m20());
        let y = Vec3::from_values(self.view.m01(), self.view.m11(), self.view.m21());
        let z = Vec3::from_values(self.view.m02(), self.view.m12(), self.view.m22());
        let nz = z.negate();

        let p = self.position + nz * self.near;
        let hh = (self.fovy / 2.0).tan() * self.near;
        let hw = self.aspect * hh;

        let top = {
            let pop = p + y * hh;
            let d = (pop - self.position).normalize();
            Plane::new(pop, x.cross(&d).normalize())
        };
        let bottom = {
            let pop = p + y * -hh;
            let d = (pop - self.position).normalize();
            Plane::new(pop, d.cross(&x).normalize())
        };
        let left = {
            let pop = p + x * -hw;
            let d = (pop - self.position).normalize();
            Plane::new(pop, y.cross(&d).normalize())
        };
        let right = {
            let pop = p + x * hw;
            let d = (pop - self.position).normalize();
            Plane::new(pop, d.cross(&y).normalize())
        };
        let near = { Plane::new(p, z) };
        let far = match self.far {
            Some(far) => Some(Plane::new(self.position + nz * far, nz)),
            None => None,
        };

        ViewFrustum::new(left, right, top, bottom, near, far)
    }

    pub fn center(&self) -> Vec3 {
        self.center
    }

    pub fn up(&self) -> Vec3 {
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

pub(super) fn frustum(
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
