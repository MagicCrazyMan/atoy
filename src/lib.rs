use std::sync::OnceLock;

use wasm_bindgen::{closure::Closure, JsCast};

pub mod bounding;
pub mod camera;
pub mod controller;
pub mod entity;
pub mod error;
pub mod event;
pub mod frustum;
pub mod geometry;
pub mod light;
pub mod lru;
pub mod material;
pub mod plane;
pub mod render;
pub mod scene;
pub mod test;
pub mod utils;
pub mod viewer;

pub(crate) fn window() -> web_sys::Window {
    web_sys::window().expect("failed to get window instance")
}

pub(crate) fn document() -> web_sys::Document {
    window()
        .document()
        .expect("failed to get document from window")
}

pub(crate) fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("failed to invoke requestAnimationFrame");
}

const INITIALIZED: OnceLock<bool> = OnceLock::new();

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init() {
    init_logger(log::LevelFilter::Info);
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init_with_log_level(level: LogLevel) {
    init_logger(level.to_native())
}

#[wasm_bindgen::prelude::wasm_bindgen]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

impl LogLevel {
    fn to_native(&self) -> log::LevelFilter {
        match self {
            LogLevel::Trace => log::LevelFilter::Trace,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Off => log::LevelFilter::Off,
        }
    }
}

fn init_logger(level: log::LevelFilter) {
    if INITIALIZED.get().cloned().unwrap_or(false) {
        return;
    }

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

    INITIALIZED
        .set(true)
        .expect("failed to init console logger");
}
