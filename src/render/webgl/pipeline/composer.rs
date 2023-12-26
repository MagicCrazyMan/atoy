use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlTexture, WebGlUniformLocation};

use crate::render::{
    pp::{Executor, ResourceKey, Resources, State, Stuff},
    webgl::{
        error::Error,
        program::{compile_shaders, create_program, ShaderSource},
    },
};

struct Compiled {
    program: WebGlProgram,
    sampler_location: WebGlUniformLocation,
}

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    compiled: Option<Compiled>,
    in_clear_color: ResourceKey<(f32, f32, f32, f32)>,
    in_textures: Vec<ResourceKey<WebGlTexture>>,
}

impl StandardComposer {
    pub fn new(
        in_textures: Vec<ResourceKey<WebGlTexture>>,
        in_clear_color: ResourceKey<(f32, f32, f32, f32)>,
    ) -> Self {
        Self {
            compiled: None,
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
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if self.compiled.is_none() {
            let vertex_shader = compile_shaders(
                state.gl(),
                &ShaderSource::VertexRaw(include_str!("./shaders/composer.vert")),
            )?;
            let fragment_shader = compile_shaders(
                state.gl(),
                &ShaderSource::FragmentRaw(include_str!("./shaders/composer.frag")),
            )?;
            let program = create_program(
                state.gl(),
                &[vertex_shader.clone(), fragment_shader.clone()],
            )?;
            let sampler_location = state
                .gl()
                .get_uniform_location(&program, "u_Sampler")
                .unwrap();

            self.compiled = Some(Compiled {
                program,
                sampler_location,
            });
        }

        let Compiled {
            program,
            sampler_location,
            ..
        } = self.compiled.as_ref().unwrap();

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

        state.gl().use_program(Some(program));
        state.gl().uniform1i(Some(sampler_location), 0);
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);

        Ok(true)
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
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
}
