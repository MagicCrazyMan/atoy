use crate::{
    render::{
        webgl::{error::Error, state::FrameState},
        Executor, Resources,
    },
    scene::Scene,
};

pub struct StandardPreparation;

impl StandardPreparation {
    pub fn new() -> Self {
        Self
    }
}

impl Executor for StandardPreparation {
    type State = FrameState;

    type Error = Error;

    fn execute(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        state.gl().viewport(
            0,
            0,
            state.canvas().width() as i32,
            state.canvas().height() as i32,
        );
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
