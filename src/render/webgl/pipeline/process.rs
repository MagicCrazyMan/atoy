use crate::render::webgl::error::Error;

use super::{RenderPipeline, RenderState, RenderStuff};

pub trait Processor<Pipeline>
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str;

    fn process(
        &mut self,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}