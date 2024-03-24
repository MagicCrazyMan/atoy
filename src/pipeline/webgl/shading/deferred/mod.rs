pub mod gbuffer;
pub mod simple;

use std::borrow::Cow;

use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        UBO_LIGHTS_BLOCK_BINDING, UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT,
        UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING, UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
    },
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentSource, Framebuffer, FramebufferAttachmentTarget, FramebufferBuilder,
            FramebufferTarget,
        },
        program::{Define, ProgramSource},
        state::FrameState,
        texture::{TextureUncompressedInternalFormat, TextureUnit},
        uniform::{UniformBinding, UniformValue},
    },
    scene::{
        AREA_LIGHTS_COUNT_DEFINE, DIRECTIONAL_LIGHTS_COUNT_DEFINE, MAX_AREA_LIGHTS_STRING,
        MAX_DIRECTIONAL_LIGHTS_STRING, MAX_POINT_LIGHTS_STRING, MAX_SPOT_LIGHTS_STRING,
        POINT_LIGHTS_COUNT_DEFINE, SPOT_LIGHTS_COUNT_DEFINE,
    },
};

pub struct StandardDeferredShading {
    shader: DeferredShader,
    framebuffer: Framebuffer,
}

impl StandardDeferredShading {
    pub fn new() -> Self {
        Self {
            shader: DeferredShader::new(),
            framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA8,
                ))
                .build(),
        }
    }

    pub fn draw_texture(&self) -> Result<Option<&WebGlTexture>, Error> {
        self.framebuffer
            .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)
    }

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        positions_and_specular_shininess_texture: &WebGlTexture,
        normals_texture: &WebGlTexture,
        albedo_texture: &WebGlTexture,
        lighting: bool,
    ) -> Result<(), Error> {
        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer.clear_buffers()?;

        let program = if lighting {
            self.shader.lighting = true;
            let program = state
                .program_store_mut()
                .get_or_compile_program(&self.shader)?;
            program.use_program()?;
            // binds atoy_Lights
            program.mount_uniform_block_by_binding(
                &UBO_LIGHTS_BLOCK_BINDING,
                UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT,
            )?;
            program.bind_uniform_value_by_binding(
                &POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_BINDING,
                &UniformValue::Integer1(0),
                None,
            )?;
            program.bind_uniform_value_by_binding(
                &NORMALS_TEXTURE_UNIFORM_BINDING,
                &UniformValue::Integer1(1),
                None,
            )?;

            program
        } else {
            self.shader.lighting = false;
            let program = state
                .program_store_mut()
                .get_or_compile_program(&self.shader)?;
            program.use_program()?;

            program
        };

        // binds atoy_Universal
        program.mount_uniform_block_by_binding(
            &UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING,
            UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
        )?;
        program.bind_uniform_value_by_binding(
            &ALBEDO_TEXTURE_UNIFORM_BINDING,
            &UniformValue::Integer1(2),
            None,
        )?;

        state.do_computation([
            (
                positions_and_specular_shininess_texture,
                TextureUnit::TEXTURE0,
            ),
            (normals_texture, TextureUnit::TEXTURE1),
            (albedo_texture, TextureUnit::TEXTURE2),
        ])?;

        self.framebuffer.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        program.unuse_program()?;

        Ok(())
    }
}

const POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME: &'static str =
    "u_PositionsAndSpecularShininessTexture";
const POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(
        POSITIONS_AND_SPECULAR_SHININESS_TEXTURE_UNIFORM_NAME,
    ));

const NORMALS_TEXTURE_UNIFORM_NAME: &'static str = "u_NormalsTexture";
const NORMALS_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(NORMALS_TEXTURE_UNIFORM_NAME));

const ALBEDO_TEXTURE_UNIFORM_NAME: &'static str = "u_AlbedoTexture";
const ALBEDO_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(ALBEDO_TEXTURE_UNIFORM_NAME));

struct DeferredShader {
    lighting: bool,
}

impl DeferredShader {
    pub fn new() -> Self {
        Self { lighting: false }
    }
}

impl ProgramSource for DeferredShader {
    fn name(&self) -> Cow<'_, str> {
        if self.lighting {
            Cow::Borrowed("DeferredShader")
        } else {
            Cow::Borrowed("DeferredShader_Lighting")
        }
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/deferred.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        if self.lighting {
            let defines: &[Define<'_>] = &[
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
            ];
            Cow::Borrowed(&defines)
        } else {
            Cow::Borrowed(&[])
        }
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
