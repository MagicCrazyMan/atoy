use std::any::Any;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    ncor::Ncor,
    render::webgl::{
        buffer::{
            BufferComponentSize, BufferData, BufferDescriptor, BufferStatus, BufferSubData,
            BufferTarget, BufferUsage,
        },
        draw::{Draw, DrawElementType, DrawMode},
        program::{AttributeValue, BufferDataType, UniformValue},
    },
};

use super::Geometry;

#[wasm_bindgen]
pub struct IndexedCube {
    size: f32,
    indices_buffer: BufferDescriptor,
    vertices_buffer: BufferDescriptor,
    normals_buffer: BufferDescriptor,
}

#[wasm_bindgen]
impl IndexedCube {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(size: Option<f32>) -> Self {
        Self::with_size(size.unwrap_or(1.0))
    }
}

impl IndexedCube {
    pub fn new() -> IndexedCube {
        Self::with_size(1.0)
    }

    pub fn with_size(size: f32) -> IndexedCube {
        Self {
            size,
            indices_buffer: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::fill_data(get_indices_buffer(), 0, 36),
                usage: BufferUsage::StaticDraw,
            }),
            vertices_buffer: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::fill_data(get_vertices_buffer(size), 0, 72 * 4),
                usage: BufferUsage::StaticDraw,
            }),
            normals_buffer: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::fill_data(get_normals_buffer(), 0, 96 * 4),
                usage: BufferUsage::StaticDraw,
            }),
        }
    }
}

#[wasm_bindgen]

impl IndexedCube {
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
        self.vertices_buffer.buffer_sub_data(BufferSubData::new(
            get_vertices_buffer(size),
            0,
            0,
            72 * 4,
        ));
    }
}

impl Geometry for IndexedCube {
    fn draw<'a>(&'a self) -> Draw<'a> {
        Draw::Elements {
            mode: DrawMode::Triangles,
            count: 36,
            element_type: DrawElementType::UnsignedByte,
            offset: 0,
            indices: Ncor::Borrowed(&self.indices_buffer),
        }
    }

    fn vertices<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.vertices_buffer),
            target: BufferTarget::Buffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        }))
    }

    fn normals<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.normals_buffer),
            target: BufferTarget::Buffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        }))
    }

    fn texture_coordinates<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        todo!()
    }

    fn attribute_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, AttributeValue>> {
        None
    }

    fn uniform_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, UniformValue>> {
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
fn get_vertices_buffer(size: f32) -> Vec<u8> {
    let s = size / 2.0;
    [
         s, s, s,  -s, s, s,  -s,-s, s,   s,-s, s, // front
         s, s,-s,  -s, s,-s,  -s, s, s,   s, s, s, // up
         s, s,-s,  -s, s,-s,  -s,-s,-s,   s,-s,-s, // back
         s,-s,-s,  -s,-s,-s,  -s,-s, s,   s,-s, s, // bottom
        -s, s, s,  -s, s,-s,  -s,-s,-s,  -s,-s, s, // left
         s, s,-s,   s, s, s,   s,-s, s,   s,-s,-s, // right
    ]
    .iter()
    .flat_map(|v| v.to_ne_bytes())
    .collect::<Vec<_>>()
}

#[rustfmt::skip]
const NORMALS: [f32; 96] = [
    0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0, // front
    0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0, // up
    0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0, // back
    0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0, // bottom
   -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
    1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0, // right
];
#[inline]
fn get_normals_buffer() -> &'static [u8] {
    unsafe { std::mem::transmute::<&[f32; 96], &[u8; 96 * 4]>(&NORMALS) }
}

#[rustfmt::skip]
const INDICES: [u8; 36] = [
    0, 1, 2,  0, 2, 3, // front
    4, 5, 6,  4, 6, 7, // up
    8,10, 9,  8,11,10, // back
   12,14,13, 12,15,14, // bottom
   16,17,18, 16,18,19, // left
   20,21,22, 20,22,23, // right
];
#[inline]
fn get_indices_buffer() -> &'static [u8] {
    &INDICES
}
