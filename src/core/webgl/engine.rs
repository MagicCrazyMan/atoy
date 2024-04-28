use crate::core::{engine::RenderEngine, scene::Scene};

use super::{component::EntityComponent, context::Context};

pub struct WebGLRenderEngine {
    context: Context,
}

impl RenderEngine for WebGLRenderEngine {
    type Component = EntityComponent;

    fn render(&self, scene: &Scene<Self::Component>) {
        todo!()
    }
}
