use std::{any::Any, cell::RefCell, rc::Rc, time::Duration};

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Performance, Window};

use crate::{
    channel::{channel, Receiver, Sender},
    window,
};

/// Clock tick indicating clock ticking information.
#[derive(Clone, Copy, PartialEq)]
pub struct Tick {
    start_time: f64,
    previous_time: Option<f64>,
    current_time: f64,
    interval: Duration,
}

impl Tick {
    /// Returns the time when clock started.
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    /// Returns previous tick time if exists.
    pub fn previous_time(&self) -> Option<f64> {
        self.previous_time.clone()
    }

    /// Returns current tick time.
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Returns delta time between current tick time and
    /// previous tick time if previous tick time exists.
    pub fn delta_time(&self) -> Option<f64> {
        if let Some(previous_time) = self.previous_time {
            Some(self.current_time - previous_time)
        } else {
            None
        }
    }

    /// Returns ticking interval of the clock.
    pub fn interval(&self) -> Duration {
        self.interval
    }
}

/// A trait defining a clock.
pub trait Clock {
    /// Returns the time when clock started.
    fn start_time(&self) -> Option<f64>;

    /// Returns the time when clock stopped.
    fn stop_time(&self) -> Option<f64>;

    /// Returns previous tick time if exists.
    fn previous_time(&self) -> Option<f64>;

    /// Returns `true` if this clock is ticking.
    fn running(&self) -> bool;

    /// Returns a [`MessageChannel`] for broadcasting ticking message.
    fn ticking(&self) -> Receiver<Tick>;

    /// Starts the clock.
    fn start(&mut self, interval: Duration);

    /// Stops the clock.
    fn stop(&mut self);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A [`Clock`] implemented by [`Performance`] from Web JavaScript.
pub struct HtmlClock {
    window: Window,
    performance: Performance,

    start_time: Option<f64>,
    stop_time: Option<f64>,
    previous_time: Rc<RefCell<Option<f64>>>,

    handle: Option<i32>,
    handler: Option<Closure<dyn FnMut()>>,
    channel: (Sender<Tick>, Receiver<Tick>),
}

impl Drop for HtmlClock {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Clock for HtmlClock {
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
        let sender = self.channel.0.clone();
        let handler = Some(Closure::new(move || {
            let current_time = performance.now();
            sender.send(&Tick {
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

    fn ticking(&self) -> Receiver<Tick> {
        self.channel.1.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl HtmlClock {
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
            channel: channel(),
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
