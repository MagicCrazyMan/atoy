use std::{borrow::Cow, time::Duration};

use super::{channel::MessageKind, AsAny};

/// Clock ticking message target.
pub const CLOCK_TICKING_MESSAGE_TARGET: &'static str = "ClockTicking";

/// Clock tick indicating clock ticking information.
#[derive(Clone, Copy, PartialEq)]
pub struct Tick {
    start_time: f64,
    previous_time: Option<f64>,
    current_time: f64,
}

impl MessageKind for Tick {
    type Payload = Tick;

    fn target() -> Cow<'static, str>
    where
        Self: Sized,
    {
        Cow::Borrowed(CLOCK_TICKING_MESSAGE_TARGET)
    }
}

impl Tick {
    /// Constructs a new clock tick.
    pub fn new(
        start_time: f64,
        previous_time: Option<f64>,
        current_time: f64,
    ) -> Self {
        Self {
            start_time,
            previous_time,
            current_time,
        }
    }

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
}

/// A trait defining a clock.
pub trait Clock: AsAny {
    /// Returns the time when clock started.
    fn start_time(&self) -> Option<f64>;

    /// Returns the time when clock stopped.
    fn stop_time(&self) -> Option<f64>;

    /// Returns previous tick time if exists.
    fn previous_time(&self) -> Option<f64>;

    /// Returns `true` if this clock is ticking.
    fn ticking(&self) -> bool;

    /// Starts the clock.
    fn start(&mut self, interval: Duration);

    /// Stops the clock.
    fn stop(&mut self);
}
