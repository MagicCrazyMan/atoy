use std::any::Any;

use rand::distributions::{Distribution, Standard};

use crate::{
    entity::Entity,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::{ProgramSource, ShaderSource},
            shader::{ShaderBuilder, ShaderType, Variable},
            uniform::{
                UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
                UniformValue,
            },
        },
    },
};

pub mod environment_mapping;
pub mod icon;
pub mod loader;
pub mod multiple_textures_instanced;
pub mod solid_color;
pub mod solid_color_instanced;
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

pub trait Material: ProgramSource {
    /// Returns transparency of this material.
    fn transparency(&self) -> Transparency;

    /// Returns an attribute value by an attribute variable name.
    fn attribute_value(&self, name: &str, entity: &Entity) -> Option<AttributeValue>;

    /// Returns an uniform value by an uniform variable name.
    fn uniform_value(&self, name: &str, entity: &Entity) -> Option<UniformValue>;

    /// Returns an uniform block buffer binding value by an uniform block interface name.
    fn uniform_block_value(&self, name: &str, entity: &Entity) -> Option<UniformBlockValue>;

    /// Returns `true` if material is ready for drawing.
    /// Drawer skips entity drawing if material is not ready.
    fn ready(&self) -> bool;

    fn instanced(&self) -> Option<i32>;

    /// Preparation before entering drawing stage.
    ///
    /// Depending on [`MaterialPolicy`](crate::render::webgl::pipeline::policy::MaterialPolicy),
    /// `self` is not always extracted from entity. Thus, if you are not sure where the `self` from,
    /// do not borrow material from entity.
    fn prepare(&mut self, state: &mut State, entity: &Entity);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A standard material source for building up a standard material.
/// Standard material source implements [`ProgramSource`] in default,
/// material implemented under this trait gains the abilities of
/// drawing basic effects, such as lighting, gamma correction and etc.
pub trait StandardMaterialSource {
    /// Returns a material name.
    fn name(&self) -> &'static str;

    /// Returns a process function for vertex shader.
    /// Uses a default one if none.
    fn vertex_process(&self) -> Option<&'static str> {
        None
    }

    /// Returns a process function for fragment shader.
    fn fragment_process(&self) -> &'static str;

    /// Returns custom vertex shader variables.
    fn vertex_variables(&self) -> Vec<Variable>;

    /// Returns custom fragment shader variables.
    fn fragment_variables(&self) -> Vec<Variable>;

    /// Returns custom attribute bindings.
    fn attribute_bindings(&self) -> Vec<AttributeBinding>;

    /// Returns custom uniform bindings.
    fn uniform_bindings(&self) -> Vec<UniformBinding>;

    /// Returns custom uniform structural bindings.
    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding>;

    /// Returns custom uniform block bindings.
    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding>;
}

impl<T> ProgramSource for T
where
    T: StandardMaterialSource,
{
    fn name(&self) -> &'static str {
        self.name()
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::Builder(ShaderBuilder::new(
                ShaderType::Vertex,
                true,
                [
                    include_str!("./standard/constants.glsl"),
                    include_str!("./standard/constants_vert.glsl"),
                ],
                self.vertex_variables(),
                [
                    self.vertex_process()
                        .unwrap_or(include_str!("./standard/default_process_vert.glsl")),
                    include_str!("./standard/entry_vert.glsl"),
                ],
            )),
            ShaderSource::Builder(ShaderBuilder::new(
                ShaderType::Fragment,
                true,
                [
                    include_str!("./standard/constants.glsl"),
                    include_str!("./standard/constants_frag.glsl"),
                ],
                self.fragment_variables(),
                [
                    include_str!("./standard/gamma.glsl"),
                    include_str!("./standard/lighting.glsl"),
                    self.fragment_process(),
                    include_str!("./standard/entry_frag.glsl"),
                ],
            )),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        self.attribute_bindings()
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        self.uniform_bindings()
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        self.uniform_structural_bindings()
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        let mut bindings = vec![
            UniformBlockBinding::StandardUniversalUniforms,
            UniformBlockBinding::StandardLights,
        ];
        bindings.extend(self.uniform_block_bindings());
        bindings
    }
}
