use proc::AsAny;
use web_sys::WebGl2RenderingContext;

use crate::core::{engine::RenderEngine, scene::Scene};

use super::{context::Context, WebGl};

#[derive(Debug, AsAny)]
pub struct WebGlRenderEngine {
    context: Context,
}

impl WebGlRenderEngine {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            context: Context::new(gl),
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }
}

impl RenderEngine for WebGlRenderEngine {
    type RenderType = WebGl;

    fn render(&self, scene: &Scene<Self::RenderType>) {
        todo!()
    }
}
