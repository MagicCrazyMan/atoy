use crate::render::webgl::error::Error;

use super::{RenderState, RenderStuff};

pub trait PostprocessOp {
    fn name(&self) -> &str;

    fn post_process(&self, state: &RenderState) -> Result<(), Error>;
}

pub enum InternalPostprocess {}

impl PostprocessOp for InternalPostprocess {
    fn name(&self) -> &str {
        todo!()
    }

    fn post_process(&self, state: &RenderState) -> Result<(), Error> {
        todo!()
    }
}
