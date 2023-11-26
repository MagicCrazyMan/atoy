use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use crate::{
    camera::Camera,
    entity::Entity,
    render::webgl::{draw::CullFace, error::Error},
    scene::Scene,
};

use super::{
    postprocess::PostprocessOp,
    preprocess::{InternalPreprocessOp, PreprocessOp},
    RenderPipeline, RenderState, RenderStuff,
};

struct StandardRenderStuff<'s> {
    scene: &'s mut Scene,
}

impl<'s> RenderStuff for StandardRenderStuff<'s> {
    fn canvas(&self) -> &HtmlCanvasElement {
        self.scene.canvas()
    }

    fn ctx_options(&self) -> Option<&JsValue> {
        None
    }

    fn entities(&mut self) -> &mut [Entity] {
        self.scene.root_entity_mut().children_mut()
    }

    fn camera(&mut self) -> &mut dyn Camera {
        self.scene.active_camera_mut()
    }
}

pub struct StandardPipeline<'p> {
    scene: &'p mut Scene,
}

impl<'p> RenderPipeline<StandardRenderStuff<'p>> for StandardPipeline<'p> {
    fn dependencies<'a>(&'a mut self) -> Result<(), Error> {
        todo!()
    }

    fn prepare<'a>(&'a mut self) -> Result<StandardRenderStuff<'p>, Error> {
        Ok(StandardRenderStuff { scene: self.scene })
    }

    fn pre_process<'a>(
        &'a mut self,
        _: &mut RenderState<StandardRenderStuff<'p>>,
    ) -> Result<&[&dyn PreprocessOp<StandardRenderStuff<'p>>], Error> {
        Ok(&[
            &InternalPreprocessOp::UpdateViewport,
            &InternalPreprocessOp::EnableDepthTest,
            &InternalPreprocessOp::EnableCullFace,
            &InternalPreprocessOp::EnableBlend,
            &InternalPreprocessOp::ClearColor(0.0, 0.0, 0.0, 0.0),
            &InternalPreprocessOp::ClearDepth(0.0),
            &InternalPreprocessOp::SetCullFaceMode(CullFace::Back),
        ])
    }

    fn post_precess<'a>(
        &'a mut self,
        _: &mut RenderState<StandardRenderStuff<'p>>,
    ) -> Result<&[&dyn PostprocessOp<StandardRenderStuff<'p>>], Error> {
        Ok(&[])
    }
}
