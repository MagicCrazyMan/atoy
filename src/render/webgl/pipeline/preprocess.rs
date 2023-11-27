use web_sys::WebGl2RenderingContext;

use crate::render::webgl::{conversion::ToGlEnum, draw::CullFace, error::Error};

use super::{RenderState, RenderStuff};

pub trait PreprocessOp {
    fn name(&self) -> &str;

    fn pre_process(&self, state: &RenderState) -> Result<(), Error>;
}

pub enum InternalPreprocessOp {
    UpdateViewport,
    EnableDepthTest,
    EnableCullFace,
    EnableBlend,
    ClearColor(f32, f32, f32, f32),
    ClearDepth(f32),
    SetCullFaceMode(CullFace),
}

impl PreprocessOp for InternalPreprocessOp {
    fn name(&self) -> &str {
        match self {
            InternalPreprocessOp::UpdateViewport => "UpdateViewport",
            InternalPreprocessOp::EnableDepthTest => "EnableDepthTest",
            InternalPreprocessOp::EnableCullFace => "EnableCullFace",
            InternalPreprocessOp::EnableBlend => "EnableBlend",
            InternalPreprocessOp::ClearColor(_, _, _, _) => "ClearDepth",
            InternalPreprocessOp::ClearDepth(_) => "ClearColor",
            InternalPreprocessOp::SetCullFaceMode(_) => "SetCullFaceMode",
        }
    }

    fn pre_process(&self, state: &RenderState) -> Result<(), Error> {
        match self {
            InternalPreprocessOp::UpdateViewport => state.gl().viewport(
                0,
                0,
                state.canvas().width() as i32,
                state.canvas().height() as i32,
            ),
            InternalPreprocessOp::EnableDepthTest => {
                state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
            }
            InternalPreprocessOp::EnableCullFace => {
                state.gl().enable(WebGl2RenderingContext::CULL_FACE);
            }
            InternalPreprocessOp::EnableBlend => state.gl().enable(WebGl2RenderingContext::BLEND),
            InternalPreprocessOp::ClearColor(red, green, blue, alpha) => {
                state.gl().clear_color(*red, *green, *blue, *alpha);
                state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
            }
            InternalPreprocessOp::ClearDepth(depth) => {
                state.gl().clear_depth(*depth);
                state.gl().clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
            }
            InternalPreprocessOp::SetCullFaceMode(cull_face) => {
                state.gl().cull_face(cull_face.gl_enum());
            }
        };

        Ok(())
    }
}
