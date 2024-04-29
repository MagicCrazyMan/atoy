use std::any::Any;

use crate::core::{engine::RenderEngine, scene::Scene, AsAny};

use super::{context::Context, WebGl};

pub struct WebGlRenderEngine {
    context: Context,
}

impl AsAny for WebGlRenderEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl RenderEngine for WebGlRenderEngine {
    type RenderType = WebGl;

    fn render(&self, scene: &Scene<Self::RenderType>) {
        todo!()
    }
}
