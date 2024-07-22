/// Clock tick indicating clock ticking information.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    /// Start time of the clock.
    pub start_time: i64,
    /// Current time of the clock.
    pub current_time: i64,
    /// Previous tick time of the clock.
    pub previous_time: i64,
    /// Delta time between previous time and current time.
    pub delta_time: i64,
}

impl Tick {
    /// Constructs a new clock tick.
    pub fn new(start_time: i64, previous_time: i64, current_time: i64) -> Self {
        Self {
            start_time,
            previous_time,
            current_time,
            delta_time: current_time - previous_time,
        }
    }
}
