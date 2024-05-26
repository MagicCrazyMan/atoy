pub mod clock;
pub mod loader;
pub mod looper;
pub mod webgl;

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("failed to get window instance")
}

pub(crate) fn document() -> web_sys::Document {
    window()
        .document()
        .expect("failed to get document from window")
}

pub(crate) fn performance() -> web_sys::Performance {
    window()
        .performance()
        .expect("failed to get performance instance from window")
}

pub(crate) fn request_animation_frame(f: &wasm_bindgen::closure::Closure<dyn FnMut(f64)>) -> i32 {
    window()
        .request_animation_frame(wasm_bindgen::JsCast::unchecked_ref(f.as_ref()))
        .expect("failed to invoke requestAnimationFrame")
}

pub(crate) fn cancel_animation_frame(handle: i32) {
    window()
        .cancel_animation_frame(handle)
        .expect("failed to invoke cancelAnimationFrame");
}
