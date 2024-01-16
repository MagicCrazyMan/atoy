use std::any::Any;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};

use crate::{frustum::ViewFrustum, plane::Plane};

use super::Camera;

pub struct OrthogonalCamera {
    position: Vec3,
    center: Vec3,
    up: Vec3,
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    near: f64,
    far: f64,
    view: Mat4,
    proj: Mat4,
    view_proj: Mat4,
    frustum: ViewFrustum,
}

impl OrthogonalCamera {
    pub fn new(
        position: Vec3,
        center: Vec3,
        up: Vec3,
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
        far: f64,
    ) -> Self {
        let view = Mat4::<f64>::from_look_at(&position, &center, &up);
        let proj = Mat4::<f64>::from_ortho(left, right, bottom, top, near, far);
        let up = up.normalize();
        let frustum = frustum(position, center, up, left, right, bottom, top, near, far);
        Self {
            position,
            center,
            up,
            left,
            right,
            bottom,
            top,
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
        self.proj = Mat4::<f64>::from_ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        self.view_proj = self.proj * self.view;
        self.update_frustum();
    }

    fn update_frustum(&mut self) {
        self.frustum = frustum(
            self.position,
            self.center,
            self.up,
            self.left,
            self.right,
            self.bottom,
            self.top,
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

    pub fn left(&self) -> f64 {
        self.left
    }

    pub fn right(&self) -> f64 {
        self.right
    }

    pub fn bottom(&self) -> f64 {
        self.bottom
    }

    pub fn top(&self) -> f64 {
        self.top
    }

    pub fn near(&self) -> f64 {
        self.near
    }

    pub fn far(&self) -> f64 {
        self.far
    }

    pub fn set_left(&mut self, left: f64) {
        self.left = left;
        self.update_proj();
    }

    pub fn set_right(&mut self, right: f64) {
        self.right = right;
        self.update_proj();
    }

    pub fn set_bottom(&mut self, bottom: f64) {
        self.bottom = bottom;
        self.update_proj();
    }

    pub fn set_top(&mut self, top: f64) {
        self.top = top;
        self.update_proj();
    }

    pub fn set_near(&mut self, near: f64) {
        self.near = near;
        self.update_proj();
    }

    pub fn set_far(&mut self, far: f64) {
        self.far = far;
        self.update_proj();
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.update_view();
    }

    pub fn set_center(&mut self, center: Vec3) {
        self.center = center;
        self.update_view();
    }

    pub fn set_up(&mut self, mut up: Vec3) {
        up.normalize_in_place();
        self.up = up;
        self.update_view();
    }
}

impl Default for OrthogonalCamera {
    fn default() -> Self {
        Self::new(
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::<f64>::new_zero(),
            Vec3::new(0.0, 1.0, 0.0),
            -1.0,
            1.0,
            -1.0,
            1.0,
            0.5,
            1.5,
        )
    }
}

impl Camera for OrthogonalCamera {
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
}

pub(super) fn frustum(
    position: Vec3,
    center: Vec3,
    up: Vec3,
    left: f64,
    right: f64,
    bottom: f64,
    top: f64,
    near: f64,
    far: f64,
) -> ViewFrustum {
    let nz = (position - center).normalize();
    let x = up.cross(&nz).normalize();
    let y = nz.cross(&x).normalize();
    let z = -nz;

    let p = position + z * near;
    let hh = (right - left).abs() / 2.0;
    let hw = (top - bottom).abs() / 2.0;

    let top = {
        let pop = p + y * hh;
        Plane::new(pop, y)
    };
    let bottom = {
        let pop = p + y * -hh;
        Plane::new(pop, -y)
    };
    let left = {
        let pop = p + x * -hw;
        Plane::new(pop, -x)
    };
    let right = {
        let pop = p + x * hw;
        Plane::new(pop, x)
    };
    let near = { Plane::new(p, nz) };
    let far = {
        let pop = position + z * far;
        Plane::new(pop, z)
    };

    log::info!("{} {}", top.point_on_plane(), top.normal());
    log::info!("{} {}", bottom.point_on_plane(), bottom.normal());
    log::info!("{} {}", left.point_on_plane(), left.normal());
    log::info!("{} {}", right.point_on_plane(), right.normal());
    log::info!("{} {}", near.point_on_plane(), near.normal());
    log::info!("{} {}", far.point_on_plane(), far.normal());

    ViewFrustum::new(left, right, top, bottom, near, Some(far))
}
