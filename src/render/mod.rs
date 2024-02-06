use crate::{camera::Camera, scene::Scene};

pub mod webgl;

pub trait Renderer {
    type State;

    type Error;

    fn render(
        &mut self,
        pipeline: &mut (dyn Pipeline<State = Self::State, Error = Self::Error> + 'static),
        camera: &mut (dyn Camera + 'static),
        scene: &mut Scene,
        timestamp: f64,
    ) -> Result<(), Self::Error>;
}

/// A rendering pipeline.
pub trait Pipeline {
    /// Runtime state.
    type State;

    /// Error that could be thrown during execution.
    type Error;

    /// Executes this rendering pipeline with specified `State` and a [`Scene`].
    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error>;
}
