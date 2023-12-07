use std::any::Any;

use palette::rgb::Rgb;

use crate::{
    entity::BorrowedMut,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

use super::Material;

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

uniform vec3 u_Color;

out vec4 out_Color;

void main() {
    out_Color = vec4(u_Color, 1.0);
}
";

#[derive(Debug, Clone, Copy)]
pub struct SolidColorMaterial {
    color: Rgb,
}

impl SolidColorMaterial {
    pub fn new() -> Self {
        Self {
            color: Rgb::default(),
        }
    }

    pub fn with_color(color: Rgb) -> Self {
        Self { color }
    }

    pub fn color(&self) -> Rgb {
        self.color
    }

    pub fn set_color(&mut self, color: Rgb) {
        self.color = color;
    }
}

impl Material for SolidColorMaterial {
    fn name(&self) -> &'static str {
        "SolidColorMaterial"
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

    fn sources(&self) -> &[ShaderSource] {
        &[
            ShaderSource::Vertex(VERTEX_SHADER_SOURCE),
            ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE),
        ]
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
            COLOR_UNIFORM => Some(UniformValue::FloatVector3([
                self.color.red,
                self.color.green,
                self.color.blue,
            ])),
            _ => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
