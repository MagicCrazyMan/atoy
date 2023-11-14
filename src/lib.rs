pub mod camera;
pub mod entity;
pub mod geometry;
pub mod material;
pub mod render;
pub mod scene;
pub mod utils;

pub enum Ncor<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> AsRef<T> for Ncor<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Ncor::Borrowed(b) => *b,
            Ncor::Owned(o) => o,
        }
    }
}

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("failed to get window instance")
}

pub(crate) fn document() -> web_sys::Document {
    window()
        .document()
        .expect("failed to get document from window")
}
