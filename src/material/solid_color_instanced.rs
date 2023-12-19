use std::any::Any;

use gl_matrix4rust::mat4::Mat4;
use palette::rgb::{Rgb, Rgba};
use web_sys::js_sys::Float32Array;

use crate::{
    entity::BorrowedMut,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        program::{ProgramSource, ShaderSource},
        uniform::{UniformBinding, UniformBlockBinding, UniformValue, UniformBlockValue},
    },
};

use super::{Material, Transparency};

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es

in vec4 a_Position;
in vec4 a_Color;
in mat4 a_InstanceMatrix;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec4 v_Color;

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

in vec4 v_Color;

out vec4 o_Color;

void main() {
    o_Color = v_Color;
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
        let colors_length = (4 * count) as u32;
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

            let Rgba { color, alpha } = rand::random::<Rgba>();
            let Rgb {
                red, green, blue, ..
            } = color;
            colors_data.set_index((index * 4) as u32 + 0, red);
            colors_data.set_index((index * 4) as u32 + 1, green);
            colors_data.set_index((index * 4) as u32 + 2, blue);
            colors_data.set_index((index * 4) as u32 + 3, alpha);
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

    pub fn random_colors(&mut self) {
        let colors_length = (4 * self.count) as u32;
        let colors_data = Float32Array::new_with_length(colors_length);
        for index in 0..self.count {
            let Rgba { color, alpha } = rand::random::<Rgba>();
            let Rgb {
                red, green, blue, ..
            } = color;
            colors_data.set_index((index * 4) as u32 + 0, red);
            colors_data.set_index((index * 4) as u32 + 1, green);
            colors_data.set_index((index * 4) as u32 + 2, blue);
            colors_data.set_index((index * 4) as u32 + 3, alpha);
        }

        self.colors
            .buffer_sub_data(BufferSource::from_float32_array(colors_data, 0, colors_length), 0);
    }
}

impl ProgramSource for SolidColorInstancedMaterial {
    fn name(&self) -> &'static str {
        "SolidColorInstancedMaterial"
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(VERTEX_SHADER_SOURCE),
            ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::FromMaterial("a_Color"),
            AttributeBinding::FromMaterial("a_InstanceMatrix"),
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[UniformBinding::ModelMatrix, UniformBinding::ViewProjMatrix]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }
}

impl Material for SolidColorInstancedMaterial {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        Some(self.count as i32)
    }

    fn attribute_value(&self, name: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        match name {
            "a_Color" => Some(AttributeValue::InstancedBuffer {
                descriptor: self.colors.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 1,
                divisor: 1,
            }),
            "a_InstanceMatrix" => Some(AttributeValue::InstancedBuffer {
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

    fn uniform_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformValue> {
        None
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
