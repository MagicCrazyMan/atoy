use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{
    conversion::{GLint, GLuint, ToGlEnum},
    draw::CullFace,
    error::Error,
    pipeline::{preprocess::PreProcessor, RenderPipeline, RenderState, RenderStuff},
    stencil::{StencilFunction, StencilOp},
};

pub struct UpdateCamera;

impl<Pipeline> PreProcessor<Pipeline> for UpdateCamera
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "UpdateCamera"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for UpdateViewport
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "UpdateViewport"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for EnableDepthTest
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableDepthTest"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for SetDepthMask
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetDepthMask"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for EnableCullFace
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableCullFace"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for SetCullFaceMode
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetCullFaceMode"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for EnableBlend
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableBlend"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for EnableStencil
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "EnableStencil"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for SetStencilMask
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilMask"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for SetStencilFunc
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilFunc"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for SetStencilOp
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "SetStencilOp"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for ClearColor
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "ClearColor"
    }

    fn pre_process(
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

impl<Pipeline> PreProcessor<Pipeline> for ClearDepth
where
    Pipeline: RenderPipeline,
{
    fn name(&self) -> &str {
        "ClearDepth"
    }

    fn pre_process(
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
