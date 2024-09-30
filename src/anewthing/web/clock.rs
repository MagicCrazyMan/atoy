use std::{cell::RefCell, marker::PhantomData, rc::Rc, time::Duration};

use tokio::sync::broadcast::{self, Receiver, Sender};
use wasm_bindgen::{closure::Closure, JsCast};

use crate::{
    anewthing::{app::App, clock::Tick, plugin::Plugin},
    performance, window,
};

/// A [`Clock`] implemented by [`Performance`](web_sys::Performance) from Web JavaScript.
pub struct WebClock<T> {
    start_on_plugin: bool,
    stop_on_plugout: bool,
    start_time: Option<i64>,
    stop_time: Option<i64>,
    interval: Duration,

    sender: Sender<T>,
    handler: *mut Option<Closure<dyn FnMut()>>,
    handle: Option<i32>,

    _tick: PhantomData<T>,
}

impl<T> WebClock<T>
where
    T: Tick + 'static,
{
    /// Construct a new web clock based on [`Performance`](https://developer.mozilla.org/en-US/docs/Web/API/Performance).
    ///
    /// `interval` is the duration between each clock tick.
    pub fn new(interval: Duration) -> Self {
        Self {
            start_on_plugin: true,
            stop_on_plugout: true,
            start_time: None,
            stop_time: None,
            interval,

            sender: broadcast::channel(5).0,
            handler: Box::into_raw(Box::new(None)),
            handle: None,

            _tick: PhantomData,
        }
    }

    /// Returns a message receiver associated with this clock.
    pub fn receiver(&self) -> Receiver<T> {
        self.sender.subscribe()
    }

    /// Sets whether automatically start the clock when plugin.
    pub fn set_start_on_plugin(&mut self, enable: bool) {
        self.start_on_plugin = enable;
    }

    /// Sets whether automatically stop the clock when plugout.
    pub fn set_stop_on_plugout(&mut self, enable: bool) {
        self.stop_on_plugout = enable;
    }

    /// Returns the interval between each clock tick.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Sets the interval between each clock tick.
    pub fn set_interval(&mut self, interval: Duration) {
        if interval == self.interval {
            return;
        }

        self.interval = interval;
        if self.running() {
            self.stop();
            self.start();
        }
    }

    /// Returns the time when clock started in milliseconds.
    pub fn start_time(&self) -> Option<i64> {
        self.start_time.clone()
    }

    /// Returns the time when clock stopped in milliseconds.
    pub fn stop_time(&self) -> Option<i64> {
        self.stop_time.clone()
    }

    /// Returns current time in milliseconds.
    pub fn current_time(&self) -> i64 {
        performance().now() as i64
    }

    /// Returns the elapsed time of the clock in milliseconds.
    ///
    /// - If clock is running, returns the time between start time and current time.
    /// - If clock is not running, returns the time between start time and stop time.
    pub fn elapsed_time(&self) -> Option<i64> {
        if self.running() {
            Some(self.current_time() - self.start_time().unwrap())
        } else {
            if let (Some(start_time), Some(stop_time)) = (self.start_time(), self.stop_time()) {
                Some(stop_time - start_time)
            } else {
                None
            }
        }
    }

    /// Returns `true` if clock is ticking.
    /// [`WebClock::start_time`] is promised to return some value when clock is running.
    pub fn running(&self) -> bool {
        self.handle.is_some()
    }

    /// Starts the clock.
    ///
    /// [`Tick`] message will be sent to message channel at intervals after started.
    pub fn start(&mut self) {
        if self.running() {
            return;
        }

        unsafe {
            let start_time = performance().now() as i64;

            let handler = {
                let start_time = start_time;
                let previous_time = Rc::new(RefCell::new(start_time));
                let sender = self.sender.clone();
                Closure::new(move || {
                    let current_time = performance().now() as i64;
                    let _ = sender.send(T::new(start_time, *previous_time.borrow(), current_time));

                    *previous_time.borrow_mut() = current_time;
                })
            };
            *self.handler = Some(handler);

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

    /// Stops the clock.
    pub fn stop(&mut self) {
        unsafe {
            if let Some(handle) = self.handle.take() {
                window().clear_interval_with_handle(handle);
            };

            (*self.handler) = None;
            self.stop_time = Some(performance().now() as i64);
        }
    }
}

impl<T> Plugin for WebClock<T>
where
    T: Tick + 'static,
{
    fn plugin(&mut self, _: &mut App) {
        if self.start_on_plugin {
            self.start();
        }
    }

    fn plugout(&mut self, _: &mut App) {
        if self.stop_on_plugout {
            self.stop();
        }
    }
}

impl<T> Drop for WebClock<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.handler));
        }
    }
}
