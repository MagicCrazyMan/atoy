pub mod bounding;
pub mod camera;
pub mod entity;
pub mod error;
pub mod event;
pub mod frustum;
pub mod geometry;
pub mod material;
pub mod plane;
pub mod render;
pub mod scene;
pub mod test;
pub mod utils;

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("failed to get window instance")
}

pub(crate) fn document() -> web_sys::Document {
    window()
        .document()
        .expect("failed to get document from window")
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init() {
    init_logger(log::LevelFilter::Info);
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init_with_log_level(level: u32) {
    match level {
        0 => init_logger(log::LevelFilter::Trace),
        1 => init_logger(log::LevelFilter::Debug),
        2 => init_logger(log::LevelFilter::Info),
        3 => init_logger(log::LevelFilter::Warn),
        4 => init_logger(log::LevelFilter::Error),
        _ => init_logger(log::LevelFilter::Info),
    }
}

fn init_logger(level: log::LevelFilter) {
    utils::set_panic_hook();
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(fern::Output::call(console_log::log))
        .apply()
        .expect("failed to init console logger");
}
