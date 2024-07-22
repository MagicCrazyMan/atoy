use std::time::Duration;

use js_sys::Promise;
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};

use crate::{
    anewthing::{
        channel::{Channel, Context, Handler}, clock::Tick, scheduler::{SchedulingTasks, Task}, web::app::App
    },
    core::web::window,
};

use super::clock::WebClock;

#[wasm_bindgen]
pub fn demo() -> App {
    let app = crate::anewthing::app::App::new(WebClock::new(Duration::from_secs(1)));
    app.channel().on(DemoHandler);
    App::new(app)
}
