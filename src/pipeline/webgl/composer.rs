use std::borrow::Cow;

use gl_matrix4rust::vec4::Vec4;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::renderer::webgl::{
    error::Error,
    framebuffer::{
        AttachmentSource, Framebuffer, FramebufferAttachmentTarget, FramebufferBuilder,
        FramebufferTarget,
    },
    program::{Define, ProgramSource},
    state::FrameState,
    texture::{TextureUncompressedInternalFormat, TextureUnit},
    uniform::{UniformBinding, UniformValue},
};

const TEXTURE_UNIFORM_NAME: &'static str = "u_Texture";
const TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(TEXTURE_UNIFORM_NAME));

const GAMMA_UNIFORM_NAME: &'static str = "u_Gamma";
const GAMMA_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(GAMMA_UNIFORM_NAME));

pub const DEFAULT_CLEAR_COLOR: Vec4<f32> = Vec4::<f32>::new_zero();
pub const DEFAULT_ENABLE_GAMMA_CORRECTION: bool = true;
pub const DEFAULT_ENABLE_GAMMA: f32 = 2.2;

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    shader_provider: ComposerShaderProvider,
    composed_framebuffer: Framebuffer,
    clear_color: Vec4<f32>,

    enable_gamma_correction: bool,
    gamma: f32,
}

impl StandardComposer {
    pub fn new() -> Self {
        Self {
            shader_provider: ComposerShaderProvider::new(false),

            composed_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA8,
                ))
                .build(),
            clear_color: DEFAULT_CLEAR_COLOR,

            enable_gamma_correction: DEFAULT_ENABLE_GAMMA_CORRECTION,
            gamma: DEFAULT_ENABLE_GAMMA,
        }
    }

    pub fn clear_color(&self) -> &Vec4<f32> {
        &self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4<f32>) {
        self.clear_color = clear_color;
    }

    pub fn gamma_correction_enabled(&self) -> bool {
        self.enable_gamma_correction
    }

    pub fn enable_gamma_correction(&mut self) {
        self.enable_gamma_correction = true;
    }

    pub fn disable_gamma_correction(&mut self) {
        self.enable_gamma_correction = false;
    }

    pub fn gamma(&self) -> f32 {
        self.gamma
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
    }
}

impl StandardComposer {
    pub fn draw<'a, I>(&mut self, state: &mut FrameState, textures: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a WebGlTexture>,
    {
        self.compose(state, textures)?;
        self.print(state)?;
        Ok(())
    }

    fn compose<'a, I>(&mut self, state: &mut FrameState, textures: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a WebGlTexture>,
    {
        self.composed_framebuffer.init(state.gl())?;
        self.composed_framebuffer
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.composed_framebuffer.clear_buffers()?;

        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl().blend_func(
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        // disable gamma correction for composing
        self.shader_provider.enable_gamma_correction = false;
        let program = state
            .program_store_mut()
            .get_or_compile_program(&self.shader_provider)?;

        program.use_program()?;
        program.bind_uniform_value_by_binding(
            &TEXTURE_UNIFORM_BINDING,
            &UniformValue::Integer1(0),
            None,
        )?;

        for texture in textures {
            state.do_computation([(texture, TextureUnit::TEXTURE0)])?;
        }

        state.gl().disable(WebGl2RenderingContext::BLEND);
        state
            .gl()
            .blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);

        self.composed_framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        program.unuse_program()?;

        Ok(())
    }

    fn print(&mut self, state: &mut FrameState) -> Result<(), Error> {
        state.gl().clear_color(
            *self.clear_color.r() as f32,
            *self.clear_color.g() as f32,
            *self.clear_color.b() as f32,
            *self.clear_color.a() as f32,
        );
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // enable gamma correction if at the final print stage
        self.shader_provider.enable_gamma_correction = self.enable_gamma_correction;
        let program = state
            .program_store_mut()
            .get_or_compile_program(&self.shader_provider)?;
        program.use_program()?;
        if self.shader_provider.enable_gamma_correction {
            program.bind_uniform_value_by_binding(
                &GAMMA_UNIFORM_BINDING,
                &UniformValue::Float1(self.gamma),
                None,
            )?;
        }

        state.do_computation([(
            self.composed_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap(),
            TextureUnit::TEXTURE0,
        )])?;

        program.unuse_program()?;

        Ok(())
    }
}

struct ComposerShaderProvider {
    enable_gamma_correction: bool,
}

impl ComposerShaderProvider {
    fn new(enable_gamma_correction: bool) -> Self {
        Self {
            enable_gamma_correction,
        }
    }
}

impl ProgramSource for ComposerShaderProvider {
    fn name(&self) -> Cow<'_, str> {
        match self.enable_gamma_correction {
            true => Cow::Borrowed("Composer_Gamma"),
            false => Cow::Borrowed("Composer"),
        }
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("./shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("./shaders/composer.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        let defines: &[Define<'_>] = match self.enable_gamma_correction {
            true => &[Define::WithoutValue(Cow::Borrowed("USE_GAMMA_CORRECTION"))],
            false => &[],
        };
        Cow::Borrowed(defines)
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
