pub mod standard;

use crate::render::webgl::error::Error;

use super::{RenderState, RenderStuff};

pub trait PreProcessor {
    fn name(&self) -> &str;

    fn pre_process(
        &mut self,
        state: &RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
