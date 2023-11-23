use std::any::Any;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::render::webgl::{
    attribute::AttributeValue,
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage},
    draw::{Draw, DrawElementType, DrawMode},
    uniform::UniformValue,
};

use super::Geometry;

#[wasm_bindgen]
pub struct IndexedCube {
    size: f64,
    indices: BufferDescriptor,
    vertices: BufferDescriptor,
    normals: BufferDescriptor,
    texture_coordinates: BufferDescriptor,
}

#[wasm_bindgen]
impl IndexedCube {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(size: Option<f64>) -> Self {
        Self::with_size(size.unwrap_or(1.0))
    }
}

impl IndexedCube {
    pub fn new() -> IndexedCube {
        Self::with_size(1.0)
    }

    pub fn with_size(size: f64) -> IndexedCube {
        Self {
            size,
            indices: BufferDescriptor::from_binary(&INDICES, 0, 36, BufferUsage::StaticDraw),
            vertices: BufferDescriptor::from_binary(
                get_vertices_buffer(size),
                0,
                72 * 4,
                BufferUsage::StaticDraw,
            ),
            normals: BufferDescriptor::from_binary(
                NORMALS_BINARY,
                0,
                96 * 4,
                BufferUsage::StaticDraw,
            ),
            texture_coordinates: BufferDescriptor::from_binary(
                TEXTURE_COORDINATES_BINARY,
                0,
                48 * 4,
                BufferUsage::StaticDraw,
            ),
        }
    }
}

#[wasm_bindgen]

impl IndexedCube {
    pub fn size(&self) -> f64 {
        self.size
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
        self.vertices
            .buffer_sub_data(get_vertices_buffer(size), 0, 0, 72 * 4);
    }
}

impl Geometry for IndexedCube {
    fn draw(&self) -> Draw {
        Draw::Elements {
            mode: DrawMode::Triangles,
            count: 36,
            element_type: DrawElementType::UnsignedByte,
            offset: 0,
            indices: self.indices.clone(),
        }
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.vertices.clone(),
            target: BufferTarget::Buffer,
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
            target: BufferTarget::Buffer,
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
            target: BufferTarget::Buffer,
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
fn get_vertices_buffer(size: f64) -> Vec<u8> {
    let s = (size / 2.0) as f32;
    [
        s, s, s,  -s, s, s,  -s,-s, s,   s,-s, s,  // v0-v1-v2-v3 front
        s, s, s,   s,-s, s,   s,-s,-s,   s, s,-s,  // v0-v3-v4-v5 right
        s, s, s,   s, s,-s,  -s, s,-s,  -s, s, s,  // v0-v5-v6-v1 top
       -s, s, s,  -s, s,-s,  -s,-s,-s,  -s,-s, s,  // v1-v6-v7-v2 left
       -s,-s,-s,   s,-s,-s,   s,-s, s,  -s,-s, s,  // v7-v4-v3-v2 bottom
        s,-s,-s,  -s,-s,-s,  -s, s,-s,   s, s,-s,  // v4-v7-v6-v5 back
    ]
    .iter()
    .flat_map(|v| v.to_ne_bytes())
    .collect::<Vec<_>>()
}

#[rustfmt::skip]
const TEXTURE_COORDINATES: [f32; 48] = [
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // front
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // right
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // top
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // left
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // bottom
    1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // back
];
const TEXTURE_COORDINATES_BINARY: &[u8; 48 * 4] =
    unsafe { std::mem::transmute::<&[f32; 48], &[u8; 48 * 4]>(&TEXTURE_COORDINATES) };

#[rustfmt::skip]
const NORMALS: [f32; 96] = [
     0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0, // front
     1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0, // right
     0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0, // top
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
     0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0, // bottom
     0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0, // back
];
const NORMALS_BINARY: &[u8; 96 * 4] =
    unsafe { std::mem::transmute::<&[f32; 96], &[u8; 96 * 4]>(&NORMALS) };

#[rustfmt::skip]
const INDICES: [u8; 36] = [
    0,  1,  2,  0,  2,  3, // front
    4,  5,  6,  4,  6,  7, // up
    8,  9, 10,  8, 10, 11, // back
   12, 13, 14, 12, 14, 15, // bottom
   16, 17, 18, 16, 18, 19, // left
   20, 21, 22, 20, 22, 23, // right
];
