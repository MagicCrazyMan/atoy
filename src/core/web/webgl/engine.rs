use proc::AsAny;
use web_sys::WebGl2RenderingContext;

use crate::core::{app::AppConfig, engine::{RenderContext, RenderEngine}};

use super::{buffer::BufferRegistry, framebuffer::FramebufferRegistry, texture::TextureRegistry};

#[derive(Debug, AsAny)]
pub struct WebGlRenderEngine {
    gl: WebGl2RenderingContext,
    buffer_registry: BufferRegistry,
    texture_registry: TextureRegistry,
    framebuffer_registry: FramebufferRegistry,
}

impl WebGlRenderEngine {
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn buffer_registry(&self) -> &BufferRegistry {
        &self.buffer_registry
    }

    pub fn texture_registry(&self) -> &TextureRegistry {
        &self.texture_registry
    }

    pub fn framebuffer_registry(&self) -> &FramebufferRegistry {
        &self.framebuffer_registry
    }
}

impl RenderEngine for WebGlRenderEngine {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized,
    {
        let gl: WebGl2RenderingContext = todo!();
        let buffer_registry = BufferRegistry::new(gl.clone());
        let texture_registry = TextureRegistry::new(gl.clone(), buffer_registry.clone());
        let framebuffer_registry = FramebufferRegistry::new(
            gl.clone(),
            buffer_registry.clone(),
            texture_registry.clone(),
        );
        Self {
            buffer_registry,
            texture_registry,
            framebuffer_registry,
            // uniform_buffer_objects: HashMap::new(),
            gl,
        }
    }

    fn render(&mut self, context: RenderContext) {
        todo!()
    }
}
