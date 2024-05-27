use std::time::Duration;

use super::{carrier::Carrier, AsAny};

/// Clock tick indicating clock ticking information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub start_time: f64,
    pub previous_time: f64,
    pub current_time: f64,
    pub delta_time: f64,
}

impl Tick {
    /// Constructs a new clock tick.
    pub fn new(start_time: f64, previous_time: f64, current_time: f64) -> Self {
        Self {
            start_time,
            previous_time,
            current_time,
            delta_time: current_time - previous_time
        }
    }
}

/// A trait defining a clock.
pub trait Clock: AsAny {
    /// Returns the time when clock started.
    fn start_time(&self) -> Option<f64>;

    /// Returns the time when clock stopped.
    fn stop_time(&self) -> Option<f64>;

    /// Returns `true` if this clock is ticking.
    fn running(&self) -> bool;

    /// Starts the clock.
    fn start(&mut self, interval: Duration);

    /// Stops the clock.
    fn stop(&mut self);

    fn on_tick(&self) -> &Carrier<Tick>;
}
