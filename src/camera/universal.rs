use std::{any::Any, cell::RefCell, collections::HashSet, f64::consts::PI, rc::Rc};

use crate::{controller::Controller, frustum::ViewFrustum, plane::Plane, viewer::Viewer};
use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::{AsVec3, Vec3},
};
use log::warn;
use uuid::Uuid;
use web_sys::MouseEvent;

use super::Camera;

const BASE_RIGHTWARD: Vec3 = Vec3::from_values(1.0, 0.0, 0.0);
const BASE_UPWARD: Vec3 = Vec3::from_values(0.0, 1.0, 0.0);
// camera coordinate system is a right hand side coordinate system
// flip z axis to convert it to left hand side
const BASE_FORWARD: Vec3 = Vec3::from_values(0.0, 0.0, -1.0);

struct Control {
    pressed_keys: HashSet<String>,
    previous_timestamp: Option<f64>,
    previous_mouse_event: Option<MouseEvent>,
    key_down_listener: Uuid,
    key_up_listener: Uuid,
    mouse_move_listener: Uuid,
    wheel_listener: Uuid,
    canvas_changed_listener: Uuid,
    pre_render_listener: Uuid,
}

struct Inner {
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
}

impl Inner {
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

/// An first personal based, controllable perspective camera with mouse, keyboard or screen touching.
///
/// UniversalCamera is inner by cloning, making it convenient to control outside [`Scene`].
#[derive(Clone)]
pub struct UniversalCamera {
    inner: Rc<RefCell<Inner>>,
    control: Rc<RefCell<Option<Control>>>,
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

        let inner = Inner {
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
        };

        Self {
            inner: Rc::new(RefCell::new(inner)),
            control: Rc::new(RefCell::new(None)),
        }
    }
}

impl UniversalCamera {
    pub fn move_right(&mut self) {
        self.inner.borrow_mut().move_right()
    }

    pub fn move_left(&mut self) {
        self.inner.borrow_mut().move_left()
    }

    pub fn move_up(&mut self) {
        self.inner.borrow_mut().move_up()
    }

    pub fn move_down(&mut self) {
        self.inner.borrow_mut().move_down()
    }

    pub fn move_forward(&mut self) {
        self.inner.borrow_mut().move_forward()
    }

    pub fn move_backward(&mut self) {
        self.inner.borrow_mut().move_backward()
    }

    pub fn rotate(&mut self, rx: f64, ry: f64, rz: f64) {
        self.inner.borrow_mut().rotate(rx, ry, rz)
    }

    pub fn rotate_y(&mut self) {
        self.inner.borrow_mut().rotate_y()
    }

    pub fn rotate_ny(&mut self) {
        self.inner.borrow_mut().rotate_ny()
    }

    pub fn rotate_x(&mut self) {
        self.inner.borrow_mut().rotate_x()
    }

    pub fn rotate_nx(&mut self) {
        self.inner.borrow_mut().rotate_nx()
    }

    pub fn rotate_z(&mut self) {
        self.inner.borrow_mut().rotate_z()
    }

    pub fn rotate_nz(&mut self) {
        self.inner.borrow_mut().rotate_nz()
    }

    pub fn fovy(&self) -> f64 {
        self.inner.borrow().fovy()
    }

    pub fn aspect(&self) -> f64 {
        self.inner.borrow().aspect()
    }

    pub fn near(&self) -> f64 {
        self.inner.borrow().near()
    }

    pub fn far(&self) -> Option<f64> {
        self.inner.borrow().far()
    }

    pub fn set_fovy(&mut self, fovy: f64) {
        self.inner.borrow_mut().set_fovy(fovy)
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.inner.borrow_mut().set_aspect(aspect)
    }

    pub fn set_near(&mut self, near: f64) {
        self.inner.borrow_mut().set_near(near)
    }

    pub fn set_far(&mut self, far: Option<f64>) {
        self.inner.borrow_mut().set_far(far)
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.inner.borrow_mut().set_position(position)
    }

    pub fn left_movement(&self) -> f64 {
        self.inner.borrow().left_movement()
    }

    pub fn right_movement(&self) -> f64 {
        self.inner.borrow().right_movement()
    }

    pub fn up_movement(&self) -> f64 {
        self.inner.borrow().up_movement()
    }

    pub fn down_movement(&self) -> f64 {
        self.inner.borrow().down_movement()
    }

    pub fn forward_movement(&self) -> f64 {
        self.inner.borrow().forward_movement()
    }

    pub fn backward_movement(&self) -> f64 {
        self.inner.borrow().backward_movement()
    }

    pub fn set_left_movement(&mut self, left_movement: f64) {
        self.inner.borrow_mut().set_left_movement(left_movement)
    }

    pub fn set_right_movement(&mut self, right_movement: f64) {
        self.inner.borrow_mut().set_right_movement(right_movement)
    }

    pub fn set_up_movement(&mut self, up_movement: f64) {
        self.inner.borrow_mut().set_up_movement(up_movement)
    }

    pub fn set_down_movement(&mut self, down_movement: f64) {
        self.inner.borrow_mut().set_down_movement(down_movement)
    }

    pub fn set_forward_movement(&mut self, forward_movement: f64) {
        self.inner
            .borrow_mut()
            .set_forward_movement(forward_movement)
    }

    pub fn set_backward_movement(&mut self, backward_movement: f64) {
        self.inner
            .borrow_mut()
            .set_backward_movement(backward_movement)
    }
}

impl Camera for UniversalCamera {
    fn position(&self) -> Vec3 {
        self.inner.borrow().view.invert().unwrap().translation()
    }

    fn view_matrix(&self) -> Mat4 {
        self.inner.borrow().view
    }

    fn proj_matrix(&self) -> Mat4 {
        self.inner.borrow().proj
    }

    fn view_proj_matrix(&self) -> Mat4 {
        self.inner.borrow().view_proj
    }

    fn view_frustum(&self) -> ViewFrustum {
        self.inner.borrow().frustum
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Controller for UniversalCamera {
    fn on_add(&mut self, viewer: &mut Viewer) {
        if self.control.borrow_mut().is_some() {
            panic!("share UniversalCamera between different viewer is not allowed");
        }

        let mut render = viewer.render().borrow_mut();

        let key_down_listener = {
            let mut viewer = viewer.clone();
            let control = Rc::clone(&self.control);
            render.key_down_event().on(move |event| {
                let key = event.key();
                match key.as_str() {
                    "w" | "a" | "s" | "d" | "W" | "A" | "S" | "D" | "ArrowUp" | "ArrowDown"
                    | "ArrowLeft" | "ArrowRight" => {
                        control
                            .borrow_mut()
                            .as_mut()
                            .unwrap()
                            .pressed_keys
                            .insert(key);
                        event.prevent_default();
                        event.stop_propagation();
                        viewer.should_render_next();
                    }
                    _ => return,
                }
            })
        };

        let key_up_listener = {
            let mut viewer = viewer.clone();
            let control = Rc::clone(&self.control);
            render.key_up_event().on(move |event| {
                let mut control = control.borrow_mut();
                let control = control.as_mut().unwrap();
                let key = event.key();
                match key.as_str() {
                    "w" | "a" | "s" | "d" | "W" | "A" | "S" | "D" | "ArrowUp" | "ArrowDown"
                    | "ArrowLeft" | "ArrowRight" => {
                        control.pressed_keys.remove(&key);
                        if control.pressed_keys.len() > 0 {
                            viewer.should_render_next();
                        }

                        event.prevent_default();
                        event.stop_propagation();
                    }
                    _ => return,
                }
            })
        };

        let mouse_move_listener = {
            let mut viewer = viewer.clone();
            let inner = Rc::clone(&self.inner);
            let control = Rc::clone(&self.control);
            render.mouse_move_event().on(move |event| {
                let mut control = control.borrow_mut();
                let previous_mouse_event = &mut control.as_mut().unwrap().previous_mouse_event;

                // 4 refers to middle button
                if event.buttons() == 4 {
                    let Some(p) = previous_mouse_event.take() else {
                        *previous_mouse_event = Some(event.clone());
                        return;
                    };

                    let mut inner = inner.borrow_mut();
                    if event.shift_key() {
                        let px = p.x();
                        let x = event.x();
                        let ox = x - px;

                        let oz = ox as f64 * inner.z_rotation;
                        inner.rotate(0.0, 0.0, oz);
                    } else {
                        let px = p.x();
                        let py = p.y();
                        let x = event.x();
                        let y = event.y();
                        let ox = x - px;
                        let oy = y - py;

                        let rx = oy as f64 * inner.x_rotation;
                        let ry = ox as f64 * inner.y_rotation;
                        inner.rotate(rx, ry, 0.0);
                    }

                    event.prevent_default();
                    event.stop_propagation();

                    *previous_mouse_event = Some(event.clone());
                    viewer.should_render_next();
                } else {
                    *previous_mouse_event = None;
                }
            })
        };

        let wheel_listener = {
            let mut viewer = viewer.clone();
            let inner = Rc::clone(&self.inner);
            render.wheel_event().on(move |event| {
                let mut inner = inner.borrow_mut();

                let forward_movement = inner.forward_movement;
                let backward_movement = inner.backward_movement;

                let delta_y = event.delta_y() / 100.0;
                if delta_y < 0.0 {
                    inner.move_directional(BASE_FORWARD, forward_movement / 2.0);
                    viewer.should_render_next();
                } else if delta_y > 0.0 {
                    inner.move_directional(BASE_FORWARD, -backward_movement / 2.0);
                    viewer.should_render_next();
                }
            })
        };

        let pre_render_listener = {
            let mut viewer = viewer.clone();
            let inner = Rc::clone(&self.inner);
            let control = Rc::clone(&self.control);
            render.pre_render_event().on(move |event| {
                let mut control = control.borrow_mut();
                let control = control.as_mut().unwrap();

                if control.pressed_keys.is_empty() {
                    return;
                }

                viewer.should_render_next();
                let timestamp = event.state().timestamp();

                let Some(previous) = control.previous_timestamp else {
                    control.previous_timestamp = Some(timestamp);
                    return;
                };

                let offset = timestamp - previous;
                control.previous_timestamp = Some(timestamp);

                if offset > 500.0 {
                    return;
                }

                let mut inner = inner.borrow_mut();
                let offset = offset / 1000.0;
                let forward_movement = inner.forward_movement;
                let backward_movement = inner.backward_movement;
                let right_movement = inner.right_movement;
                let left_movement = inner.left_movement;
                let up_movement = inner.up_movement;
                let down_movement = inner.down_movement;
                let y_rotation = inner.y_rotation * 120.0;
                for key in control.pressed_keys.iter() {
                    match key.as_str() {
                        "w" | "W" => {
                            inner.move_directional(BASE_FORWARD, offset * forward_movement)
                        }
                        "s" | "S" => {
                            inner.move_directional(BASE_FORWARD, offset * -backward_movement)
                        }
                        "d" | "D" => {
                            inner.move_directional(BASE_RIGHTWARD, offset * right_movement)
                        }
                        "a" | "A" => {
                            inner.move_directional(BASE_RIGHTWARD, offset * -left_movement)
                        }
                        "ArrowUp" => inner.move_directional(BASE_UPWARD, offset * up_movement),
                        "ArrowDown" => inner.move_directional(BASE_UPWARD, offset * -down_movement),
                        "ArrowLeft" => inner.rotate(0.0, offset * y_rotation, 0.0),
                        "ArrowRight" => inner.rotate(0.0, offset * -y_rotation, 0.0),
                        _ => return,
                    }
                }
            })
        };

        let canvas_changed_listener = {
            let mut viewer = viewer.clone();
            let inner = Rc::clone(&self.inner);
            render.canvas_changed_event().on(move |event| {
                let mut inner = inner.borrow_mut();
                let canvas = event.canvas();

                let aspect = canvas.width() as f64 / canvas.height() as f64;
                if aspect != inner.aspect {
                    inner.set_aspect(aspect);
                    viewer.should_render_next();
                }
            })
        };

        *self.control.borrow_mut() = Some(Control {
            pressed_keys: HashSet::new(),
            previous_timestamp: None,
            previous_mouse_event: None,
            key_down_listener,
            key_up_listener,
            mouse_move_listener,
            wheel_listener,
            canvas_changed_listener,
            pre_render_listener,
        });
    }

    fn on_remove(&mut self, viewer: &mut Viewer) {
        let mut control = self.control.borrow_mut();
        let Some(control) = control.take() else {
            return;
        };

        let mut render = viewer.render().borrow_mut();
        render.key_down_event().off(&control.key_down_listener);
        render.key_up_event().off(&control.key_up_listener);
        render.mouse_move_event().off(&control.mouse_move_listener);
        render.wheel_event().off(&control.wheel_listener);
        render
            .canvas_changed_event()
            .off(&control.canvas_changed_listener);
        render.pre_render_event().off(&control.pre_render_listener);
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
