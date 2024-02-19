use crate::{camera::Camera, pipeline::Pipeline, scene::Scene};

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
