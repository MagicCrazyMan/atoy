use crate::render::webgl::error::Error;

use super::{RenderPipeline, RenderState, RenderStuff};

pub trait PostProcessor {
    fn name(&self) -> &str;

    fn post_process(
        &mut self,
        pipeline: &mut dyn RenderPipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
