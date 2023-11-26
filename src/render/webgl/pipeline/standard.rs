use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

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

impl<'s> RenderStuff<'s> for StandardRenderStuff<'s> {
    fn canvas(&'s self) -> &'s HtmlCanvasElement {
        self.scene.canvas()
    }

    fn ctx_options(&'s self) -> Option<&'s JsValue> {
        None
    }

    fn entities(&'s mut self) -> &'s mut [Entity] {
        self.scene.root_entity_mut().children_mut()
    }

    fn camera(&'s mut self) -> &'s mut dyn Camera {
        self.scene.active_camera_mut()
    }
}

pub struct StandardPipeline<'p> {
    scene: &'p mut Scene,
}

impl<'s, 'p: 's> RenderPipeline<'s, 'p, StandardRenderStuff<'s>> for StandardPipeline<'p> {
    fn dependencies(&'p mut self) -> Result<(), Error> {
        todo!()
    }

    fn prepare(&'p mut self) -> Result<StandardRenderStuff<'s>, Error> {
        Ok(StandardRenderStuff { scene: self.scene })
    }

    fn pre_process(
        &'p mut self,
        _: &'p RenderState<StandardRenderStuff<'s>>,
    ) -> Result<&'p [&'p dyn PreprocessOp<StandardRenderStuff<'s>>], Error> {
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

    fn post_precess(
        &'p mut self,
        _: &'p RenderState<StandardRenderStuff<'s>>,
    ) -> Result<&'p [&'p dyn PostprocessOp<StandardRenderStuff<'s>>], Error> {
        Ok(&[])
    }
}
