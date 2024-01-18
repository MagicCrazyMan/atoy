use std::borrow::Cow;

use gl_matrix4rust::vec4::Vec4;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{
    error::Error,
    program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
    state::FrameState,
};

const TEXTURE: &'static str = "u_Texture";

pub const DEFAULT_CLEAR_COLOR: Vec4 = Vec4::<f64>::new_zero();

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    clear_color: Vec4,
}

impl StandardComposer {
    pub fn new() -> Self {
        Self {
            clear_color: DEFAULT_CLEAR_COLOR,
        }
    }

    pub fn clear_color(&self) -> &Vec4 {
        &self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
    }

    pub fn compose<'a, I>(
        &self,
        state: &mut FrameState,
        textures: I,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a WebGlTexture>,
    {
        state.gl().clear_color(
            *self.clear_color.r() as f32,
            *self.clear_color.g() as f32,
            *self.clear_color.b() as f32,
            *self.clear_color.a() as f32,
        );
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        let program = state.program_store_mut().use_program(&ComposerProgram)?;
        state.gl().uniform1i(
            program.get_or_retrieve_uniform_location(TEXTURE).as_ref(),
            0,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        for texture in textures {
            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                0,
            );

            state
                .gl()
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
        }

        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        state.gl().disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }
}

struct ComposerProgram;

impl ProgramSource for ComposerProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("ComposerProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/composer.frag")))
    }
}
