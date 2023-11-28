use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{
    conversion::ToGlEnum,
    draw::CullFace,
    error::Error,
    pipeline::{preprocess::PreProcessor, RenderPipeline, RenderState, RenderStuff},
};

pub struct UpdateCamera;

impl PreProcessor for UpdateCamera {
    fn name(&self) -> &str {
        "UpdateCamera"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        stuff.camera_mut().update_frame(state);
        Ok(())
    }
}

pub struct UpdateViewport;

impl PreProcessor for UpdateViewport {
    fn name(&self) -> &str {
        "UpdateViewport"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
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

impl PreProcessor for EnableDepthTest {
    fn name(&self) -> &str {
        "EnableDepthTest"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        Ok(())
    }
}

pub struct EnableCullFace;

impl PreProcessor for EnableCullFace {
    fn name(&self) -> &str {
        "EnableCullFace"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
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

impl PreProcessor for SetCullFaceMode {
    fn name(&self) -> &str {
        "SetCullFaceMode"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.cull_face(self.0.gl_enum());
        Ok(())
    }
}

pub struct EnableBlend;

impl PreProcessor for EnableBlend {
    fn name(&self) -> &str {
        "EnableBlend"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
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

impl PreProcessor for ClearColor {
    fn name(&self) -> &str {
        "ClearColor"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
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

impl PreProcessor for ClearDepth {
    fn name(&self) -> &str {
        "ClearDepth"
    }

    fn pre_process(
        &mut self,
        _: &mut dyn RenderPipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state.gl.clear_depth(self.0);
        state.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        Ok(())
    }
}
