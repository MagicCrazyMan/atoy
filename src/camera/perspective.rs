use std::any::Any;

use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::render::webgl::pipeline::RenderState;

use super::Camera;

#[wasm_bindgen]
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
        Self {
            position: Vec3::from_as_vec3(position),
            center: Vec3::from_as_vec3(center),
            up: Vec3::from_as_vec3(up),
            fovy,
            aspect,
            near,
            far,
            view,
            proj,
            view_proj: proj * view,
        }
    }

    fn update_view(&mut self) {
        self.view = Mat4::from_look_at(&self.position, &self.center, &self.up);
        self.view_proj = self.proj * self.view
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
        self.view_proj = self.proj * self.view
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

    pub fn near(&self) -> f64 {
        self.near
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
        self.up.copy(up);
        self.update_view();
    }
}

impl Camera for PerspectiveCamera {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn direction(&self) -> Vec3 {
        self.center - self.position
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_frame(&mut self, state: &RenderState) {
        self.set_aspect(state.canvas().width() as f64 / state.canvas().height() as f64)
    }
}
