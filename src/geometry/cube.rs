use std::any::Any;

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Float32Array;

use crate::render::webgl::{
    attribute::AttributeValue,
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage},
    draw::{Draw, DrawMode},
    uniform::UniformValue,
};

use super::Geometry;

#[wasm_bindgen]
pub struct Cube {
    size: f64,
    vertices: BufferDescriptor,
    normals: BufferDescriptor,
    texture_coordinates: BufferDescriptor,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(size: Option<f64>) -> Self {
        Self::with_size(size.unwrap_or(1.0))
    }
}

impl Cube {
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    pub fn with_size(size: f64) -> Cube {
        let vertices = get_vertices_buffer(size);
        let vertices_buffer = Float32Array::new_with_length(vertices.len() as u32);
        let normal_buffer = Float32Array::new_with_length(NORMALS.len() as u32);
        let tex_coords_buffer = Float32Array::new_with_length(TEXTURE_COORDINATES.len() as u32);
        vertices_buffer.copy_from(&vertices);
        normal_buffer.copy_from(&NORMALS);
        tex_coords_buffer.copy_from(&TEXTURE_COORDINATES);
        Self {
            size,
            vertices: BufferDescriptor::from_float32_array(
                vertices_buffer,
                0,
                108,
                BufferUsage::StaticDraw,
            ),
            normals: BufferDescriptor::from_float32_array(
                normal_buffer,
                0,
                144,
                BufferUsage::StaticDraw,
            ),
            texture_coordinates: BufferDescriptor::from_float32_array(
                tex_coords_buffer,
                0,
                48,
                BufferUsage::StaticDraw,
            ),
        }
    }
}

#[wasm_bindgen]
impl Cube {
    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;

        let vertices = get_vertices_buffer(size);
        let vertices_buffer = Float32Array::new_with_length(vertices.len() as u32);
        vertices_buffer.copy_from(&vertices);
        self.vertices
            .buffer_sub_float32_array(vertices_buffer, 0, 0, 108);
    }
}

impl Geometry for Cube {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: 36,
        }
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.vertices.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn normals(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.normals.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Four,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.texture_coordinates.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<UniformValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[rustfmt::skip]
fn get_vertices_buffer(size: f64) -> [f32; 108] {
    let s = (size / 2.0) as f32;
    [
        -s,  s,  s,  -s, -s,  s,   s,  s,  s,   s,  s,  s,  -s, -s,  s,   s, -s,  s, // front
        -s,  s, -s,  -s,  s,  s,   s,  s, -s,   s,  s, -s,  -s,  s,  s,   s,  s,  s, // up
        -s,  s, -s,   s,  s, -s,  -s, -s, -s,   s,  s, -s,   s, -s, -s,  -s, -s, -s, // back
        -s, -s, -s,   s, -s, -s,  -s, -s,  s,   s, -s, -s,   s, -s,  s,  -s, -s,  s, // bottom
        -s,  s, -s,  -s, -s, -s,  -s,  s,  s,  -s,  s,  s,  -s, -s, -s,  -s, -s,  s, // left
         s,  s,  s,   s, -s,  s,   s,  s, -s,   s,  s, -s,   s, -s,  s,   s, -s, -s, // right
    ]
}

#[rustfmt::skip]
const NORMALS: [f32; 144] = [
     0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0, // front
     0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0, // up
     0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0, // back
     0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0, // bottom
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
     1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0, // right
];

#[rustfmt::skip]
const TEXTURE_COORDINATES: [f32; 48] = [
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // front
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // up
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // back
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // bottom
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // left
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // right
];
