use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{
    conversion::ToGlEnum,
    draw::CullFace,
    error::Error,
    pipeline::{RenderState, RenderStuff},
};

use super::PreProcessor;

pub struct UpdateCamera;

impl<Stuff> PreProcessor<Stuff> for UpdateCamera
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "UpdateCamera"
    }

    fn pre_process(&mut self, state: &RenderState, stuff: &mut Stuff) -> Result<(), Error> {
        stuff.camera_mut().update_frame(state);
        Ok(())
    }
}

pub struct UpdateViewport;

impl<Stuff> PreProcessor<Stuff> for UpdateViewport
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "UpdateViewport"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
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

impl<Stuff> PreProcessor<Stuff> for EnableDepthTest
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "EnableDepthTest"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        Ok(())
    }
}

pub struct EnableCullFace;

impl<Stuff> PreProcessor<Stuff> for EnableCullFace
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "EnableCullFace"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
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

impl<Stuff> PreProcessor<Stuff> for SetCullFaceMode
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "SetCullFaceMode"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
        state.gl.cull_face(self.0.gl_enum());
        Ok(())
    }
}

pub struct EnableBlend;

impl<Stuff> PreProcessor<Stuff> for EnableBlend
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "EnableBlend"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::BLEND);
        Ok(())
    }
}

pub struct ClearColor(f32, f32, f32, f32);

impl ClearColor {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self(red, green, blue, alpha)
    }
}

impl<Stuff> PreProcessor<Stuff> for ClearColor
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "ClearColor"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
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

impl<Stuff> PreProcessor<Stuff> for ClearDepth
where
    Stuff: RenderStuff,
{
    fn name(&self) -> &str {
        "ClearDepth"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut Stuff) -> Result<(), Error> {
        state.gl.clear_depth(self.0);
        state.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        Ok(())
    }
}
