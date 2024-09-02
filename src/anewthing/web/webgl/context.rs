use web_sys::WebGl2RenderingContext;

use super::{buffer::WebGlBufferManager, program::WebGlProgramManager};

pub struct Context {
    gl: WebGl2RenderingContext,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,
}
