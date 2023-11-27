pub mod camera;
pub mod entity;
pub mod geometry;
pub mod material;
pub mod render;
pub mod scene;
pub mod test;
pub mod error;
pub mod event;
pub mod utils;

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("failed to get window instance")
}

pub(crate) fn document() -> web_sys::Document {
    window()
        .document()
        .expect("failed to get document from window")
}
