use std::any::Any;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    ncor::Ncor,
    render::webgl::{
        buffer::{
            BufferData, BufferDescriptor, BufferItemSize, BufferStatus, BufferTarget, BufferUsage,
        },
        draw::{Draw, DrawMode},
        program::{AttributeValue, BufferDataType, UniformValue},
    },
};

use super::Geometry;

#[wasm_bindgen]
pub struct Cube {
    size: f32,
    vertices_buffer: BufferDescriptor,
    normals_buffer: BufferDescriptor,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(size: Option<f32>) -> Self {
        Self::with_size(size.unwrap_or(1.0))
    }
}

impl Cube {
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    pub fn with_size(size: f32) -> Cube {
        Self {
            size,
            vertices_buffer: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::FillData {
                    data: Box::new(get_vertices_buffer(size)),
                    src_byte_offset: 0,
                    src_byte_length: 0,
                },
                usage: BufferUsage::StaticDraw,
            }),
            normals_buffer: BufferDescriptor::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::FillData {
                    data: Box::new(get_normals_buffer()),
                    src_byte_offset: 0,
                    src_byte_length: 0,
                },
                usage: BufferUsage::StaticDraw,
            }),
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
        // self.vertices = AttributeValue::Buffer {
        //     descriptor: BufferDescriptor::new(BufferStatus::UpdateBuffer {
        //         id: self.,
        //         data: (),
        //         usage: (),
        //     }),
        //     target: BufferTarget::Buffer,
        //     size: BufferItemSize::Three,
        //     data_type: BufferDataType::Float,
        //     normalized: false,
        //     stride: 0,
        //     offset: 0,
        // };
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

    fn vertices<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.vertices_buffer),
            target: BufferTarget::Buffer,
            size: BufferItemSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            stride: 0,
            offset: 0,
        }))
    }

    fn normals<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.normals_buffer),
            target: BufferTarget::Buffer,
            size: BufferItemSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            stride: 0,
            offset: 0,
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

const NORMALS: [f32; 144] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
    0.0, 0.0, 0.0, 1.0, 0.0, // front
    0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 1.0, 0.0, 0.0, // up
    0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0,
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, // back
    0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0,
    0.0, 0.0, 0.0, -1.0, 0.0, 0.0, // bottom
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0,
    0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0, 0.0, // right
];
#[inline]
fn get_normals_buffer() -> &'static [u8] {
    unsafe { std::mem::transmute::<&[f32; 144], &[u8; 576]>(&NORMALS) }
}
