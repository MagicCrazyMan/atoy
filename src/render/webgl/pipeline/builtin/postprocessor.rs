use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{
    error::Error,
    pipeline::{postprocess::PostProcessor, RenderPipeline, RenderState, RenderStuff},
};

pub struct Reset;

impl PostProcessor for Reset {
    fn name(&self) -> &str {
        "Reset"
    }

    fn post_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
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
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
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

        Ok(())
    }
}
