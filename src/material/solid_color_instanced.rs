use std::sync::OnceLock;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};
use palette::rgb::Rgba;

use crate::{
    ncor::Ncor,
    render::webgl::{
        buffer::{
            BufferComponentSize, BufferData, BufferDescriptor, BufferStatus, BufferTarget,
            BufferUsage,
        },
        program::{
            AttributeBinding, AttributeValue, BufferDataType, ShaderSource, UniformBinding,
            UniformValue,
        },
    },
};

use super::WebGLMaterial;

const COLOR_UNIFORM: &'static str = "u_Color";
const LOCAL_MODEL_MATRIX_ATTRIBUTE: &'static str = "a_LocalMatrix";

static ATTRIBUTE_BINDINGS: OnceLock<[AttributeBinding; 2]> = OnceLock::new();
static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 3]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es

in vec4 a_Position;
in mat4 a_LocalMatrix;
uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_LocalMatrix * a_Position;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec4 u_Color;

out vec4 outColor;

void main() {
    outColor = u_Color;
}
";

pub struct SolidColorInstancedMaterial {
    count: i32,
    color: Rgba,
    model_matrices: BufferDescriptor,
}

impl SolidColorInstancedMaterial {
    pub fn new(color: Rgba, count: i32, grid: i32, width: f32, height: f32) -> Self {
        let cell_width = width / (grid as f32);
        let cell_height = height / (grid as f32);
        let start_x = width / 2.0 - cell_width / 2.0;
        let start_z = height / 2.0 - cell_height / 2.0;

        let bytes_length = (16 * 4 * count) as usize;
        let mut data = Vec::with_capacity(bytes_length);
        for index in 0..count {
            let row = index / grid;
            let col = index % grid;

            let center_x = start_x - col as f32 * cell_width;
            let center_z = start_z - row as f32 * cell_height;
            data.extend_from_slice(
                Mat4::from_translation(Vec3::from_values(center_x, 0.0, center_z)).as_ref(),
            );
        }

        Self {
            color,
            count,
            model_matrices: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::fill_data(data, 0, bytes_length as u32),
                usage: BufferUsage::DynamicDraw,
            }),
        }
    }
}

impl WebGLMaterial for SolidColorInstancedMaterial {
    fn name(&self) -> &str {
        "SolidColorInstancedMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        ATTRIBUTE_BINDINGS.get_or_init(|| {
            [
                AttributeBinding::GeometryPosition,
                AttributeBinding::FromMaterial(String::from(LOCAL_MODEL_MATRIX_ATTRIBUTE)),
            ]
        })
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        UNIFORM_BINDINGS.get_or_init(|| {
            [
                UniformBinding::ModelMatrix,
                UniformBinding::ViewProjMatrix,
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

    fn attribute_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, AttributeValue>> {
        match name {
            LOCAL_MODEL_MATRIX_ATTRIBUTE => Some(Ncor::Owned(AttributeValue::InstancedBuffer {
                descriptor: Ncor::Borrowed(&self.model_matrices),
                target: BufferTarget::Buffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                components_length_per_instance: 4,
                divisor: 1,
            })),
            _ => None,
        }
    }

    fn uniform_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, UniformValue>> {
        match name {
            COLOR_UNIFORM => Some(Ncor::Owned(UniformValue::FloatVector4 {
                data: Box::new(self.color),
                src_offset: 0,
                src_length: 0,
            })),
            _ => None,
        }
    }

    fn instanced(&self) -> Option<i32> {
        Some(self.count)
    }
}
