use std::borrow::Cow;

use gl_matrix4rust::vec4::Vec4;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::renderer::webgl::{
    error::Error,
    framebuffer::{
        AttachmentProvider, Framebuffer, FramebufferAttachment, FramebufferBuilder,
        FramebufferTarget,
    },
    program::{Define, ShaderProvider},
    state::FrameState,
    texture::{TextureColorFormat, TextureUnit},
    uniform::UniformValue,
};

const TEXTURE_UNIFORM_NAME: &'static str = "u_Texture";
const GAMMA_UNIFORM_NAME: &'static str = "u_Gamma";

pub const DEFAULT_CLEAR_COLOR: Vec4<f32> = Vec4::<f32>::new_zero();
pub const DEFAULT_ENABLE_GAMMA_CORRECTION: bool = true;
pub const DEFAULT_ENABLE_GAMMA: f32 = 2.2;

/// Standard texture composer.
/// Composes all textures into canvas framebuffer.
pub struct StandardComposer {
    shader_provider: ComposerShaderProvider,
    composed_framebuffer: Option<Framebuffer>,
    clear_color: Vec4<f32>,

    enable_gamma_correction: bool,
    gamma: f32,
}

impl StandardComposer {
    pub fn new() -> Self {
        Self {
            shader_provider: ComposerShaderProvider::new(false),

            composed_framebuffer: None,
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
    fn composed_framebuffer(&mut self, state: &mut FrameState) -> &mut Framebuffer {
        self.composed_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new().set_color_attachment0(AttachmentProvider::new_texture(
                    TextureColorFormat::RGBA8,
                )),
            )
        })
    }

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
        self.composed_framebuffer(state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.composed_framebuffer(state).clear_buffers()?;

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
            .use_program(&self.shader_provider)?;
        state.bind_uniform_value_by_variable_name(
            program,
            TEXTURE_UNIFORM_NAME,
            &UniformValue::Integer1(0),
        )?;

        for texture in textures {
            state.do_computation([(texture, TextureUnit::TEXTURE0)])?;
        }

        state.gl().disable(WebGl2RenderingContext::BLEND);
        state
            .gl()
            .blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);

        self.composed_framebuffer(state).unbind();

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
            .use_program(&self.shader_provider)?;
        if self.shader_provider.enable_gamma_correction {
            state.bind_uniform_value_by_variable_name(
                program,
                GAMMA_UNIFORM_NAME,
                &UniformValue::Float1(self.gamma),
            )?;
        }

        state.do_computation([(
            self.composed_framebuffer
                .as_ref()
                .unwrap()
                .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                .unwrap(),
            TextureUnit::TEXTURE0,
        )])?;

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

impl ShaderProvider for ComposerShaderProvider {
    fn name(&self) -> Cow<'static, str> {
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

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        match self.enable_gamma_correction {
            true => &[Define::WithoutValue(Cow::Borrowed("USE_GAMMA_CORRECTION"))],
            false => &[],
        }
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
