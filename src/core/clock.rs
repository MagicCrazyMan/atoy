use std::time::Duration;

use super::{app::AppConfig, engine::RenderEngine, resource::Resources, scene::Scene, AsAny, Rrc};

/// Clock tick indicating clock ticking information.
pub struct Tick {
    pub start_time: f64,
    pub previous_time: f64,
    pub current_time: f64,
}

impl Tick {
    /// Constructs a new clock tick.
    pub fn new(start_time: f64, previous_time: f64, current_time: f64) -> Self {
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
    pub fn previous_time(&self) -> f64 {
        self.previous_time
    }

    /// Returns current tick time.
    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    /// Returns delta time between current tick time and
    /// previous tick time if previous tick time exists.
    pub fn delta_time(&self) -> f64 {
        self.current_time - self.previous_time
    }
}

/// A trait defining a clock.
pub trait Clock: AsAny {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized;

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
}
