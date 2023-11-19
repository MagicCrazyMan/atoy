use palette::rgb::Rgb;

use crate::{render::webgl::program::{
    AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
}, geometry::cube::Cube};

use super::WebGLMaterial;

const COLOR_UNIFORM: &'static str = "u_Color";

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;

uniform mat4 u_ModelViewProjMatrix;

void main() {
    gl_Position = u_ModelViewProjMatrix * a_Position;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec3 u_Color;

out vec4 out_color;

void main() {
    out_color = vec4(u_Color, 1.0);
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
}

impl WebGLMaterial for SolidColorMaterial {
    fn name(&self) -> &'static str {
        "SolidColorMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelViewProjMatrix,
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

    fn attribute_value<'a>(&'a self, _name: &str) -> Option<AttributeValue<'a>> {
        None
    }

    fn uniform_value<'a>(&'a self, name: &str) -> Option<UniformValue<'a>> {
        match name {
            COLOR_UNIFORM => Some(UniformValue::FloatVector3 {
                data: &self.color,
                src_offset: 0,
                src_length: 3,
            }),
            _ => None,
        }
    }
}
