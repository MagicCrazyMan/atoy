use std::{any::Any, borrow::Cow, cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{frustum::ViewFrustum, plane::Plane, render::pp::State};
use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};
use log::{error, warn};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, WheelEvent};

use super::Camera;

const BASE_RIGHTWARD: Vec3 = Vec3::from_values(1.0, 0.0, 0.0);
const BASE_UPWARD: Vec3 = Vec3::from_values(0.0, 1.0, 0.0);
// camera coordinate system is a right hand side coordinate system
// flip z axis to convert it to left hand side
const BASE_FORWARD: Vec3 = Vec3::from_values(0.0, 0.0, -1.0);

struct Shareable {
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
    z_rotation: f64,

    binding_canvas: Option<HtmlCanvasElement>,
    keys_pressed: HashSet<String>,
    previous_timestamp: Option<f64>,
    keydown_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    keyup_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    mousemove_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    wheel_callback: Option<Closure<dyn FnMut(WheelEvent)>>,
}

impl Shareable {
    #[inline]
    fn move_right(&mut self) {
        self.move_directional(BASE_RIGHTWARD, self.right_movement)
    }

    #[inline]
    fn move_left(&mut self) {
        self.move_directional(BASE_RIGHTWARD, -self.left_movement)
    }

    #[inline]
    fn move_up(&mut self) {
        self.move_directional(BASE_UPWARD, self.up_movement)
    }

    #[inline]
    fn move_down(&mut self) {
        self.move_directional(BASE_UPWARD, -self.down_movement)
    }

    #[inline]
    fn move_forward(&mut self) {
        self.move_directional(BASE_FORWARD, self.forward_movement);
    }

    #[inline]
    fn move_backward(&mut self) {
        self.move_directional(BASE_FORWARD, -self.backward_movement);
    }

    #[inline]
    fn move_directional(&mut self, direction: Vec3, movement: f64) {
        let offset = direction * -movement;
        self.view = Mat4::from_translation(&offset) * self.view;
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    #[inline]
    fn rotate_y(&mut self) {
        self.rotate(0.0, self.y_rotation, 0.0)
    }

    #[inline]
    fn rotate_ny(&mut self) {
        self.rotate(0.0, -self.y_rotation, 0.0)
    }

    #[inline]
    fn rotate_x(&mut self) {
        self.rotate(self.x_rotation, 0.0, 0.0)
    }

    #[inline]
    fn rotate_nx(&mut self) {
        self.rotate(-self.x_rotation, 0.0, 0.0)
    }

    #[inline]
    fn rotate_z(&mut self) {
        self.rotate(0.0, 0.0, self.z_rotation)
    }

    #[inline]
    fn rotate_nz(&mut self) {
        self.rotate(0.0, 0.0, -self.z_rotation)
    }

    #[inline]
    fn rotate(&mut self, rx: f64, ry: f64, rz: f64) {
        if ry != 0.0 {
            let r = match self
                .view
                .invert() // inverts the view matrix, gets a camera transform matrix
                .map(|rto| rto.transpose()) // transposes it, makes it available to transform a vector
                .map(|trto| BASE_UPWARD.transform_mat4(&trto).normalize()) // transforms BASE_UPWARD vector by that matrix, we gets the Y axis of the view matrix but representing in world space
                .and_then(|up| Mat4::from_rotation(ry,&up)) // then, makes a rotation matrix from it
            {
                Ok(r) => r,
                Err(err) => {
                    warn!(
                        target: "UniversalCamera",
                        "unexpected rotation: {err}"
                    );
                    return;
                }
            };

            // finally, applies the rotation matrix to the view matrix, makes it always rotates around the Y axis of the WORLD SPACE
            self.view = r * self.view;
        }
        if rx != 0.0 {
            self.view = Mat4::from_x_rotation(rx) * self.view;
        }
        if rz != 0.0 {
            self.view = Mat4::from_z_rotation(rz) * self.view;
        }

        self.view_proj = self.proj * self.view;

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
        self.frustum = frustum(&self.view, self.fovy, self.aspect, self.near, self.far);
    }
}

impl Drop for Shareable {
    fn drop(&mut self) {
        let Some(canvas) = self.binding_canvas.take() else {
            return;
        };

        if let Some(callback) = self.keydown_callback.take() {
            if let Err(err) = canvas
                .remove_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
            {
                error!(
                    target: "UniversalCamera",
                    "failed to unbind keydown event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }
        }

        if let Some(callback) = self.keyup_callback.take() {
            if let Err(err) = canvas
                .remove_event_listener_with_callback("keyup", callback.as_ref().unchecked_ref())
            {
                error!(
                    target: "UniversalCamera",
                    "failed to unbind keyup event: {}",
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

        if let Some(callback) = self.wheel_callback.take() {
            if let Err(err) = canvas
                .remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
            {
                error!(
                    target: "UniversalCamera",
                    "failed to unbind wheel event: {}",
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

        let view = Mat4::from_look_at(&position, &center, &up);
        let proj = Mat4::from_perspective(fovy, aspect, near, far);
        let frustum = frustum(&view, fovy, aspect, near, far);

        let default_movement = 1.0;
        let default_rotation = PI / 360.0;

        let sharable = Shareable {
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
            z_rotation: default_rotation,

            binding_canvas: None,
            keys_pressed: HashSet::new(),
            previous_timestamp: None,
            keydown_callback: None,
            keyup_callback: None,
            mousemove_callback: None,
            wheel_callback: None,
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

    pub fn rotate(&mut self, rx: f64, ry: f64, rz: f64) {
        self.sharable.borrow_mut().rotate(rx, ry, rz)
    }

    pub fn rotate_y(&mut self) {
        self.sharable.borrow_mut().rotate_y()
    }

    pub fn rotate_ny(&mut self) {
        self.sharable.borrow_mut().rotate_ny()
    }

    pub fn rotate_x(&mut self) {
        self.sharable.borrow_mut().rotate_x()
    }

    pub fn rotate_nx(&mut self) {
        self.sharable.borrow_mut().rotate_nx()
    }

    pub fn rotate_z(&mut self) {
        self.sharable.borrow_mut().rotate_z()
    }

    pub fn rotate_nz(&mut self) {
        self.sharable.borrow_mut().rotate_nz()
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

    pub fn update_frame(&mut self, state: &State) {
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
            shareable.keydown_callback = Some(Closure::new(move |event: KeyboardEvent| {
                let Some(shareable) = shareable_weak.upgrade() else {
                    return;
                };
                let mut shareable = shareable.borrow_mut();

                let key = event.key();
                match key.as_str() {
                    "w" | "a" | "s" | "d" | "ArrowUp" | "ArrowDown" | "ArrowLeft"
                    | "ArrowRight" => {
                        shareable.keys_pressed.insert(key);

                        event.prevent_default();
                        event.stop_propagation();
                    }
                    _ => return,
                }
            }));
            if let Err(err) = canvas.add_event_listener_with_callback(
                "keydown",
                shareable
                    .keydown_callback
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
            shareable.keyup_callback = Some(Closure::new(move |event: KeyboardEvent| {
                let Some(shareable) = shareable_weak.upgrade() else {
                    return;
                };
                let mut shareable = shareable.borrow_mut();

                let key = event.key();
                match key.as_str() {
                    "w" | "a" | "s" | "d" | "ArrowUp" | "ArrowDown" | "ArrowLeft"
                    | "ArrowRight" => {
                        shareable.keys_pressed.remove(&key);

                        event.prevent_default();
                        event.stop_propagation();
                    }
                    _ => return,
                }
            }));
            if let Err(err) = canvas.add_event_listener_with_callback(
                "keyup",
                shareable
                    .keyup_callback
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

                    if event.shift_key() {
                        let px = p.x();
                        let x = event.x();
                        let ox = x - px;

                        let oz = ox as f64 * shareable.z_rotation;
                        shareable.rotate(0.0, 0.0, oz);
                    } else {
                        let px = p.x();
                        let py = p.y();
                        let x = event.x();
                        let y = event.y();
                        let ox = x - px;
                        let oy = y - py;

                        let rx = oy as f64 * shareable.x_rotation;
                        let ry = ox as f64 * shareable.y_rotation;
                        shareable.rotate(rx, ry, 0.0);
                    }

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

            let shareable_weak = Rc::downgrade(&self.sharable);
            shareable.wheel_callback = Some(Closure::new(move |event: WheelEvent| {
                let Some(shareable) = shareable_weak.upgrade() else {
                    return;
                };
                let mut shareable = shareable.borrow_mut();

                let forward_movement = shareable.forward_movement;
                let backward_movement = shareable.backward_movement;

                let delta_y = event.delta_y() / 100.0;
                if delta_y < 0.0 {
                    shareable.move_directional(BASE_FORWARD, forward_movement / 2.0);
                } else if delta_y > 0.0 {
                    shareable.move_directional(BASE_FORWARD, -backward_movement / 2.0);
                }
            }));
            if let Err(err) = canvas.add_event_listener_with_callback(
                "wheel",
                shareable
                    .wheel_callback
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            ) {
                error!(
                    target: "UniversalCamera",
                    "failed to bind wheel event: {}",
                    err.as_string().map(|err| Cow::Owned(err)).unwrap_or(Cow::Borrowed("unknown reason")),
                );
            }

            shareable.binding_canvas = Some(canvas.clone());
        }

        // iterate keys pressed
        if !shareable.keys_pressed.is_empty() {
            let current = state.timestamp();
            let Some(previous) = shareable.previous_timestamp else {
                shareable.previous_timestamp = Some(current);
                return;
            };

            let offset = current - previous;
            if offset > 500.0 {
                shareable.previous_timestamp = Some(current);
                return;
            }

            let keys_pressed: *const HashSet<String> = &shareable.keys_pressed;
            unsafe {
                let iter = (*keys_pressed).iter();

                let offset = offset / 1000.0;
                let forward_movement = shareable.forward_movement;
                let backward_movement = shareable.backward_movement;
                let right_movement = shareable.right_movement;
                let left_movement = shareable.left_movement;
                let up_movement = shareable.up_movement;
                let down_movement = shareable.down_movement;
                let y_rotation = shareable.y_rotation * 120.0;

                for key in iter {
                    match key.as_str() {
                        "w" => shareable.move_directional(BASE_FORWARD, offset * forward_movement),
                        "s" => {
                            shareable.move_directional(BASE_FORWARD, offset * -backward_movement)
                        }
                        "d" => shareable.move_directional(BASE_RIGHTWARD, offset * right_movement),
                        "a" => shareable.move_directional(BASE_RIGHTWARD, offset * -left_movement),
                        "ArrowUp" => shareable.move_directional(BASE_UPWARD, offset * up_movement),
                        "ArrowDown" => {
                            shareable.move_directional(BASE_UPWARD, offset * -down_movement)
                        }
                        "ArrowLeft" => shareable.rotate(0.0, offset * y_rotation, 0.0),
                        "ArrowRight" => shareable.rotate(0.0, offset * -y_rotation, 0.0),
                        _ => return,
                    }
                }
            }

            shareable.previous_timestamp = Some(current);
        }
    }
}

impl Camera for UniversalCamera {
    fn position(&self) -> Vec3 {
        self.sharable.borrow().view.invert().unwrap().translation()
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
}

impl Default for UniversalCamera {
    fn default() -> Self {
        Self::new(
            Vec3::from_values(0.0, 0.0, 2.0),
            Vec3::new(),
            Vec3::from_values(0.0, 1.0, 0.0),
            60.0f64.to_radians(),
            1.0,
            0.5,
            None,
        )
    }
}

fn frustum(view: &Mat4, fovy: f64, aspect: f64, near: f64, far: Option<f64>) -> ViewFrustum {
    let x = Vec3::from_values(view.m00(), view.m10(), view.m20());
    let y = Vec3::from_values(view.m01(), view.m11(), view.m21());
    let nz = Vec3::from_values(view.m02(), view.m12(), view.m22());
    let z = nz.negate();
    let position = view.invert().unwrap().translation();

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
        Some(far) => Some(Plane::new(p + z * far, z)),
        None => None,
    };

    ViewFrustum::new(left, right, top, bottom, near, far)
}
