pub mod gbuffer;
pub mod simple;

use std::borrow::Cow;

use web_sys::WebGlTexture;

use crate::{
    light::{
        area_light::{AREA_LIGHTS_COUNT_DEFINE, MAX_AREA_LIGHTS},
        directional_light::{DIRECTIONAL_LIGHTS_COUNT_DEFINE, MAX_DIRECTIONAL_LIGHTS},
        point_light::{MAX_POINT_LIGHTS, POINT_LIGHTS_COUNT_DEFINE},
        spot_light::{MAX_SPOT_LIGHTS, SPOT_LIGHTS_COUNT_DEFINE},
    },
    render::webgl::{
        buffer::BufferDescriptor,
        error::Error,
        framebuffer::{
            AttachmentProvider, Framebuffer, FramebufferAttachment, FramebufferBuilder,
            FramebufferTarget,
        },
        pipeline::{
            UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME, UBO_UNIVERSAL_UNIFORMS_BINDING,
            UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
        },
        program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
        shader::{Define, ShaderBuilder},
        state::FrameState,
        texture::{TextureInternalFormatUncompressed, TextureUnit},
        uniform::{UniformBlockValue, UniformValue},
    },
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
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureInternalFormatUncompressed::RGBA8),
            ))
        })
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT0))
    }

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        positions_and_specular_shininess_texture: &WebGlTexture,
        normals_texture: &WebGlTexture,
        albedo_texture: &WebGlTexture,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        self.framebuffer(state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer(state).clear_buffers()?;

        let program = if let Some(lights_ubo) = lights_ubo {
            let program = state.program_store_mut().use_program_with_defines(
                &DeferredShadingProgram,
                &[],
                &[Define::WithoutValue(Cow::Borrowed(LIGHTING_DEFINE))],
            )?;

            // binds atoy_Lights
            state.bind_uniform_block_value_by_block_name(
                program,
                UBO_LIGHTS_BLOCK_NAME,
                &UniformBlockValue::BufferBase {
                    descriptor: lights_ubo.clone(),
                    binding: UBO_LIGHTS_BINDING,
                },
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME,
                &UniformValue::Integer1(0),
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                NORMALS_TEXTURE_UNIFORM_NAME,
                &UniformValue::Integer1(1),
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
            &UniformBlockValue::BufferBase {
                descriptor: universal_ubo.clone(),
                binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
            },
        )?;

        state.bind_uniform_value_by_variable_name(
            program,
            ALBEDO_TEXTURE_UNIFORM_NAME,
            &UniformValue::Integer1(2),
        )?;

        state.do_computation([
            (
                positions_and_specular_shininess_texture,
                TextureUnit::TEXTURE0,
            ),
            (normals_texture, TextureUnit::TEXTURE1),
            (albedo_texture, TextureUnit::TEXTURE2),
        ]);

        self.framebuffer(state).unbind();

        Ok(())
    }
}

const POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME: &'static str =
    "u_PositionsAndSpecularShininessTexture";
const NORMALS_TEXTURE_UNIFORM_NAME: &'static str = "u_NormalsTexture";
const ALBEDO_TEXTURE_UNIFORM_NAME: &'static str = "u_AlbedoTexture";

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
            vec![
                Define::WithValue(
                    Cow::Borrowed(DIRECTIONAL_LIGHTS_COUNT_DEFINE),
                    Cow::Owned(MAX_DIRECTIONAL_LIGHTS.to_string()),
                ),
                Define::WithValue(
                    Cow::Borrowed(POINT_LIGHTS_COUNT_DEFINE),
                    Cow::Owned(MAX_POINT_LIGHTS.to_string()),
                ),
                Define::WithValue(
                    Cow::Borrowed(SPOT_LIGHTS_COUNT_DEFINE),
                    Cow::Owned(MAX_SPOT_LIGHTS.to_string()),
                ),
                Define::WithValue(
                    Cow::Borrowed(AREA_LIGHTS_COUNT_DEFINE),
                    Cow::Owned(MAX_AREA_LIGHTS.to_string()),
                ),
            ],
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
