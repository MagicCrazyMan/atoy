use std::{any::Any, sync::OnceLock};

use wasm_bindgen::prelude::wasm_bindgen;

use crate::render::webgl::{
    buffer::{BufferData, BufferDescriptor, BufferStatus, BufferTarget, BufferUsage},
    draw::Draw,
    program::{AttributeValue, BufferDataType, UniformValue},
};

use super::Geometry;

#[wasm_bindgen]
pub struct Cube {
    size: f32,
    vertices: AttributeValue,
    normals: AttributeValue,
}

impl Cube {
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    pub fn with_size(size: f32) -> Cube {
        let vertices = get_vertices_buffer(size);
        let bytes_size = vertices.len() as i32;
        Self {
            size,
            vertices: AttributeValue::Buffer {
                descriptor: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                    id: None,
                    data: BufferData::FillData {
                        data: Box::new(vertices),
                        src_byte_offset: 0,
                        src_byte_length: 0,
                    },
                    usage: BufferUsage::StaticDraw,
                }),
                target: BufferTarget::Buffer,
                size: bytes_size,
                data_type: BufferDataType::Float,
                normalized: false,
                stride: 0,
                offset: 0,
            },
            normals: AttributeValue::Buffer {
                descriptor: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                    id: None,
                    data: BufferData::FillData {
                        data: Box::new(get_normals_buffer()),
                        src_byte_offset: 0,
                        src_byte_length: 0,
                    },
                    usage: BufferUsage::StaticDraw,
                }),
                target: BufferTarget::Buffer,
                size: bytes_size,
                data_type: BufferDataType::Float,
                normalized: false,
                stride: 0,
                offset: 0,
            },
        }
    }
}

#[wasm_bindgen]

impl Cube {
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
}

impl Geometry for Cube {
    fn vertices(&self) -> Option<&AttributeValue> {
        Some(&self.vertices)
    }

    fn normals(&self) -> Option<&AttributeValue> {
        todo!()
        //     Some(vec![
        //         0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  // front
        //         0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  // up
        //         0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  // back
        //         0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  // bottom
        //        -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0,  // left
        //         1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  // right
        //     ])
    }

    fn draw(&self) -> Draw {
        todo!()
    }

    fn attribute_value<'a>(&self, _name: &str) -> Option<&AttributeValue> {
        None
    }

    fn uniform_value<'a>(&self, _name: &str) -> Option<&UniformValue> {
        None
    }

    fn texture_coordinates<'a>(&'a self) -> Option<&AttributeValue> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn get_vertices_buffer(size: f32) -> Vec<u8> {
    let s = size / 2.0;
    [
        -s, s, s, -s, -s, s, s, s, s, s, s, s, -s, -s, s, s, -s, s, // front
        -s, s, -s, -s, s, s, s, s, -s, s, s, -s, -s, s, s, s, s, s, // up
        -s, s, -s, s, s, -s, -s, -s, -s, s, s, -s, s, -s, -s, -s, -s, -s, // back
        -s, -s, -s, s, -s, -s, -s, -s, s, s, -s, -s, s, -s, s, -s, -s, s, // bottom
        -s, s, -s, -s, -s, -s, -s, s, s, -s, s, s, -s, -s, -s, -s, -s, s, // left
        s, s, s, s, -s, s, s, s, -s, s, s, -s, s, -s, s, s, -s, -s, // right
    ]
    .iter()
    .flat_map(|v| v.to_ne_bytes())
    .collect::<Vec<_>>()
}

static NORMALS_BUFFER: OnceLock<Vec<u8>> = OnceLock::new();
fn get_normals_buffer() -> &'static Vec<u8> {
    NORMALS_BUFFER.get_or_init(|| {
        ([
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, // front
            0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // up
            0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0,
            0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, // back
            0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom
            -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0,
            -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, // right
        ] as [f32; 144])
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect::<Vec<_>>()
    })
}
