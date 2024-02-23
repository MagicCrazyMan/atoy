pub mod gbuffer;
pub mod simple;

use std::borrow::Cow;

use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME, UBO_UNIVERSAL_UNIFORMS_BINDING,
        UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
    },
    renderer::webgl::{
        buffer::BufferDescriptor,
        error::Error,
        framebuffer::{
            AttachmentProvider, Framebuffer, FramebufferAttachment, FramebufferBuilder,
            FramebufferTarget,
        },
        program::{Define, ShaderProvider},
        state::FrameState,
        texture::{TextureColorFormat, TextureUnit},
        uniform::{UniformBlockValue, UniformValue},
    },
    scene::{
        AREA_LIGHTS_COUNT_DEFINE, DIRECTIONAL_LIGHTS_COUNT_DEFINE, MAX_AREA_LIGHTS_STRING,
        MAX_DIRECTIONAL_LIGHTS_STRING, MAX_POINT_LIGHTS_STRING, MAX_SPOT_LIGHTS_STRING,
        POINT_LIGHTS_COUNT_DEFINE, SPOT_LIGHTS_COUNT_DEFINE,
    },
};

pub struct StandardDeferredShading {
    shader: DeferredShader,
    framebuffer: Option<Framebuffer>,
}

impl StandardDeferredShading {
    pub fn new() -> Self {
        Self {
            shader: DeferredShader::new(),
            framebuffer: None,
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new().set_color_attachment0(AttachmentProvider::new_texture(
                    TextureColorFormat::RGBA8,
                )),
            )
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
            self.shader.lighting = true;
            let program = state.program_store_mut().use_program(&self.shader)?;

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
            self.shader.lighting = false;
            state.program_store_mut().use_program(&self.shader)?
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
        ])?;

        self.framebuffer(state).unbind();

        Ok(())
    }
}

const POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME: &'static str =
    "u_PositionsAndSpecularShininessTexture";
const NORMALS_TEXTURE_UNIFORM_NAME: &'static str = "u_NormalsTexture";
const ALBEDO_TEXTURE_UNIFORM_NAME: &'static str = "u_AlbedoTexture";

struct DeferredShader {
    lighting: bool,
}

impl DeferredShader {
    pub fn new() -> Self {
        Self { lighting: false }
    }
}

impl ShaderProvider for DeferredShader {
    fn name(&self) -> Cow<'static, str> {
        if self.lighting {
            Cow::Borrowed("DeferredShader")
        } else {
            Cow::Borrowed("DeferredShader_Lighting")
        }
    }

    fn vertex_source(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("../../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("../../shaders/deferred.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        if self.lighting {
            &[
                Define::WithoutValue(Cow::Borrowed("USE_LIGHTING")),
                Define::WithValue(
                    Cow::Borrowed(DIRECTIONAL_LIGHTS_COUNT_DEFINE),
                    Cow::Borrowed(MAX_DIRECTIONAL_LIGHTS_STRING),
                ),
                Define::WithValue(
                    Cow::Borrowed(POINT_LIGHTS_COUNT_DEFINE),
                    Cow::Borrowed(MAX_POINT_LIGHTS_STRING),
                ),
                Define::WithValue(
                    Cow::Borrowed(SPOT_LIGHTS_COUNT_DEFINE),
                    Cow::Borrowed(MAX_SPOT_LIGHTS_STRING),
                ),
                Define::WithValue(
                    Cow::Borrowed(AREA_LIGHTS_COUNT_DEFINE),
                    Cow::Borrowed(MAX_AREA_LIGHTS_STRING),
                ),
            ]
        } else {
            &[]
        }
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
