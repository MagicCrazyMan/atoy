use crate::render::webgl::error::Error;

use super::{RenderState, RenderStuff};

pub trait PostprocessOp<S> {
    fn name(&self) -> &str;

    fn post_process(&self, state: &RenderState<S>) -> Result<(), Error>;
}

pub enum InternalPostprocess {}

impl<S: RenderStuff> PostprocessOp<S> for InternalPostprocess {
    fn name(&self) -> &str {
        todo!()
    }

    fn post_process(&self, state: &RenderState<S>) -> Result<(), Error> {
        todo!()
    }
}
