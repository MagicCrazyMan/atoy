use std::{any::Any, cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Performance, Window};

use crate::{
    notify::{Notifiee, Notifier, Notifying}, share::Share, window
};

#[derive(Clone, Copy, PartialEq)]
pub struct Tick {
    start_time: f64,
    previous_time: Option<f64>,
    current_time: f64,
    interval: Duration,
}

impl Tick {
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    pub fn previous_time(&self) -> Option<f64> {
        self.previous_time.clone()
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn delta_time(&self) -> Option<f64> {
        if let Some(previous_time) = self.previous_time {
            Some(self.current_time - previous_time)
        } else {
            None
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

pub trait Clock {
    fn start_time(&self) -> Option<f64>;

    fn stop_time(&self) -> Option<f64>;

    fn previous_time(&self) -> Option<f64>;

    fn running(&self) -> bool;

    fn start(&mut self, interval: Duration);

    fn stop(&mut self);

    fn on_tick<N>(&mut self, notifiee: N) -> Notifying<Tick>
    where
        N: Notifiee<Tick> + 'static;

    fn un_tick(&mut self, key: usize);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct WebClock {
    window: Window,
    performance: Performance,

    start_time: Option<f64>,
    stop_time: Option<f64>,
    previous_time: Share<Option<f64>>,

    handle: Option<i32>,
    handler: Option<Closure<dyn FnMut()>>,
    notifier: Share<Notifier<Tick>>,
}

impl Drop for WebClock {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Clock for WebClock {
    fn start_time(&self) -> Option<f64> {
        self.start_time.clone()
    }

    fn stop_time(&self) -> Option<f64> {
        self.stop_time.clone()
    }

    fn previous_time(&self) -> Option<f64> {
        self.previous_time.borrow().clone()
    }

    fn running(&self) -> bool {
        self.handle.is_some()
    }

    fn start(&mut self, interval: Duration) {
        if self.handle.is_some() {
            return;
        }

        let start_time = self.performance.now();

        let previous_time = Rc::clone(&self.previous_time);
        let performance = self.performance.clone();
        let notifier = self.notifier.clone();
        let handler = Some(Closure::new(move || {
            let current_time = performance.now();
            notifier.borrow_mut().notify(&Tick {
                start_time,
                previous_time: previous_time.borrow().clone(),
                current_time,
                interval,
            });
            previous_time.borrow_mut().replace(current_time);
        }));
        let handle = self
            .window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                handler.as_ref().unwrap().as_ref().unchecked_ref(),
                interval.as_millis() as i32,
            )
            .expect("failed to set interval");

        self.start_time = Some(start_time);
        self.stop_time = None;
        self.handle = Some(handle);
        self.handler = handler;
    }

    fn stop(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
        self.window.clear_interval_with_handle(handle);
        self.handler.take();

        self.stop_time = Some(self.performance.now());
    }

    fn on_tick<N>(&mut self, notifiee: N) -> Notifying<Tick>
    where
        N: Notifiee<Tick> + 'static,
    {
        self.notifier.borrow_mut().register(notifiee)
    }

    fn un_tick(&mut self, key: usize) {
        self.notifier.borrow_mut().unregister(key);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl WebClock {
    pub fn new() -> Self {
        let window = window();
        Self {
            performance: window.performance().expect("failed to get web performance"),
            window,

            start_time: None,
            stop_time: None,
            previous_time: Rc::new(RefCell::new(None)),

            handle: None,
            handler: None,

            notifier: Rc::new(RefCell::new(Notifier::new())),
        }
    }

    pub fn elapsed_time(&self) -> Option<f64> {
        if let (Some(start_time), Some(stop_time)) = (self.start_time, self.stop_time) {
            Some(stop_time - start_time)
        } else {
            None
        }
    }
}
