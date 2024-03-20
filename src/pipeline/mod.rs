use crate::scene::Scene;

pub mod webgl;

/// A rendering pipeline.
pub trait Pipeline {
    /// Runtime state.
    type State;

    /// Error that could be thrown during execution.
    type Error;

    /// Executes this rendering pipeline with specified `State` and a [`Scene`].
    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error>;
}
