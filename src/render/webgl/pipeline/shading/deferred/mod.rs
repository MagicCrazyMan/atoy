pub mod gbuffer;

use std::borrow::Cow;

use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        ClearPolicy, Framebuffer, FramebufferAttachment, FramebufferSizePolicy, FramebufferTarget,
        TextureProvider,
    },
    pipeline::{
        UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME, UBO_UNIVERSAL_UNIFORMS_BINDING,
        UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
    },
    program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
    shader::ShaderBuilder,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat, TextureUnit},
    uniform::{UniformBlockValue, UniformValue},
};

use super::LIGHTING_DEFINE;

pub struct StandardDeferredShading {
    framebuffer: Option<Framebuffer>,
}

impl StandardDeferredShading {
    pub fn new() -> Self {
        Self { framebuffer: None }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA8,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                )],
                [],
                [],
                None,
            )
        })
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(0))
    }

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        positions_texture: &WebGlTexture,
        normals_texture: &WebGlTexture,
        albedo_and_specular_shininess_texture: &WebGlTexture,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        self.framebuffer(state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        let program = if let Some(lights_ubo) = lights_ubo {
            let program = state.program_store_mut().use_program_with_defines(
                &DeferredShadingProgram,
                &[],
                &[Cow::Borrowed(LIGHTING_DEFINE)],
            )?;

            // binds atoy_Lights
            state.bind_uniform_block_value_by_block_name(
                program,
                UBO_LIGHTS_BLOCK_NAME,
                UniformBlockValue::BufferBase {
                    descriptor: lights_ubo.clone(),
                    binding: UBO_LIGHTS_BINDING,
                },
            )?;

            program
        } else {
            state
                .program_store_mut()
                .use_program(&DeferredShadingProgram)?
        };

        // binds atoy_UniversalUniforms
        state.bind_uniform_block_value_by_block_name(
            program,
            UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
            UniformBlockValue::BufferBase {
                descriptor: universal_ubo.clone(),
                binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
            },
        )?;

        state.bind_uniform_value_by_variable_name(
            program,
            POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME,
            UniformValue::Integer1(0),
        )?;
        state.bind_uniform_value_by_variable_name(
            program,
            NORMALS_TEXTURE_UNIFORM_NAME,
            UniformValue::Integer1(1),
        )?;
        state.bind_uniform_value_by_variable_name(
            program,
            ALBEDO_AND_TRANSPARENCY_TEXTURE_UNIFORM_NAME,
            UniformValue::Integer1(2),
        )?;

        state.do_computation([
            (positions_texture, TextureUnit::TEXTURE0),
            (normals_texture, TextureUnit::TEXTURE1),
            (albedo_and_specular_shininess_texture, TextureUnit::TEXTURE2),
        ]);

        self.framebuffer(state).unbind();

        Ok(())
    }
}

const POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME: &'static str =
    "u_PositionsAndSpecularShininessTexture";
const NORMALS_TEXTURE_UNIFORM_NAME: &'static str = "u_NormalsTexture";
const ALBEDO_AND_TRANSPARENCY_TEXTURE_UNIFORM_NAME: &'static str = "u_AlbedoAndTransparencyTexture";

struct DeferredShadingProgram;

impl ProgramSource for DeferredShadingProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("DeferredShadingProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!(
            "../../shaders/computation.vert"
        )))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Builder(ShaderBuilder::new(
            true,
            vec![],
            vec![Cow::Borrowed(include_str!(
                "../../../../../material/shaders/constants.glsl"
            ))],
            vec![
                Cow::Borrowed(include_str!(
                    "../../../../../material/shaders/lighting.glsl"
                )),
                Cow::Borrowed(include_str!("../../shaders/deferred_frag.glsl")),
            ],
        ))
    }
}
