use std::{any::Any, cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};

use crate::{
    core::{
        channel::Sender,
        clock::{Clock, Tick},
        AsAny,
    },
    performance, window,
};

/// A [`Clock`] implemented by [`Performance`](web_sys::Performance) from Web JavaScript.
pub struct WebClock {
    start_time: Rc<RefCell<Option<f64>>>,
    stop_time: Option<f64>,
    previous_time: Rc<RefCell<Option<f64>>>,
    interval: Option<Duration>,

    handle: Option<i32>,
    handler: Closure<dyn FnMut()>,
}

impl Drop for WebClock {
    fn drop(&mut self) {
        self.stop();
    }
}

impl AsAny for WebClock {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clock for WebClock {
    fn start_time(&self) -> Option<f64> {
        self.start_time.borrow().clone()
    }

    fn stop_time(&self) -> Option<f64> {
        self.stop_time.clone()
    }

    fn previous_time(&self) -> Option<f64> {
        self.previous_time.borrow().clone()
    }

    fn ticking(&self) -> bool {
        self.handle.is_some()
    }

    fn start(&mut self, interval: Duration) {
        if self.handle.is_some() {
            return;
        }

        let handle = window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                self.handler.as_ref().unchecked_ref(),
                interval.as_millis() as i32,
            )
            .expect("failed to set interval");

        *self.start_time.borrow_mut() = Some(performance().now());
        self.stop_time = None;
        self.interval = Some(interval);
        self.handle = Some(handle);
    }

    fn stop(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };

        self.stop_time = Some(performance().now());
        self.interval = None;
        window().clear_interval_with_handle(handle);
    }
}

impl WebClock {
    pub fn new(sender: Sender<Tick>) -> Self {
        let start_time = Rc::new(RefCell::new(None));
        let previous_time = Rc::new(RefCell::new(None));
        let handler = {
            let start_time = Rc::clone(&start_time);
            let previous_time = Rc::clone(&previous_time);
            Closure::new(move || {
                let current_time = performance().now();
                sender.send(Tick::new(
                    start_time.borrow().clone().unwrap(),
                    previous_time.borrow().clone(),
                    current_time,
                ));
                previous_time.borrow_mut().replace(current_time);
            })
        };

        Self {
            start_time,
            stop_time: None,
            previous_time,
            interval: None,

            handle: None,
            handler,
        }
    }

    pub fn elapsed_time(&self) -> Option<f64> {
        if let (Some(start_time), Some(stop_time)) = (self.start_time(), self.stop_time()) {
            Some(stop_time - start_time)
        } else {
            None
        }
    }

    pub fn interval(&self) -> Option<Duration> {
        self.interval.clone()
    }
}
