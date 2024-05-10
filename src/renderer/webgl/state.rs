use std::ptr::NonNull;

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlTexture};

use crate::camera::Camera;

use super::{
    buffer::BufferStore, capabilities::Capabilities, conversion::ToGlEnum, error::Error, params::GetWebGlParameters, program::ProgramStore, texture::{TextureStore, TextureUnit}
};

pub struct FrameState {
    timestamp: f64,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    camera: NonNull<(dyn Camera + 'static)>,

    program_store: NonNull<ProgramStore>,
    buffer_store: NonNull<BufferStore>,
    texture_store: NonNull<TextureStore>,
    capabilities: NonNull<Capabilities>,
}

impl FrameState {
    /// Constructs a new rendering state.
    pub fn new(
        timestamp: f64,
        camera: &mut (dyn Camera + 'static),
        gl: WebGl2RenderingContext,
        canvas: HtmlCanvasElement,
        program_store: &mut ProgramStore,
        buffer_store: &mut BufferStore,
        texture_store: &mut TextureStore,
        capabilities: &mut Capabilities,
    ) -> Self {
        unsafe {
            Self {
                timestamp,
                gl,
                canvas,
                camera: NonNull::new_unchecked(camera),
                program_store: NonNull::new_unchecked(program_store),
                buffer_store: NonNull::new_unchecked(buffer_store),
                texture_store: NonNull::new_unchecked(texture_store),
                capabilities: NonNull::new_unchecked(capabilities),
            }
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Returns the [`Camera`].
    pub fn camera(&self) -> &dyn Camera {
        unsafe { self.camera.as_ref() }
    }

    /// Returns the [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store(&self) -> &ProgramStore {
        unsafe { self.program_store.as_ref() }
    }

    /// Returns the mutable [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        unsafe { self.program_store.as_mut() }
    }

    /// Returns the [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store(&self) -> &BufferStore {
        unsafe { self.buffer_store.as_ref() }
    }

    /// Returns the [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store(&self) -> &TextureStore {
        unsafe { self.texture_store.as_ref() }
    }

    /// Returns the [`Capabilities`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn capabilities(&self) -> &Capabilities {
        unsafe { self.capabilities.as_ref() }
    }

    /// Applies computation using current binding framebuffer and program.
    pub fn do_computation<'a, I>(&self, textures: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (&'a WebGlTexture, TextureUnit)>,
    {
        let sampler = self.capabilities().computation_sampler()?;
        let mut states = Vec::new();
        for (texture, unit) in textures {
            self.gl.active_texture(unit.gl_enum());
            let binding = self.gl.texture_binding_2d();
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            self.gl
                .bind_sampler(unit.unit_index() as u32, Some(&sampler));
            states.push((unit, binding));
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);

        for (unit, binding) in states {
            self.gl.active_texture(unit.gl_enum());
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());
            self.gl.bind_sampler(unit.unit_index() as u32, None);
        }

        Ok(())
    }
}
