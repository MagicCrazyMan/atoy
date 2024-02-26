use crate::{camera::Camera, clock::Clock, pipeline::Pipeline, scene::Scene};

pub mod webgl;

pub trait Renderer {
    type State;

    type Clock: Clock;

    type Error;

    fn render(
        &mut self,
        pipeline: &mut (dyn Pipeline<State = Self::State, Clock = Self::Clock, Error = Self::Error> + 'static),
        camera: &mut (dyn Camera + 'static),
        scene: &mut Scene<Self::Clock>,
        timestamp: f64,
    ) -> Result<(), Self::Error>;
}
