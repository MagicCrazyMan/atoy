use std::any::Any;

use gl_matrix4rust::mat4::Mat4;
use palette::rgb::Rgb;
use web_sys::js_sys::Float32Array;

use crate::render::webgl::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{
        BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
        BufferUsage,
    },
    program::ShaderSource,
    uniform::{UniformBinding, UniformValue},
};

use super::{Material, MaterialRenderEntity};

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
    count: usize,
    colors: BufferDescriptor,
    instance_matrices: BufferDescriptor,
}

impl SolidColorInstancedMaterial {
    pub fn new(count: usize, grid: usize, width: f64, height: f64) -> Self {
        let cell_width = width / (grid as f64);
        let cell_height = height / (grid as f64);
        let start_x = width / 2.0 - cell_width / 2.0;
        let start_z = height / 2.0 - cell_height / 2.0;

        let matrices_length = (16 * count) as u32;
        let colors_length = (3 * count) as u32;
        let matrices_data = Float32Array::new_with_length(matrices_length);
        let colors_data = Float32Array::new_with_length(colors_length);
        for index in 0..count {
            let row = index / grid;
            let col = index % grid;

            let center_x = (start_x - col as f64 * cell_width) as f32;
            let center_z = (start_z - row as f64 * cell_height) as f32;
            let matrix = Mat4::from_translation(&(center_x, 0.0, center_z));
            matrices_data.set_index((index * 16) as u32 + 0, matrix.raw()[0]);
            matrices_data.set_index((index * 16) as u32 + 1, matrix.raw()[1]);
            matrices_data.set_index((index * 16) as u32 + 2, matrix.raw()[2]);
            matrices_data.set_index((index * 16) as u32 + 3, matrix.raw()[3]);
            matrices_data.set_index((index * 16) as u32 + 4, matrix.raw()[4]);
            matrices_data.set_index((index * 16) as u32 + 5, matrix.raw()[5]);
            matrices_data.set_index((index * 16) as u32 + 6, matrix.raw()[6]);
            matrices_data.set_index((index * 16) as u32 + 7, matrix.raw()[7]);
            matrices_data.set_index((index * 16) as u32 + 8, matrix.raw()[8]);
            matrices_data.set_index((index * 16) as u32 + 9, matrix.raw()[9]);
            matrices_data.set_index((index * 16) as u32 + 10, matrix.raw()[10]);
            matrices_data.set_index((index * 16) as u32 + 11, matrix.raw()[11]);
            matrices_data.set_index((index * 16) as u32 + 12, matrix.raw()[12]);
            matrices_data.set_index((index * 16) as u32 + 13, matrix.raw()[13]);
            matrices_data.set_index((index * 16) as u32 + 14, matrix.raw()[14]);
            matrices_data.set_index((index * 16) as u32 + 15, matrix.raw()[15]);

            let Rgb {
                blue, green, red, ..
            } = rand::random::<Rgb>();
            colors_data.set_index((index * 3) as u32 + 0, red);
            colors_data.set_index((index * 3) as u32 + 1, green);
            colors_data.set_index((index * 3) as u32 + 2, blue);
        }

        Self {
            count,
            colors: BufferDescriptor::new(
                BufferSource::from_float32_array(colors_data, 0, colors_length),
                BufferUsage::StaticDraw,
            ),
            instance_matrices: BufferDescriptor::new(
                BufferSource::from_float32_array(matrices_data, 0, matrices_length),
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
        Some(self.count as i32)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn attribute_value(&self, name: &str, _: &MaterialRenderEntity) -> Option<AttributeValue> {
        match name {
            COLOR_ATTRIBUTE => Some(AttributeValue::InstancedBuffer {
                descriptor: self.colors.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 1,
                divisor: 1,
            }),
            INSTANCE_MODEL_MATRIX_ATTRIBUTE => Some(AttributeValue::InstancedBuffer {
                descriptor: self.instance_matrices.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 4,
                divisor: 1,
            }),
            _ => None,
        }
    }

    fn uniform_value(&self, _: &str, _: &MaterialRenderEntity) -> Option<UniformValue> {
        None
    }
}
