use std::time::Duration;

use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use web_sys::window;

use crate::anewthing::{
    app::App,
    channel::{Context, Handler},
    clock::Tick,
};

use super::clock::WebClock;

#[wasm_bindgen]
pub struct WebApp(App);

#[wasm_bindgen]
impl WebApp {
    pub fn set_clock_interval(&mut self, interval: u32) {
        self.0.plugin_mut::<WebClock>().unwrap().set_interval(Duration::from_millis(interval as u64));
    }
}

#[wasm_bindgen]
pub fn demo() -> WebApp {
    let mut app = App::new();
    app.add_plugin(WebClock::new(Duration::from_secs(1)));
    app.channel().on(TickHandler);

    WebApp(app)
}

struct TickHandler;

impl Handler<Tick> for TickHandler {
    fn handle(&mut self, msg: &Tick, _: &mut Context) {
        log::info!("tick {}", msg.current_time);
    }
}
