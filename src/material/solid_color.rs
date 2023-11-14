use std::{borrow::Cow, sync::OnceLock};

use palette::rgb::Rgba;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::render::webgl::program::{
    AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
};

use super::WebGLMaterial;

const NAME: &'static str = "SolidColorMaterial";
const COLOR_UNIFORM: &'static str = "u_Color";

static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 2]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "
attribute vec4 a_Position;

uniform mat4 u_ModelViewProjMatrix;

varying vec4 v_Position;

void main() {
    v_Position = u_ModelViewProjMatrix * a_Position;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "
uniform vec4 u_Color;

varying vec4 v_Position;

void main() {
    gl_FragColor = u_Color;
    gl_Position = v_Position;
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
        Self {
            color: Rgba::new(
                red.unwrap_or(0.0),
                green.unwrap_or(0.0),
                blue.unwrap_or(0.0),
                alpha.unwrap_or(0.0),
            ),
        }
    }
}

impl SolidColorMaterial {
    pub fn new() -> Self {
        Self {
            color: Rgba::default(),
        }
    }

    pub fn with_color(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            color: Rgba::new(red, green, blue, alpha),
        }
    }
}

impl WebGLMaterial for SolidColorMaterial {
    fn name(&self) -> &str {
        NAME
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[]
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

    fn attribute_value(&self, _name: &str) -> Option<&AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<&UniformValue> {
        todo!()
        // match name {
        //     COLOR_UNIFORM => Some(&UniformValue::FloatVector4 {
        //         data: Box::new(self.color),
        //         src_offset: 0,
        //         src_length: 0,
        //     }),
        //     _ => None,
        // }
    }
}
