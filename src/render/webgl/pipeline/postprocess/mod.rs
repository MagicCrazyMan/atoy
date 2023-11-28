pub mod standard;

use web_sys::WebGl2RenderingContext;

use crate::render::webgl::error::Error;

use super::{RenderState, RenderStuff};

pub trait PostProcessor {
    fn name(&self) -> &str;

    fn post_process(&self, state: &RenderState, stuff: &mut dyn RenderStuff) -> Result<(), Error>;
}

pub enum StandardPostProcess {
    Reset,
}

impl PostProcessor for StandardPostProcess {
    fn name(&self) -> &str {
        match self {
            StandardPostProcess::Reset => "Reset",
        }
    }

    fn post_process(&self, state: &RenderState, _: &mut dyn RenderStuff) -> Result<(), Error> {
        match self {
            StandardPostProcess::Reset => {
                state.gl.use_program(None);
                state
                    .gl
                    .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
                state
                    .gl
                    .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);
                state
                    .gl
                    .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
                state
                    .gl
                    .bind_renderbuffer(WebGl2RenderingContext::READ_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::COPY_READ_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::COPY_WRITE_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::PIXEL_PACK_BUFFER, None);
                state
                    .gl
                    .bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, None);
                for index in 0..32 {
                    state
                        .gl
                        .active_texture(WebGl2RenderingContext::TEXTURE0 + index);
                    state
                        .gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
                    state
                        .gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, None);
                }
                state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
                state.gl.bind_vertex_array(None);
                state.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
                state.gl.disable(WebGl2RenderingContext::CULL_FACE);
                state.gl.disable(WebGl2RenderingContext::BLEND);
                state.gl.disable(WebGl2RenderingContext::DITHER);
                state
                    .gl
                    .disable(WebGl2RenderingContext::POLYGON_OFFSET_FILL);
                state
                    .gl
                    .disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
                state.gl.disable(WebGl2RenderingContext::SAMPLE_COVERAGE);
                state.gl.disable(WebGl2RenderingContext::SCISSOR_TEST);
                state.gl.disable(WebGl2RenderingContext::STENCIL_TEST);
                state.gl.disable(WebGl2RenderingContext::RASTERIZER_DISCARD);
            }
        };

        Ok(())
    }
}
