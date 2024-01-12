use crate::{camera::Camera, scene::Scene};

use self::pp::Pipeline;

pub mod pp;
pub mod webgl;

pub trait Render {
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
