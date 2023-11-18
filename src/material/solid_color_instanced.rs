use std::sync::OnceLock;

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};
use palette::rgb::Rgb;
use wasm_bindgen_test::console_log;

use crate::{
    ncor::Ncor,
    render::webgl::{
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage,
        },
        program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
    },
};

use super::WebGLMaterial;

const COLOR_ATTRIBUTE: &'static str = "a_Color";
const INSTANCE_MODEL_MATRIX_ATTRIBUTE: &'static str = "a_InstanceMatrix";

static ATTRIBUTE_BINDINGS: OnceLock<[AttributeBinding; 3]> = OnceLock::new();
static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 2]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es

in vec4 a_Position;
in vec3 a_Color;
in mat4 a_InstanceMatrix;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec3 v_Color;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_InstanceMatrix * a_Position;
    v_Color = a_Color;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

in vec3 v_Color;

out vec4 out_Color;

void main() {
    out_Color = vec4(v_Color, 1.0);
}
";

pub struct SolidColorInstancedMaterial {
    count: i32,
    colors_buffer: BufferDescriptor,
    model_matrices_buffer: BufferDescriptor,
}

impl SolidColorInstancedMaterial {
    pub fn new(count: i32, grid: i32, width: f64, height: f64) -> Self {
        let cell_width = width / (grid as f64);
        let cell_height = height / (grid as f64);
        let start_x = width / 2.0 - cell_width / 2.0;
        let start_z = height / 2.0 - cell_height / 2.0;

        let matrices_bytes_length = (16 * 4 * count) as usize;
        let colors_bytes_length = (3 * 4 * count) as usize;
        let mut matrices_data = Vec::with_capacity(matrices_bytes_length);
        let mut colors_data = Vec::with_capacity(colors_bytes_length);
        for index in 0..count {
            let row = index / grid;
            let col = index % grid;

            let center_x = start_x - col as f64 * cell_width;
            let center_z = start_z - row as f64 * cell_height;
            matrices_data.extend_from_slice(
                Mat4::<f32>::from_translation(Vec3::from_values(center_x as f32, 0.0, center_z as f32)).as_ref(),
            );

            let Rgb {
                blue, green, red, ..
            } = rand::random::<Rgb>();
            colors_data.extend(
                [red, green, blue]
                    .iter()
                    .flat_map(|component| component.to_ne_bytes())
                    .collect::<Vec<_>>(),
            );
        }

        Self {
            count,
            colors_buffer: BufferDescriptor::with_binary(
                colors_data,
                0,
                colors_bytes_length as u32,
                BufferUsage::StaticDraw,
            ),
            model_matrices_buffer: BufferDescriptor::with_binary(
                matrices_data,
                0,
                matrices_bytes_length as u32,
                BufferUsage::StaticDraw,
            ),
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
                AttributeBinding::FromMaterial(String::from(COLOR_ATTRIBUTE)),
                AttributeBinding::FromMaterial(String::from(INSTANCE_MODEL_MATRIX_ATTRIBUTE)),
            ]
        })
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        UNIFORM_BINDINGS
            .get_or_init(|| [UniformBinding::ModelMatrix, UniformBinding::ViewProjMatrix])
    }

    fn sources(&self) -> &[ShaderSource] {
        SHADER_SOURCES.get_or_init(|| {
            [
                ShaderSource::Vertex(VERTEX_SHADER_SOURCE.to_string()),
                ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE.to_string()),
            ]
        })
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        Some(self.count)
    }

    fn attribute_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, AttributeValue>> {
        match name {
            COLOR_ATTRIBUTE => Some(Ncor::Owned(AttributeValue::InstancedBuffer {
                descriptor: Ncor::Borrowed(&self.colors_buffer),
                target: BufferTarget::Buffer,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::Float,
                normalized: false,
                components_length_per_instance: 1,
                divisor: 1,
            })),
            INSTANCE_MODEL_MATRIX_ATTRIBUTE => Some(Ncor::Owned(AttributeValue::InstancedBuffer {
                descriptor: Ncor::Borrowed(&self.model_matrices_buffer),
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

    fn uniform_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, UniformValue>> {
        None
    }
}
