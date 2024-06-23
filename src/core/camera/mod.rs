use gl_matrix4rust::mat4::Mat4;

use super::{frustum::ViewFrustum, AsAny};

pub trait Camera: AsAny {
    fn view_matrix(&self) -> &Mat4<f64>;

    fn projection_matrix(&self) -> &Mat4<f64>;

    fn view_projection_matrix(&self) -> &Mat4<f64>;

    fn view_frustum(&self) -> &ViewFrustum;
}
