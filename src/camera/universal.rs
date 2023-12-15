use std::{any::Any, borrow::Cow, cell::RefCell, f64::consts::PI, rc::Rc};

use crate::{frustum::ViewFrustum, plane::Plane, render::pp::State};
use gl_matrix4rust::{
    mat3::Mat3,
    mat4::{AsMat4, Mat4},
    quat::{AsQuat, Quat},
    quat2::Quat2,
    vec3::{AsVec3, Vec3},
};
use log::{error, info};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent};

use super::Camera;

const BASIC_RIGHTWARD: (f64, f64, f64) = (1.0, 0.0, 0.0);
const BASIC_UPWARD: (f64, f64, f64) = (0.0, 1.0, 0.0);
// camera coordinate system is a right hand side coordinate system
// flip z axis to convert it to left hand side
const BASIC_FORWARD: (f64, f64, f64) = (0.0, 0.0, -1.0);

struct Shareable {
    forward: Vec3,
    rightward: Vec3,
    upward: Vec3,

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
    forward_movement: f64,
    backward_movement: f64,
    y_rotation: f64,
    x_rotation: f64,

    binding_canvas: Option<HtmlCanvasElement>,
    keyboard_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    mousemove_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
}

impl Shareable {
    #[inline]
    fn move_right(&mut self) {
        self.move_directional(self.rightward, self.right_movement)
    }

    #[inline]
    fn move_left(&mut self) {
        self.move_directional(self.rightward, -self.left_movement)
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
    fn move_forward(&mut self) {
        self.move_directional(self.forward, self.forward_movement);
    }

    #[inline]
    fn move_backward(&mut self) {
        self.move_directional(self.forward, -self.backward_movement);
    }

    #[inline]
    fn move_directional(&mut self, direction: Vec3, movement: f64) {
        let offset = direction * -movement;
        self.view = self.view.translate(&offset);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    #[inline]
    fn rotate_y(&mut self) {
        self.rotate_rad(0.0, self.y_rotation)
    }

    #[inline]
    fn rotate_ny(&mut self) {
        self.rotate_rad(0.0, -self.y_rotation)
    }

    #[inline]
    fn rotate_x(&mut self) {
        self.rotate_rad(self.x_rotation, 0.0)
    }

    #[inline]
    fn rotate_nx(&mut self) {
        self.rotate_rad(-self.x_rotation, 0.0)
    }

    #[inline]
    fn rotate_rad(&mut self, rx: f64, ry: f64) {
        let p = self.view.translation();
        self.view = self.view.translate(&p.negate()).rotate_y(ry).translate(&p);
        self.view_proj = self.proj * self.view;

        self.forward = Vec3::from_as_vec3(BASIC_FORWARD)
            .transform_mat4(&self.view)
            .normalize();
        self.rightward = Vec3::from_as_vec3(BASIC_RIGHTWARD)
            .transform_mat4(&self.view)
            .normalize();
        self.upward = Vec3::from_as_vec3(BASIC_UPWARD)
            .transform_mat4(&self.view)
            .normalize();

        // ensuring perpendicular to each other
        self.rightward = self.forward.cross(&self.upward).normalize();
        self.upward = self.rightward.cross(&self.forward).normalize();

        info!("{:?}", self.forward);
        info!("{:?}", self.rightward);
        info!("{:?}", self.upward);

        self.update_frustum();
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
        let current = self.view.translation().negate();
        let offset = position - current;
        self.view = self.view.translate(&offset);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
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

    fn forward_movement(&self) -> f64 {
        self.forward_movement
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

    fn set_forward_movement(&mut self, forward_movement: f64) {
        self.forward_movement = forward_movement
    }

    fn set_backward_movement(&mut self, backward_movement: f64) {
        self.backward_movement = backward_movement
    }

    fn update_proj(&mut self) {
        self.proj = Mat4::from_perspective(self.fovy, self.aspect, self.near, self.far);
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_frustum(&mut self) {
        self.frustum = frustum(
            self.view.translation().negate(),
            self.forward,
            self.rightward,
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

        if let Some(callback) = self.mousemove_callback.take() {
            if let Err(err) = canvas
                .remove_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref())
            {
                error!(
                    target: "UniversalCamera",
                    "failed to unbind mousemove event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }
        }
    }
}

/// An first personal based, controllable perspective camera with mouse, keyboard or screen touching.
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

        let forward = (center - position).normalize();
        // let backward = forward.negate();
        // let rightward = forward.cross(&up).normalize();
        // let upward = rightward.cross(&forward).normalize();
        // info!("{:?}", forward);
        // info!("{:?}", rightward);
        // info!("{:?}", upward);

        let view = Mat4::from_look_at(&position, &center, &up);
        let forward = Vec3::from_as_vec3(BASIC_FORWARD)
            .transform_mat4(&view)
            .normalize();
        let rightward = Vec3::from_as_vec3(BASIC_RIGHTWARD)
            .transform_mat4(&view)
            .normalize();
        let upward = Vec3::from_as_vec3(BASIC_UPWARD)
            .transform_mat4(&view)
            .normalize();
        info!("{:?}", forward);
        info!("{:?}", rightward);
        info!("{:?}", upward);
        let rightward = forward.cross(&upward).normalize();
        let upward = rightward.cross(&forward).normalize();
        info!("{:?}", forward);
        info!("{:?}", rightward);
        info!("{:?}", upward);
        let proj = Mat4::from_perspective(fovy, aspect, near, far);
        let frustum = frustum(
            position, forward, rightward, upward, fovy, aspect, near, far,
        );

        let default_movement = 0.5;
        let default_rotation = 0.0005;

        let sharable = Shareable {
            rightward,
            forward,
            upward,

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
            forward_movement: default_movement,
            backward_movement: default_movement,
            y_rotation: default_rotation,
            x_rotation: default_rotation,

            binding_canvas: None,
            keyboard_callback: None,
            mousemove_callback: None,
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

    pub fn move_forward(&mut self) {
        self.sharable.borrow_mut().move_forward()
    }

    pub fn move_backward(&mut self) {
        self.sharable.borrow_mut().move_backward()
    }

    pub fn rotate(&mut self, horizontal_rad: f64, vertical_rad: f64) {
        self.sharable
            .borrow_mut()
            .rotate_rad(horizontal_rad, vertical_rad)
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

    pub fn forward_movement(&self) -> f64 {
        self.sharable.borrow().forward_movement()
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

    pub fn set_forward_movement(&mut self, forward_movement: f64) {
        self.sharable
            .borrow_mut()
            .set_forward_movement(forward_movement)
    }

    pub fn set_backward_movement(&mut self, backward_movement: f64) {
        self.sharable
            .borrow_mut()
            .set_backward_movement(backward_movement)
    }
}

impl Camera for UniversalCamera {
    fn position(&self) -> Vec3 {
        self.sharable.borrow().view.translation()
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
                info!("{}", key);
                match key.as_str() {
                    "w" => shareable.move_forward(),
                    "a" => shareable.move_left(),
                    "s" => shareable.move_backward(),
                    "d" => shareable.move_right(),
                    "ArrowUp" => shareable.move_up(),
                    "ArrowDown" => shareable.move_down(),
                    "ArrowLeft" => shareable.rotate_y(),
                    "ArrowRight" => shareable.rotate_ny(),
                    _ => return,
                }

                event.prevent_default();
                event.stop_propagation();
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

            let shareable_weak = Rc::downgrade(&self.sharable);
            let previous_mouse = Rc::new(RefCell::new(None));
            shareable.mousemove_callback = Some(Closure::new(move |event: MouseEvent| {
                let Some(shareable) = shareable_weak.upgrade() else {
                    return;
                };
                let mut shareable = shareable.borrow_mut();

                let mut previous_event = previous_mouse.borrow_mut();

                if event.buttons() == 4 {
                    let Some(p) = previous_event.take() else {
                        *previous_event = Some(event);
                        return;
                    };

                    let px = p.x();
                    let py = p.y();
                    let x = event.x();
                    let y = event.y();
                    let ox = x - px;
                    let oy = y - py;

                    let rx = oy as f64 * shareable.x_rotation;
                    let ry = ox as f64 * shareable.y_rotation;
                    shareable.rotate_rad(rx, ry);

                    event.prevent_default();
                    event.stop_propagation();

                    *previous_event = Some(event);
                } else {
                    *previous_event = None;
                }
            }));
            if let Err(err) = canvas.add_event_listener_with_callback(
                "mousemove",
                shareable
                    .mousemove_callback
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            ) {
                error!(
                    target: "UniversalCamera",
                    "failed to bind mousemove event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }

            shareable.binding_canvas = Some(canvas.clone());
        }
    }
}

pub(super) fn frustum(
    position: Vec3,
    forward: Vec3,
    rightward: Vec3,
    upward: Vec3,
    fovy: f64,
    aspect: f64,
    near: f64,
    far: Option<f64>,
) -> ViewFrustum {
    let x = rightward;
    let y = upward;
    let z = forward;
    let nz = forward.negate();

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
