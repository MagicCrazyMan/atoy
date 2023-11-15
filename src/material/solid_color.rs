use std::sync::OnceLock;

use palette::rgb::Rgba;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

use crate::{
    ncor::Ncor,
    render::webgl::program::{
        AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
    },
};

use super::WebGLMaterial;

const COLOR_UNIFORM: &'static str = "u_Color";

static ATTRIBUTE_BINDINGS: OnceLock<[AttributeBinding; 1]> = OnceLock::new();
static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 2]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "
attribute vec4 a_Position;

uniform mat4 u_ModelViewProjMatrix;

void main() {
    gl_Position = u_ModelViewProjMatrix * a_Position;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec4 u_Color;

void main() {
    gl_FragColor = u_Color;
}
";

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct SolidColorMaterial {
    color: Rgba,
}

#[wasm_bindgen]
impl SolidColorMaterial {
    #[wasm_bindgen]
    pub fn new_constructor(
        red: Option<f32>,
        green: Option<f32>,
        blue: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self::with_color(Rgba::new(
            red.unwrap_or(1.0),
            green.unwrap_or(0.0),
            blue.unwrap_or(0.0),
            alpha.unwrap_or(1.0),
        ))
    }
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
}

impl WebGLMaterial for SolidColorMaterial {
    fn name(&self) -> &str {
        "SolidColorMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        ATTRIBUTE_BINDINGS.get_or_init(|| [AttributeBinding::GeometryPosition])
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        UNIFORM_BINDINGS.get_or_init(|| {
            [
                UniformBinding::ModelViewProjMatrix,
                UniformBinding::FromMaterial(COLOR_UNIFORM.to_string()),
            ]
        })
    }

    fn sources(&self) -> &[ShaderSource] {
        SHADER_SOURCES.get_or_init(|| {
            [
                ShaderSource::Vertex(VERTEX_SHADER_SOURCE.to_string()),
                ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE.to_string()),
            ]
        })
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, AttributeValue>> {
        None
    }

    fn uniform_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, UniformValue>> {
        match name {
            COLOR_UNIFORM => Some(Ncor::Owned(UniformValue::FloatVector4 {
                data: Box::new(self.color),
                src_offset: 0,
                src_length: 4,
            })),
            _ => None,
        }
    }
}
