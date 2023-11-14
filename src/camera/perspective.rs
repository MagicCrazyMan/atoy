use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};
use wasm_bindgen::prelude::wasm_bindgen;

use super::Camera;

#[wasm_bindgen]
pub struct PerspectiveCamera {
    position: Vec3,
    direction: Vec3,
    center: Vec3,
    up: Vec3,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: Option<f32>,
    view: Mat4,
    proj: Mat4,
}

impl PerspectiveCamera {
    pub fn new(
        position: Vec3,
        center: Vec3,
        up: Vec3,
        fovy: f32,
        aspect: f32,
        near: f32,
        far: Option<f32>,
    ) -> Self {
        let mut camera = Self {
            position,
            direction: center - position,
            center,
            up,
            fovy,
            aspect,
            near,
            far,
            view: Mat4::new_identity(),
            proj: Mat4::new_identity(),
        };
        camera.update_view();
        camera.update_proj();
        camera
    }

    fn update_view(&mut self) {
        self.view = Mat4::from_look_at(&self.position, &self.center, &self.up);
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
    }

    pub fn position(&self) -> &Vec3<f32> {
        &self.position
    }

    pub fn center(&self) -> &Vec3<f32> {
        &self.center
    }

    pub fn fovy(&self) -> f32 {
        self.fovy
    }

    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    pub fn near(&self) -> f32 {
        self.near
    }

    pub fn set_fovy(&mut self, fovy: f32) {
        self.fovy = fovy;
        self.update_proj();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update_proj();
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
        self.update_proj();
    }

    pub fn set_eye_silent(&mut self, eye: Vec3) {
        self.position = eye;
    }

    pub fn set_center_silent(&mut self, center: Vec3) {
        self.center = center;
    }

    pub fn set_up_silent(&mut self, up: Vec3) {
        self.up = up;
    }

    pub fn set_fovy_silent(&mut self, fovy: f32) {
        self.fovy = fovy;
    }

    pub fn set_aspect_silent(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn set_near_silent(&mut self, near: f32) {
        self.near = near;
    }
}

impl Camera for PerspectiveCamera {
    fn direction(&self) -> &Vec3 {
        &self.direction
    }

    fn position(&self) -> &Vec3 {
        &self.position
    }

    fn view(&self) -> &Mat4 {
        &self.view
    }

    fn proj(&self) -> &Mat4 {
        &self.proj
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.direction = self.center - position;
        self.update_view();
    }

    fn set_center(&mut self, center: Vec3) {
        self.center = center;
        self.direction = center - self.position;
        self.update_view();
    }

    fn set_up(&mut self, up: Vec3) {
        self.up = up;
        self.update_view();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
