pub trait Tick: Clone {
    /// Constructs a new clock tick.
    fn new(start_time: i64, previous_time: i64, current_time: i64) -> Self
    where
        Self: Sized;

    /// Returns start time of the clock.
    fn start_time(&self) -> i64;

    /// Returns current time of this tick.
    fn current_time(&self) -> i64;

    /// Returns previous time of last tick.
    fn previous_time(&self) -> i64;

    /// Returns elapsed time between previous time and current time.
    fn elapsed_time(&self) -> i64;
}

/// Simle clock tick indicating clock ticking information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleTick {
    start_time: i64,
    current_time: i64,
    previous_time: i64,
    elapsed_time: i64,
}

impl Tick for SimpleTick {
    fn new(start_time: i64, previous_time: i64, current_time: i64) -> Self
    where
        Self: Sized,
    {
        Self {
            start_time,
            previous_time,
            current_time,
            elapsed_time: current_time - previous_time,
        }
    }

    fn start_time(&self) -> i64 {
        self.start_time
    }

    fn current_time(&self) -> i64 {
        self.current_time
    }

    fn previous_time(&self) -> i64 {
        self.previous_time
    }

    fn elapsed_time(&self) -> i64 {
        self.elapsed_time
    }
}
