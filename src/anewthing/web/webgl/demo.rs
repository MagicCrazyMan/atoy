use std::{any::TypeId, str::FromStr, time::Duration};

use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use web_sys::window;

use crate::anewthing::{
    app::App,
    channel::{Context, Handler},
    clock::{SimpleTick, Tick},
    key::Key,
    plugin::Plugin,
};

use super::{clock::WebClock, renderer::WebGlRenderer};

pub struct TickOne(SimpleTick);

impl Tick for TickOne {
    fn new(start_time: i64, previous_time: i64, current_time: i64) -> Self
    where
        Self: Sized,
    {
        Self(SimpleTick::new(start_time, previous_time, current_time))
    }

    fn start_time(&self) -> i64 {
        self.0.start_time()
    }

    fn current_time(&self) -> i64 {
        self.0.current_time()
    }

    fn previous_time(&self) -> i64 {
        self.0.previous_time()
    }

    fn elapsed_time(&self) -> i64 {
        self.0.elapsed_time()
    }
}

pub struct TickTwo(SimpleTick);

impl Tick for TickTwo {
    fn new(start_time: i64, previous_time: i64, current_time: i64) -> Self
    where
        Self: Sized,
    {
        Self(SimpleTick::new(start_time, previous_time, current_time))
    }

    fn start_time(&self) -> i64 {
        self.0.start_time()
    }

    fn current_time(&self) -> i64 {
        self.0.current_time()
    }

    fn previous_time(&self) -> i64 {
        self.0.previous_time()
    }

    fn elapsed_time(&self) -> i64 {
        self.0.elapsed_time()
    }
}

#[wasm_bindgen]
pub struct WebApp(App);

#[wasm_bindgen]
impl WebApp {
    pub fn set_clock_one_interval(&mut self, interval: u32) {
        self.0
            .plugin_mut::<WebClock<TickOne>>()
            .unwrap()
            .set_interval(Duration::from_millis(interval as u64));
    }

    pub fn set_clock_two_interval(&mut self, interval: u32) {
        self.0
            .plugin_mut::<WebClock<TickTwo>>()
            .unwrap()
            .set_interval(Duration::from_millis(interval as u64));
    }
}

#[wasm_bindgen]
pub fn demo() -> Result<WebApp, JsValue> {
    let mut app = App::new(WebGlRenderer::new());
    app.add_plugin(WebClock::<TickOne>::new(Duration::from_secs(1)))
        .map_err(|_| JsValue::from_str("duplicated plugin"))?;
    app.add_plugin(WebClock::<TickTwo>::new(Duration::from_secs(2)))
        .map_err(|_| JsValue::from_str("duplicated plugin"))?;
    app.channel().on(TickOneHandler);
    app.channel().on(TickTwoHandler);

    Ok(WebApp(app))
}

struct TickOneHandler;

impl Handler<TickOne> for TickOneHandler {
    fn handle(&mut self, msg: &TickOne, _: &mut Context) {
        log::info!("clock 1 {}", msg.current_time());
    }
}

struct TickTwoHandler;

impl Handler<TickTwo> for TickTwoHandler {
    fn handle(&mut self, msg: &TickTwo, _: &mut Context) {
        log::info!("clock 2 {}", msg.current_time());
    }
}
