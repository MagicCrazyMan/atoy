use std::{any::Any, borrow::Cow, cell::RefCell, rc::Rc};

use crate::{frustum::ViewFrustum, render::pp::State};
use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};
use log::error;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, KeyboardEvent};

use super::{perspective::frustum, Camera};

struct Shareable {
    position: Vec3,
    pseudo_center: Vec3,
    rightward: Vec3,
    toward: Vec3,
    upward: Vec3,
    distance: f64,

    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,

    view: Mat4,
    proj: Mat4,
    view_proj: Mat4,
    frustum: ViewFrustum,

    left_movement: f64,
    right_movement: f64,
    up_movement: f64,
    down_movement: f64,
    toward_movement: f64,
    backward_movement: f64,

    binding_canvas: Option<HtmlCanvasElement>,
    keyboard_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
}

impl Shareable {
    #[inline]
    fn move_right(&mut self) {
        self.move_directional(self.rightward, self.left_movement)
    }

    #[inline]
    fn move_left(&mut self) {
        self.move_directional(self.rightward, -self.right_movement)
    }

    #[inline]
    fn move_up(&mut self) {
        self.move_directional(self.upward, self.up_movement)
    }

    #[inline]
    fn move_down(&mut self) {
        self.move_directional(self.upward, -self.down_movement)
    }

    #[inline]
    fn move_toward(&mut self) {
        self.move_directional(self.toward, self.toward_movement);
    }

    #[inline]
    fn move_backward(&mut self) {
        self.move_directional(self.toward, -self.backward_movement);
    }

    #[inline]
    fn move_toward_locking(&mut self) {
        if self.distance == 0.0 {
            return;
        }

        let remain = self.distance - self.toward_movement;
        if remain < 0.0 {
            self.distance = 0.0;
            self.move_directional(self.toward, self.distance);
        } else {
            self.distance = remain;
            self.move_directional(self.toward, self.toward_movement);
        }
    }

    #[inline]
    fn move_backward_locking(&mut self) {
        self.distance += self.backward_movement;
        self.move_directional(self.toward, self.backward_movement);
    }

    #[inline]
    fn move_directional(&mut self, direction: Vec3, movement: f64) {
        let np = self.position + direction * movement;
        let nc = np + self.toward;
        self.position = np;
        self.pseudo_center = nc;

        self.update_view();
    }

    #[inline]
    fn rotate(&mut self) {
        
    }

    fn fovy(&self) -> f64 {
        self.fovy
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

    fn set_fovy(&mut self, fovy: f64) {
        self.fovy = fovy;
        self.update_proj();
    }

    fn set_aspect(&mut self, aspect: f64) {
        self.aspect = aspect;
        self.update_proj();
    }

    fn set_near(&mut self, near: f64) {
        self.near = near;
        self.update_proj();
    }

    fn set_far(&mut self, far: Option<f64>) {
        self.far = far;
        self.update_proj();
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.toward = (self.pseudo_center - self.position).normalize();
        self.rightward = self.toward.cross(&self.upward).normalize();
        self.upward = self.rightward.cross(&self.toward).normalize();
        self.pseudo_center = self.position + self.toward * self.distance;
        self.update_view();
    }

    fn set_center(&mut self, center: Vec3) {
        self.toward = (center - self.position).normalize();
        self.rightward = self.toward.cross(&self.upward).normalize();
        self.upward = self.rightward.cross(&self.toward).normalize();
        self.pseudo_center = self.position + self.toward * self.distance;
        self.update_view();
    }

    fn left_movement(&self) -> f64 {
        self.left_movement
    }

    fn right_movement(&self) -> f64 {
        self.right_movement
    }

    fn up_movement(&self) -> f64 {
        self.up_movement
    }

    fn down_movement(&self) -> f64 {
        self.down_movement
    }

    fn toward_movement(&self) -> f64 {
        self.toward_movement
    }

    fn backward_movement(&self) -> f64 {
        self.backward_movement
    }

    fn set_left_movement(&mut self, left_movement: f64) {
        self.left_movement = left_movement
    }

    fn set_right_movement(&mut self, right_movement: f64) {
        self.right_movement = right_movement
    }

    fn set_up_movement(&mut self, up_movement: f64) {
        self.up_movement = up_movement
    }

    fn set_down_movement(&mut self, down_movement: f64) {
        self.down_movement = down_movement
    }

    fn set_toward_movement(&mut self, toward_movement: f64) {
        self.toward_movement = toward_movement
    }

    fn set_backward_movement(&mut self, backward_movement: f64) {
        self.backward_movement = backward_movement
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_view(&mut self) {
        self.view = Mat4::from_look_at(&self.position, &self.pseudo_center, &self.upward);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_frustum(&mut self) {
        self.frustum = frustum(
            self.position,
            self.pseudo_center,
            self.upward,
            self.fovy,
            self.aspect,
            self.near,
            self.far,
        );
    }
}

impl Drop for Shareable {
    fn drop(&mut self) {
        let Some(canvas) = self.binding_canvas.take() else {
            return;
        };

        if let Some(callback) = self.keyboard_callback.take() {
            if let Err(err) = canvas
                .remove_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
            {
                error!(
                    target: "UniversalCamera",
                    "failed to unbind keyboard event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }
        }
    }
}

/// An controllable perspective camera with mouse, keyboard or screen touching.
///
/// UniversalCamera is shareable by cloning, making it convenient to control outside [`Scene`].
#[derive(Clone)]
pub struct UniversalCamera {
    sharable: Rc<RefCell<Shareable>>,
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
        let position = Vec3::from_as_vec3(position);
        let center = Vec3::from_as_vec3(center);
        let up = Vec3::from_as_vec3(up);

        let distance = 10.0;
        let toward = (center - position).normalize();
        let rightward = toward.cross(&up).normalize();
        let upward = rightward.cross(&toward).normalize();
        let pseudo_center = position + toward * distance;
        let frustum = frustum(position, center, up, fovy, aspect, near, far);

        let view = Mat4::from_look_at(&position, &pseudo_center, &up);
        let proj = Mat4::from_perspective(fovy, aspect, near, far);

        let default_movement = 0.5;

        let sharable = Shareable {
            position,
            pseudo_center,
            rightward,
            toward,
            upward,
            distance,

            fovy,
            aspect,
            near,
            far,
            view,
            proj,
            view_proj: proj * view,
            frustum,

            left_movement: default_movement,
            right_movement: default_movement,
            up_movement: default_movement,
            down_movement: default_movement,
            toward_movement: default_movement,
            backward_movement: default_movement,

            binding_canvas: None,
            keyboard_callback: None,
        };

        Self {
            sharable: Rc::new(RefCell::new(sharable)),
        }
    }
}

impl UniversalCamera {
    pub fn move_right(&mut self) {
        self.sharable.borrow_mut().move_right()
    }

    pub fn move_left(&mut self) {
        self.sharable.borrow_mut().move_left()
    }

    pub fn move_up(&mut self) {
        self.sharable.borrow_mut().move_up()
    }

    pub fn move_down(&mut self) {
        self.sharable.borrow_mut().move_down()
    }

    pub fn move_toward(&mut self) {
        self.sharable.borrow_mut().move_toward()
    }

    pub fn move_backward(&mut self) {
        self.sharable.borrow_mut().move_backward()
    }

    pub fn move_toward_locking(&mut self) {
        self.sharable.borrow_mut().move_toward_locking()
    }

    pub fn move_backward_locking(&mut self) {
        self.sharable.borrow_mut().move_backward_locking()
    }

    pub fn fovy(&self) -> f64 {
        self.sharable.borrow().fovy()
    }

    pub fn aspect(&self) -> f64 {
        self.sharable.borrow().aspect()
    }

    pub fn near(&self) -> f64 {
        self.sharable.borrow().near()
    }

    pub fn far(&self) -> Option<f64> {
        self.sharable.borrow().far()
    }

    pub fn set_fovy(&mut self, fovy: f64) {
        self.sharable.borrow_mut().set_fovy(fovy)
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.sharable.borrow_mut().set_aspect(aspect)
    }

    pub fn set_near(&mut self, near: f64) {
        self.sharable.borrow_mut().set_near(near)
    }

    pub fn set_far(&mut self, far: Option<f64>) {
        self.sharable.borrow_mut().set_far(far)
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.sharable.borrow_mut().set_position(position)
    }

    pub fn set_center(&mut self, center: Vec3) {
        self.sharable.borrow_mut().set_center(center)
    }

    pub fn left_movement(&self) -> f64 {
        self.sharable.borrow().left_movement()
    }

    pub fn right_movement(&self) -> f64 {
        self.sharable.borrow().right_movement()
    }

    pub fn up_movement(&self) -> f64 {
        self.sharable.borrow().up_movement()
    }

    pub fn down_movement(&self) -> f64 {
        self.sharable.borrow().down_movement()
    }

    pub fn toward_movement(&self) -> f64 {
        self.sharable.borrow().toward_movement()
    }

    pub fn backward_movement(&self) -> f64 {
        self.sharable.borrow().backward_movement()
    }

    pub fn set_left_movement(&mut self, left_movement: f64) {
        self.sharable.borrow_mut().set_left_movement(left_movement)
    }

    pub fn set_right_movement(&mut self, right_movement: f64) {
        self.sharable
            .borrow_mut()
            .set_right_movement(right_movement)
    }

    pub fn set_up_movement(&mut self, up_movement: f64) {
        self.sharable.borrow_mut().set_up_movement(up_movement)
    }

    pub fn set_down_movement(&mut self, down_movement: f64) {
        self.sharable.borrow_mut().set_down_movement(down_movement)
    }

    pub fn set_toward_movement(&mut self, toward_movement: f64) {
        self.sharable
            .borrow_mut()
            .set_toward_movement(toward_movement)
    }

    pub fn set_backward_movement(&mut self, backward_movement: f64) {
        self.sharable
            .borrow_mut()
            .set_backward_movement(backward_movement)
    }
}

impl Camera for UniversalCamera {
    fn position(&self) -> Vec3 {
        self.sharable.borrow().position
    }

    fn view_matrix(&self) -> Mat4 {
        self.sharable.borrow().view
    }

    fn proj_matrix(&self) -> Mat4 {
        self.sharable.borrow().proj
    }

    fn view_proj_matrix(&self) -> Mat4 {
        self.sharable.borrow().view_proj
    }

    fn view_frustum(&self) -> ViewFrustum {
        self.sharable.borrow().frustum
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_frame(&mut self, state: &State) {
        let mut shareable = self.sharable.borrow_mut();

        let aspect = state.canvas().width() as f64 / state.canvas().height() as f64;
        if aspect != shareable.aspect {
            shareable.set_aspect(aspect);
        }

        // binds to canvas
        if shareable
            .binding_canvas
            .as_ref()
            .map(|canvas| canvas != state.canvas())
            .unwrap_or(true)
        {
            let canvas = state.canvas();

            let shareable_weak = Rc::downgrade(&self.sharable);
            shareable.keyboard_callback = Some(Closure::new(move |event: KeyboardEvent| {
                let Some(shareable) = shareable_weak.upgrade() else {
                    return;
                };
                let mut shareable = shareable.borrow_mut();

                let key = event.key();
                match key.as_str() {
                    "w" => shareable.move_toward(),
                    "a" => shareable.move_left(),
                    "s" => shareable.move_backward(),
                    "d" => shareable.move_right(),
                    _ => {}
                }
            }));
            if let Err(err) = canvas.add_event_listener_with_callback(
                "keydown",
                shareable
                    .keyboard_callback
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            ) {
                error!(
                    target: "UniversalCamera",
                    "failed to bind keyboard event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }

            shareable.binding_canvas = Some(canvas.clone());
        }
    }
}
