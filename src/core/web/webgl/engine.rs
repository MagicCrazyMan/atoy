use proc::AsAny;
use web_sys::WebGl2RenderingContext;

use crate::core::{app::Context, engine::RenderEngine};

use super::{buffer::BufferRegistry, framebuffer::FramebufferRegistry, texture::TextureRegistry};

#[derive(Debug, AsAny)]
pub struct WebGlRenderEngine {
    gl: WebGl2RenderingContext,
    buffer_registry: BufferRegistry,
    texture_registry: TextureRegistry,
    framebuffer_registry: FramebufferRegistry,
}

impl WebGlRenderEngine {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
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

impl<CLK> RenderEngine<CLK> for WebGlRenderEngine {
    fn render(&mut self, scene: &Context<CLK, Self>) {
        todo!()
    }
}
