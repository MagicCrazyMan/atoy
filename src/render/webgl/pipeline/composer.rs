use std::{any::Any, borrow::Cow};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    render::{
        pp::{Executor, ResourceKey, Resources, State},
        webgl::{
            attribute::AttributeBinding,
            error::Error,
            program::{ProgramSource, ShaderSource},
            uniform::{UniformBinding, UniformBlockBinding, UniformStructuralBinding},
        },
    },
    scene::Scene,
};

const SAMPLER_UNIFORM: UniformBinding = UniformBinding::FromMaterial("u_Sampler");

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    in_clear_color: ResourceKey<(f32, f32, f32, f32)>,
    in_textures: Vec<ResourceKey<WebGlTexture>>,
}

impl StandardComposer {
    pub fn new(
        in_textures: Vec<ResourceKey<WebGlTexture>>,
        in_clear_color: ResourceKey<(f32, f32, f32, f32)>,
    ) -> Self {
        Self {
            in_clear_color,
            in_textures,
        }
    }
}

impl Executor for StandardComposer {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        let program_item = state.program_store_mut().use_program(&ComposerMaterial)?;

        if let Some((r, g, b, a)) = resources.get(&self.in_clear_color) {
            state.gl().clear_color(*r, *g, *b, *a);
        } else {
            state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        }
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        state
            .gl()
            .uniform1i(program_item.uniform_locations().get(&SAMPLER_UNIFORM), 0);
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);

        Ok(true)
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        for texture_key in &self.in_textures {
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

struct ComposerMaterial;

impl ProgramSource for ComposerMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("ComposerMaterial")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!("./shaders/composer.frag"))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}
