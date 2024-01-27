use std::{any::Any, borrow::Cow};

use crate::{
    notify::Notifier,
    readonly::Readonly,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ProgramSource,
        shader::Define,
        state::FrameState,
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::{StandardMaterial, StandardMaterialSource, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,
    diffuse: UniformValue,
    notifier: Notifier<()>,
}

impl TextureMaterial {
    pub fn new(diffuse: UniformValue, transparency: Transparency) -> Self {
        Self {
            transparency,
            diffuse,
            notifier: Notifier::new(),
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn ready(&self) -> bool {
        true
    }

    fn prepare(&mut self, _: &mut FrameState) {}

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::FromMaterial(Cow::Borrowed("u_DiffuseTexture")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>> {
        match name {
            "u_DiffuseTexture" => Some(Readonly::Borrowed(&self.diffuse)),
            "u_Transparency" => Some(Readonly::Owned(UniformValue::Float1(
                self.transparency.alpha(),
            ))),
            "u_SpecularShininess" => Some(Readonly::Owned(UniformValue::Float1(128.0))),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue> {
        None
    }

    fn notifier(&mut self) -> &mut Notifier<()> {
        &mut self.notifier
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_standard_material_source(&self) -> &dyn StandardMaterialSource {
        self
    }

    fn as_program_source(&self) -> &dyn ProgramSource {
        self
    }
}

impl StandardMaterialSource for TextureMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("TextureMaterial")
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/texture_build_material.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Define> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Define> {
        vec![]
    }
}
