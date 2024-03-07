use crate::{clock::Clock, scene::Scene};

pub mod webgl;

/// A rendering pipeline.
pub trait Pipeline {
    /// Runtime state.
    type State;

    /// Scene clock
    type Clock: Clock;

    /// Error that could be thrown during execution.
    type Error;

    /// Executes this rendering pipeline with specified `State` and a [`Scene`].
    fn execute(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene<Self::Clock>,
    ) -> Result<(), Self::Error>;
}
