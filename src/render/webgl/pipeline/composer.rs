use std::{any::Any, borrow::Cow};

use gl_matrix4rust::vec4::{AsVec4, Vec4};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    render::{
        webgl::{
            error::Error,
            program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
            state::FrameState,
        },
        Executor, ResourceKey, Resources,
    },
    scene::Scene,
};

const TEXTURE: &'static str = "u_Texture";

pub static DEFAULT_CLEAR_COLOR: Vec4 = Vec4::from_values(0.0, 0.0, 0.0, 0.0);

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    clear_color_key: Option<ResourceKey<Vec4>>,
    textures_key: Vec<ResourceKey<WebGlTexture>>,
}

impl StandardComposer {
    pub fn new(
        textures_key: Vec<ResourceKey<WebGlTexture>>,
        clear_color_key: Option<ResourceKey<Vec4>>,
    ) -> Self {
        Self {
            clear_color_key,
            textures_key,
        }
    }

    fn clear_color<'a, 'b>(&'a self, resources: &'b Resources) -> &'b Vec4 {
        if let Some(color) = self
            .clear_color_key
            .as_ref()
            .and_then(|key| resources.get(key))
        {
            color
        } else {
            &DEFAULT_CLEAR_COLOR
        }
    }
}

impl Executor for StandardComposer {
    type State = FrameState;

    type Error = Error;

    fn before(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        let program = state.program_store_mut().use_program(&ComposerProgram)?;

        let clear_color = self.clear_color(resources);
        state.gl().clear_color(
            clear_color.x() as f32,
            clear_color.y() as f32,
            clear_color.z() as f32,
            clear_color.w() as f32,
        );
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        state.gl().uniform1i(
            program.get_or_retrieve_uniform_location(TEXTURE).as_ref(),
            0,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        state.gl().disable(WebGl2RenderingContext::BLEND);
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        for texture_key in &self.textures_key {
            let Some(texture) = resources.get(texture_key) else {
                continue;
            };

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

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
