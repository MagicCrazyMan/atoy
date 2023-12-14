use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{frustum::ViewFrustum, render::pp::State};
use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};

use super::{perspective::PerspectiveCamera, Camera};

struct Shareable {
    rightward: Vec3,
    toward: Vec3,
    upward: Vec3,
    left_movement: f64,
    right_movement: f64,
    up_movement: f64,
    down_movement: f64,
    toward_movement: f64,
    backward_movement: f64,
}

impl Shareable {
    fn new(camera: &PerspectiveCamera) -> Self {
        let mut me = Self {
            rightward: Vec3::from_values(0.0, 0.0, 0.0),
            toward: Vec3::from_values(0.0, 0.0, 0.0),
            upward: Vec3::from_values(0.0, 0.0, 0.0),
            left_movement: 1.0,
            right_movement: 1.0,
            up_movement: 1.0,
            down_movement: 1.0,
            toward_movement: 1.0,
            backward_movement: 1.0,
        };
        me.update(camera);
        me
    }

    fn update(&mut self, camera: &PerspectiveCamera) {
        let toward = (camera.center() - camera.position()).normalize();
        let rightward = toward.cross(&camera.up()).normalize();
        let upward = rightward.cross(&toward).normalize();

        self.toward = toward;
        self.rightward = rightward;
        self.upward = upward;
    }
}

/// An controllable perspective camera with mouse, keyboard or screen touching.
///
/// UniversalCamera is shareable by cloning, making it convenient to control outside [`Scene`].
#[derive(Clone)]
pub struct UniversalCamera {
    camera: Rc<RefCell<PerspectiveCamera>>,
    inner: Rc<RefCell<Shareable>>,
}

impl UniversalCamera {
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
        let camera = PerspectiveCamera::new(position, center, up, fovy, aspect, near, far);
        let inner = Shareable::new(&camera);
        Self {
            camera: Rc::new(RefCell::new(camera)),
            inner: Rc::new(RefCell::new(inner)),
        }
    }

    pub fn go_left(&mut self) {}

    pub fn go_right(&mut self) {
        let mut camera = self.camera.borrow_mut();
        let mut inner = self.inner.borrow_mut();
        let m = inner.rightward * inner.right_movement;
        let np = camera.position() + m;
    }

    pub fn go_up(&mut self) {}

    pub fn go_down(&mut self) {}

    pub fn go_toward(&mut self) {}

    pub fn go_backward(&mut self) {}
}

impl UniversalCamera {
    pub fn center(&self) -> Vec3 {
        self.camera.borrow().center()
    }

    pub fn up(&self) -> Vec3 {
        self.camera.borrow().up()
    }

    pub fn fovy(&self) -> f64 {
        self.camera.borrow().fovy()
    }

    pub fn aspect(&self) -> f64 {
        self.camera.borrow().aspect()
    }

    pub fn set_fovy(&mut self, fovy: f64) {
        self.camera.borrow_mut().set_fovy(fovy);
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.camera.borrow_mut().set_aspect(aspect);
    }

    pub fn set_near(&mut self, near: f64) {
        self.camera.borrow_mut().set_near(near);
    }

    pub fn set_position<V>(&mut self, position: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        let mut camera = self.camera.borrow_mut();
        camera.set_position(position);
        self.inner.borrow_mut().update(&camera);
    }

    pub fn set_center<V>(&mut self, center: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        let mut camera = self.camera.borrow_mut();
        camera.set_center(center);
        self.inner.borrow_mut().update(&camera);
    }

    pub fn set_up<V>(&mut self, up: &V)
    where
        V: AsVec3<f64> + ?Sized,
    {
        let mut camera = self.camera.borrow_mut();
        camera.set_up(up);
        self.inner.borrow_mut().update(&camera);
    }
}

impl Camera for UniversalCamera {
    fn position(&self) -> Vec3 {
        self.camera.borrow().position()
    }

    fn center(&self) -> Vec3 {
        self.camera.borrow().center()
    }

    fn up(&self) -> Vec3 {
        self.camera.borrow().up()
    }

    fn aspect(&self) -> f64 {
        self.camera.borrow().aspect()
    }

    fn near(&self) -> f64 {
        self.camera.borrow().near()
    }

    fn far(&self) -> Option<f64> {
        self.camera.borrow().far()
    }

    fn view_matrix(&self) -> Mat4 {
        self.camera.borrow().view_matrix()
    }

    fn proj_matrix(&self) -> Mat4 {
        self.camera.borrow().proj_matrix()
    }

    fn view_proj_matrix(&self) -> Mat4 {
        self.camera.borrow().view_proj_matrix()
    }

    fn view_frustum(&self) -> ViewFrustum {
        self.camera.borrow().view_frustum()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_frame(&mut self, state: &State) {
        self.camera.borrow_mut().update_frame(state);
    }
}
