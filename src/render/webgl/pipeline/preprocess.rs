use crate::render::webgl::error::Error;

use super::{RenderPipeline, RenderState, RenderStuff};

pub trait PreProcessor {
    fn name(&self) -> &str;

    fn pre_process(
        &mut self,
        pipeline: &mut dyn RenderPipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
