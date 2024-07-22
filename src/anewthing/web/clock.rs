use std::{cell::RefCell, rc::Rc, time::Duration};

use async_trait::async_trait;
use wasm_bindgen::{closure::Closure, JsCast, prelude::wasm_bindgen};

use crate::{
    anewthing::{
        channel::Channel,
        clock::{Clock, Tick},
    },
    performance, window,
};

/// A [`Clock`] implemented by [`Performance`](web_sys::Performance) from Web JavaScript.
#[wasm_bindgen]
pub struct WebClock {
    start_time: Option<f64>,
    stop_time: Option<f64>,
    interval: Duration,

    handle: Option<i32>,
    handler: *mut Option<Closure<dyn FnMut()>>,
}

impl WebClock {
    pub fn new(interval: Duration) -> Self {
        Self {
            start_time: None,
            stop_time: None,
            interval,

            handle: None,
            handler: Box::into_raw(Box::new(None)),
        }
    }

    pub fn elapsed_time(&self) -> Option<f64> {
        if let (Some(start_time), Some(stop_time)) = (self.start_time(), self.stop_time()) {
            Some(stop_time - start_time)
        } else {
            None
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Drop for WebClock {
    fn drop(&mut self) {
        self.stop();

        unsafe {
            drop(Box::from_raw(self.handler));
        }
    }
}

#[async_trait(?Send)]
impl Clock for WebClock {
    fn start_time(&self) -> Option<f64> {
        self.start_time.clone()
    }

    fn stop_time(&self) -> Option<f64> {
        self.stop_time.clone()
    }

    fn running(&self) -> bool {
        self.handle.is_some()
    }

    fn start(&mut self, channel: Channel) {
        if self.running() {
            return;
        }
        log::info!("1111");

        unsafe {
            let start_time = performance().now();

            let handler = {
                let start_time = start_time;
                let previous_time = Rc::new(RefCell::new(start_time));
                Closure::new(move || {
                    log::info!("1111");
                    let current_time = performance().now();
                    channel.send(Tick::new(
                        start_time,
                        previous_time.borrow().clone(),
                        current_time,
                    ));
                    *previous_time.borrow_mut() = current_time;
                })
            };
            (*self.handler) = Some(handler);

            let handle = window()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    (*self.handler).as_ref().unwrap().as_ref().unchecked_ref(),
                    self.interval.as_millis() as i32,
                )
                .expect("failed to set interval");

            self.start_time = Some(start_time);
            self.stop_time = None;
            self.handle = Some(handle);
        }
    }

    fn stop(&mut self) {
        unsafe {
            if let Some(handle) = self.handle.take() {
                window().clear_interval_with_handle(handle);
            };

            (*self.handler) = None;
            self.stop_time = Some(performance().now());
        }
    }
}
