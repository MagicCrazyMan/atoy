use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{
    conversion::{GLint, GLuint, ToGlEnum},
    draw::CullFace,
    error::Error,
    pipeline::{process::Processor, RenderPipeline, RenderState, RenderStuff},
    stencil::{StencilFunction, StencilOp},
};

pub struct UpdateCamera;

impl<Pipeline> Processor<Pipeline> for UpdateCamera
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "UpdateCamera"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        stuff.camera_mut().update_frame(state);
        Ok(())
    }
}

pub struct UpdateViewport;

impl<Pipeline> Processor<Pipeline> for UpdateViewport
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "UpdateViewport"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.viewport(
            0,
            0,
            state.canvas.width() as i32,
            state.canvas.height() as i32,
        );
        Ok(())
    }
}

pub struct EnableDepthTest;

impl<Pipeline> Processor<Pipeline> for EnableDepthTest
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableDepthTest"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        Ok(())
    }
}

pub struct SetDepthMask(bool);

impl SetDepthMask {
    pub fn new(enabled: bool) -> Self {
        Self(enabled)
    }
}

impl<Pipeline> Processor<Pipeline> for SetDepthMask
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetDepthMask"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.depth_mask(self.0);
        Ok(())
    }
}

pub struct EnableCullFace;

impl<Pipeline> Processor<Pipeline> for EnableCullFace
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableCullFace"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::CULL_FACE);
        Ok(())
    }
}

pub struct SetCullFaceMode(CullFace);

impl SetCullFaceMode {
    pub fn new(cull_face: CullFace) -> Self {
        Self(cull_face)
    }
}

impl<Pipeline> Processor<Pipeline> for SetCullFaceMode
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetCullFaceMode"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.cull_face(self.0.gl_enum());
        Ok(())
    }
}

pub struct EnableBlend;

impl<Pipeline> Processor<Pipeline> for EnableBlend
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableBlend"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::BLEND);
        Ok(())
    }
}

pub struct EnableStencil;

impl<Pipeline> Processor<Pipeline> for EnableStencil
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableStencil"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::STENCIL_TEST);
        Ok(())
    }
}

pub struct SetStencilMask(GLuint);

impl SetStencilMask {
    pub fn new(mask: GLuint) -> Self {
        Self(mask)
    }
}

impl<Pipeline> Processor<Pipeline> for SetStencilMask
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilMask"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.stencil_mask(self.0);
        Ok(())
    }
}

pub struct SetStencilFunc(StencilFunction, GLint, GLuint);

impl SetStencilFunc {
    pub fn new(stencil_func: StencilFunction, ref_val: GLint, mask: GLuint) -> SetStencilFunc {
        Self(stencil_func, ref_val, mask)
    }
}

impl<Pipeline> Processor<Pipeline> for SetStencilFunc
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilFunc"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.stencil_func(self.0.gl_enum(), self.1, self.2);
        Ok(())
    }
}

pub struct SetStencilOp(StencilOp, StencilOp, StencilOp);

impl SetStencilOp {
    pub fn new(fail: StencilOp, z_fail: StencilOp, z_pass: StencilOp) -> SetStencilOp {
        Self(fail, z_fail, z_pass)
    }
}

impl<Pipeline> Processor<Pipeline> for SetStencilOp
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilOp"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state
            .gl
            .stencil_op(self.0.gl_enum(), self.1.gl_enum(), self.2.gl_enum());
        Ok(())
    }
}

pub struct ClearColor(f32, f32, f32, f32);

impl ClearColor {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(red, green, blue, alpha)
    }
}

impl<Pipeline> Processor<Pipeline> for ClearColor
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "ClearColor"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.clear_color(self.0, self.1, self.2, self.3);
        state.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        Ok(())
    }
}

pub struct ClearDepth(f32);

impl ClearDepth {
    pub fn new(depth: f32) -> Self {
        Self(depth)
    }
}

impl<Pipeline> Processor<Pipeline> for ClearDepth
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "ClearDepth"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.clear_depth(self.0);
        state.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        Ok(())
    }
}

pub struct Reset;

impl<Pipeline> Processor<Pipeline> for Reset
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "Reset"
    }

    fn process(
        &mut self,
        _: &mut Pipeline,
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
