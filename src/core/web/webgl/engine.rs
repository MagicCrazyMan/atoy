use proc::AsAny;
use web_sys::WebGl2RenderingContext;

use crate::core::{engine::RenderEngine, scene::Scene};

use super::context::Context;

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
    fn render(&mut self, scene: &mut Scene) {
        todo!()
    }
}
