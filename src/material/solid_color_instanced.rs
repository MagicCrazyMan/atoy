use gl_matrix4rust::mat4::{AsMat4, Mat4};
use palette::rgb::Rgb;

use crate::render::webgl::{
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage},
    program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
};

use super::Material;

const COLOR_ATTRIBUTE: &'static str = "a_Color";
const INSTANCE_MODEL_MATRIX_ATTRIBUTE: &'static str = "a_InstanceMatrix";

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
    instance_matrices_buffer: BufferDescriptor,
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
            matrices_data.extend(Mat4::from_translation(&(center_x, 0.0, center_z)).to_gl_binary());

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
            colors_buffer: BufferDescriptor::from_binary(
                colors_data,
                0,
                colors_bytes_length as u32,
                BufferUsage::StaticDraw,
            ),
            instance_matrices_buffer: BufferDescriptor::from_binary(
                matrices_data,
                0,
                matrices_bytes_length as u32,
                BufferUsage::StaticDraw,
            ),
        }
    }
}

impl Material for SolidColorInstancedMaterial {
    fn name(&self) -> &'static str {
        "SolidColorInstancedMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::FromMaterial(COLOR_ATTRIBUTE),
            AttributeBinding::FromMaterial(INSTANCE_MODEL_MATRIX_ATTRIBUTE),
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[UniformBinding::ModelMatrix, UniformBinding::ViewProjMatrix]
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
        Some(self.count)
    }

    fn attribute_value(&self, name: &str) -> Option<AttributeValue> {
        match name {
            COLOR_ATTRIBUTE => Some(AttributeValue::InstancedBuffer {
                descriptor: self.colors_buffer.clone(),
                target: BufferTarget::Buffer,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::Float,
                normalized: false,
                components_length_per_instance: 1,
                divisor: 1,
            }),
            INSTANCE_MODEL_MATRIX_ATTRIBUTE => Some(AttributeValue::InstancedBuffer {
                descriptor: self.instance_matrices_buffer.clone(),
                target: BufferTarget::Buffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                components_length_per_instance: 4,
                divisor: 1,
            }),
            _ => None,
        }
    }

    fn uniform_value<'a>(&'a self, _name: &str) -> Option<UniformValue<'a>> {
        None
    }
}
