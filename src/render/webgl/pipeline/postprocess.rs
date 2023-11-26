use web_sys::WebGl2RenderingContext;

use crate::render::webgl::error::Error;

use super::{RenderState, RenderStuff};

pub trait PostprocessOp<'s, S> {
    fn name(&'s self) -> &'s str;

    fn post_process(&'s self, state: &'s RenderState<S>) -> Result<(), Error>;
}

pub enum InternalPostprocess {}

impl<'s, S: RenderStuff<'s>> PostprocessOp<'s, S> for InternalPostprocess {
    fn name(&'s self) -> &'s str {
        todo!()
    }

    fn post_process(&'s self, state: &'s RenderState<S>) -> Result<(), Error> {
        todo!()
    }
}
