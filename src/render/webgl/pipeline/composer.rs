use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
    WebGlUniformLocation,
};

use crate::render::{
    pp::{Executor, ResourceKey, Resources, State, Stuff},
    webgl::{
        error::Error,
        program::{compile_shaders, create_program, ShaderSource},
    },
};

#[rustfmt::skip]
const BUFFER_DATA: [f32; 16] = [
    // vertices
    1.0,-1.0,  1.0,1.0, -1.0,1.0, -1.0,-1.0,
    // textures coordinates
    1.0, 0.0,  1.0,1.0,  0.0,1.0,  0.0, 0.0
];

struct Compiled {
    gl: WebGl2RenderingContext,
    shaders: [WebGlShader; 2],
    program: WebGlProgram,
    position_location: u32,
    texture_location: u32,
    sampler_location: WebGlUniformLocation,
    buffer: WebGlBuffer,
}

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    compiled: Option<Compiled>,
    textures_keys: Vec<ResourceKey<WebGlTexture>>,
}

impl StandardComposer {
    pub fn new(textures_keys: Vec<ResourceKey<WebGlTexture>>) -> Self {
        Self {
            compiled: None,
            textures_keys,
        }
    }
}

impl Executor for StandardComposer {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if self.compiled.is_none() {
            let vertex_shader = compile_shaders(
                state.gl(),
                &ShaderSource::Vertex(include_str!("./shaders/composer.vert")),
            )?;
            let fragment_shader = compile_shaders(
                state.gl(),
                &ShaderSource::Fragment(include_str!("./shaders/composer.frag")),
            )?;
            let program = create_program(
                state.gl(),
                &[vertex_shader.clone(), fragment_shader.clone()],
            )?;
            let position_location = state.gl().get_attrib_location(&program, "a_Position") as u32;
            let texture_location = state.gl().get_attrib_location(&program, "a_TexCoord") as u32;
            let sampler_location = state
                .gl()
                .get_uniform_location(&program, "u_Sampler")
                .unwrap();

            let buffer = state
                .gl()
                .create_buffer()
                .ok_or(Error::CreateBufferFailure)?;
            state
                .gl()
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
            state.gl().buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                unsafe { &std::mem::transmute_copy::<[f32; 16], [u8; 64]>(&BUFFER_DATA) },
                WebGl2RenderingContext::STATIC_DRAW,
            );

            self.compiled = Some(Compiled {
                gl: state.gl().clone(),
                shaders: [vertex_shader, fragment_shader],
                program,
                position_location,
                texture_location,
                sampler_location,
                buffer,
            });
        }

        let Compiled {
            program,
            position_location,
            texture_location,
            sampler_location,
            buffer,
            ..
        } = self.compiled.as_ref().unwrap();

        state.gl().use_program(Some(program));

        state
            .gl()
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        state.gl().vertex_attrib_pointer_with_i32(
            *position_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        state.gl().enable_vertex_attrib_array(*position_location);
        state.gl().vertex_attrib_pointer_with_i32(
            *texture_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            32,
        );
        state.gl().enable_vertex_attrib_array(*texture_location);
        state.gl().uniform1i(Some(sampler_location), 0);

        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Compiled {
            position_location,
            texture_location,
            ..
        } = self.compiled.as_ref().unwrap();

        state
            .gl()
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        state.gl().disable_vertex_attrib_array(*position_location);
        state.gl().disable_vertex_attrib_array(*texture_location);

        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        for texture_key in &self.textures_keys {
            let Some(texture) = resources.get(texture_key) else {
                continue;
            };

            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );

            state
                .gl()
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
        }

        Ok(())
    }
}

impl Drop for StandardComposer {
    fn drop(&mut self) {
        if let Some(Compiled {
            gl,
            shaders,
            program,
            buffer,
            ..
        }) = &self.compiled
        {
            gl.delete_buffer(Some(buffer));
            gl.delete_program(Some(program));
            gl.delete_shader(Some(&shaders[0]));
            gl.delete_shader(Some(&shaders[1]));
        }
    }
}
