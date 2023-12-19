use std::any::Any;

use palette::rgb::Rgba;

use crate::{
    entity::BorrowedMut,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::{ProgramSource, ShaderSource},
        uniform::{UniformBinding, UniformBlockBinding, UniformValue, UniformBlockValue},
    },
};

use super::{Material, Transparency};

const COLOR_UNIFORM: &'static str = "u_Color";

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewMatrix;
uniform mat4 u_ProjMatrix;

void main() {
    gl_Position = u_ProjMatrix * u_ViewMatrix * u_ModelMatrix * a_Position;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec4 u_Color;

out vec4 out_Color;

void main() {
    out_Color = u_Color;
}
";

#[derive(Debug, Clone, Copy)]
pub struct SolidColorMaterial {
    color: Rgba,
}

impl SolidColorMaterial {
    pub fn new() -> Self {
        Self {
            color: Rgba::default(),
        }
    }

    pub fn with_color(color: Rgba) -> Self {
        Self { color }
    }

    pub fn color(&self) -> Rgba {
        self.color
    }

    pub fn set_color(&mut self, color: Rgba) {
        self.color = color;
    }
}

impl ProgramSource for SolidColorMaterial {
    fn name(&self) -> &'static str {
        "SolidColorMaterial"
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::VertexRaw(VERTEX_SHADER_SOURCE),
            ShaderSource::FragmentRaw(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewMatrix,
            UniformBinding::ProjMatrix,
            UniformBinding::FromMaterial(COLOR_UNIFORM),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }
}

impl Material for SolidColorMaterial {
    fn transparency(&self) -> Transparency {
        if self.color.alpha == 0.0 {
            Transparency::Transparent
        } else if self.color.alpha == 1.0 {
            Transparency::Opaque
        } else {
            Transparency::Translucent(self.color.alpha)
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            COLOR_UNIFORM => Some(UniformValue::Float4(
                self.color.red,
                self.color.green,
                self.color.blue,
                self.color.alpha,
            )),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformBlockValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
