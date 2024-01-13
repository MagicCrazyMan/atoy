use std::{any::Any, borrow::Cow};

use rand::distributions::{Distribution, Standard};

use crate::{
    entity::Entity,
    event::EventAgency,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
        shader::ShaderBuilder,
        state::FrameState,
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

pub mod loader;
pub mod solid_color;
pub mod texture_mapping;

/// Material transparency.
#[derive(Clone, Copy, PartialEq)]
pub enum Transparency {
    Opaque,
    Transparent,
    Translucent(f32),
}

impl Distribution<Transparency> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Transparency {
        let alpha = rng.gen::<f32>();
        if alpha == 1.0 {
            Transparency::Opaque
        } else if alpha == 0.0 {
            Transparency::Transparent
        } else {
            Transparency::Translucent(alpha)
        }
    }
}

impl Transparency {
    /// Returns alpha.
    pub fn alpha(&self) -> f32 {
        match self {
            Transparency::Opaque => 1.0,
            Transparency::Transparent => 0.0,
            Transparency::Translucent(alpha) => *alpha,
        }
    }
}

pub trait StandardMaterial: StandardMaterialSource {
    /// Returns `true` if material is ready for drawing.
    /// Drawer skips entity drawing if material is not ready.
    fn ready(&self) -> bool;

    /// Prepares material.
    fn prepare(&mut self, state: &mut FrameState, entity: &Entity);

    /// Returns transparency of this material.
    fn transparency(&self) -> Transparency;

    /// Returns attribute bindings requirements.
    fn attribute_bindings(&self) -> &[AttributeBinding];

    /// Returns uniform bindings requirements.
    fn uniform_bindings(&self) -> &[UniformBinding];

    /// Returns uniform block bindings requirements.
    fn uniform_block_bindings(&self) -> &[UniformBlockBinding];

    /// Returns an attribute value by an attribute variable name.
    fn attribute_value(&self, name: &str) -> Option<AttributeValue>;

    /// Returns an uniform value by an uniform variable name.
    fn uniform_value(&self, name: &str) -> Option<UniformValue>;

    /// Returns an uniform block buffer binding value by an uniform block name.
    fn uniform_block_value(&self, name: &str) -> Option<UniformBlockValue>;

    fn changed_event(&self) -> &EventAgency<()>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_standard_program_source(&self) -> &dyn StandardMaterialSource;

    fn as_program_source(&self) -> &dyn ProgramSource;
}

/// A standard material source for building up a standard material.
/// Standard material source implements [`ProgramSource`] in default,
/// material implemented under this trait gains the abilities of
/// drawing basic effects, such as lighting, gamma correction and etc.
pub trait StandardMaterialSource {
    /// Returns a material name.
    fn name(&self) -> Cow<'static, str>;

    /// Returns a process function for vertex shader.
    /// Uses a default one if none.
    fn vertex_process(&self) -> Option<Cow<'static, str>>;

    /// Returns a process function for fragment shader.
    fn fragment_process(&self) -> Cow<'static, str>;

    /// Returns custom vertex shader defines arguments.
    fn vertex_defines(&self) -> Vec<Cow<'static, str>>;

    /// Returns custom fragment shader defines arguments.
    fn fragment_defines(&self) -> Vec<Cow<'static, str>>;
}

static DEFAULT_VERTEX_PROCESS: Cow<'static, str> =
    Cow::Borrowed(include_str!("./shaders/default_process_vert.glsl"));

impl<S> ProgramSource for S
where
    S: StandardMaterialSource,
{
    fn name(&self) -> Cow<'static, str> {
        self.name()
    }

    fn vertex_source(&self) -> VertexShaderSource {
        let vertex_process = self
            .vertex_process()
            .unwrap_or(DEFAULT_VERTEX_PROCESS.clone());
        VertexShaderSource::Builder(ShaderBuilder::new(
            true,
            self.vertex_defines(),
            vec![
                Cow::Borrowed(include_str!("./shaders/constants.glsl")),
                Cow::Borrowed(include_str!("./shaders/constants_vert.glsl")),
            ],
            vec![
                vertex_process,
                Cow::Borrowed(include_str!("./shaders/entry_vert.glsl")),
            ],
        ))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Builder(ShaderBuilder::new(
            true,
            self.fragment_defines(),
            vec![
                Cow::Borrowed(include_str!("./shaders/constants.glsl")),
                Cow::Borrowed(include_str!("./shaders/constants_frag.glsl")),
            ],
            vec![
                Cow::Borrowed(include_str!("./shaders/lighting.glsl")),
                Cow::Borrowed(include_str!("./shaders/bloom.glsl")),
                self.fragment_process(),
                Cow::Borrowed(include_str!("./shaders/entry_frag.glsl")),
            ],
        ))
    }
}
