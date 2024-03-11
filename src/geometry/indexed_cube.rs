use std::{any::Any, cell::OnceCell};

use crate::{
    bounding::BoundingVolume,
    clock::Tick,
    renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            self, Buffer, BufferComponentSize, BufferDataType, BufferSource, BufferData,
            BufferUsage, MemoryPolicy,
        },
        draw::{CullFace, Draw, DrawMode, ElementIndicesDataType},
        uniform::{UniformBlockValue, UniformValue},
    },
    value::{Readonly, Value},
};

use super::{cube::build_bounding_volume, Geometry};

pub struct IndexedCube {
    size: f64,
    indices: Value<'static, Buffer>,
    positions: Value<'static, Buffer>,
    normals_and_textures: Value<'static, Buffer>,

    bounding_volume: BoundingVolume,
}

impl IndexedCube {
    /// Constructs a cube using elemental index with size `1.0`.
    pub fn new() -> IndexedCube {
        Self::with_size(1.0)
    }

    /// Constructs a cube using elemental index with specified size.
    pub fn with_size(size: f64) -> IndexedCube {
        let positions = if size == 1.0 {
            positions_size_one_buffer()
        } else {
            let buffer = buffer::Builder::default()
                .buffer_data(PositionsBufferSource(size))
                .set_memory_policy(MemoryPolicy::restorable(PositionsBufferSource(size)))
                .build();
            Value::Owned(buffer)
        };

        let normals_and_textures = normals_texture_coordinates_buffer_descriptor();

        Self {
            size,
            indices: indices_buffer_descriptor(),
            positions,
            normals_and_textures,
            bounding_volume: build_bounding_volume(size),
        }
    }
}

impl IndexedCube {
    /// Gets cube size.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// Sets cube size.
    pub fn set_size(&mut self, size: f64) {
        if BufferUsage::STATIC_DRAW == self.positions.value().usage() {
            self.positions
                .value_mut()
                .clear(false, Some(BufferUsage::DYNAMIC_DRAW));
        }
        self.positions
            .value_mut()
            .buffer_sub_data(PositionsBufferSource(size), 0);
        self.positions
            .value_mut()
            .set_memory_policy(MemoryPolicy::restorable(PositionsBufferSource(size)));

        self.size = size;
        self.bounding_volume = build_bounding_volume(size);
    }
}

impl Geometry for IndexedCube {
    fn draw(&self) -> Draw {
        Draw::Elements {
            mode: DrawMode::TRIANGLES,
            count: 36,
            offset: 0,
            indices: self.indices.value(),
            indices_data_type: ElementIndicesDataType::UNSIGNED_BYTE,
        }
    }

    fn cull_face(&self) -> Option<CullFace> {
        Some(CullFace::BACK)
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>> {
        Some(Readonly::Borrowed(&self.bounding_volume))
    }

    fn positions(&self) -> AttributeValue<'_> {
        AttributeValue::ArrayBuffer {
            buffer: self.positions.value(),
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            byte_offset: 0,
        }
    }

    fn normals(&self) -> Option<AttributeValue<'_>> {
        Some(AttributeValue::ArrayBuffer {
            buffer: self.normals_and_textures.value(),
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            byte_offset: 0,
        })
    }

    fn tangents(&self) -> Option<AttributeValue<'_>> {
        None
    }

    fn bitangents(&self) -> Option<AttributeValue<'_>> {
        None
    }

    fn texture_coordinates(&self) -> Option<AttributeValue<'_>> {
        Some(AttributeValue::ArrayBuffer {
            buffer: self.normals_and_textures.value(),
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            byte_offset: 72 * 4,
        })
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue<'_>> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<Readonly<'_, UniformValue>> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        None
    }

    fn tick(&mut self, _: &Tick) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[rustfmt::skip]
fn build_positions(size: f64) -> [u8; 72 * 4] {
    let s = (size / 2.0) as f32;
    let positions = [
        s, s, s,  -s, s, s,  -s,-s, s,   s,-s, s,  // v0-v1-v2-v3 front
        s, s, s,   s,-s, s,   s,-s,-s,   s, s,-s,  // v0-v3-v4-v5 right
        s, s, s,   s, s,-s,  -s, s,-s,  -s, s, s,  // v0-v5-v6-v1 top
       -s, s, s,  -s, s,-s,  -s,-s,-s,  -s,-s, s,  // v1-v6-v7-v2 left
       -s,-s,-s,   s,-s,-s,   s,-s, s,  -s,-s, s,  // v7-v4-v3-v2 bottom
        s,-s,-s,  -s,-s,-s,  -s, s,-s,   s, s,-s,  // v4-v7-v6-v5 back
    ];
    unsafe {
        std::mem::transmute::<[f32; 72], [u8; 72 * 4]>(positions)
    }
}

/// Positions buffer cache for cube with size 1, for debug purpose
static mut POSITIONS_SIZE_ONE_BUFFER: OnceCell<Buffer> = OnceCell::new();
fn positions_size_one_buffer() -> Value<'static, Buffer> {
    unsafe {
        let buffer = match POSITIONS_SIZE_ONE_BUFFER.get_mut() {
            Some(buffer) => buffer,
            None => {
                let buffer = buffer::Builder::default()
                    .buffer_data(PositionsBufferSource(1.0))
                    .set_memory_policy(MemoryPolicy::restorable(PositionsBufferSource(1.0)))
                    .build();
                POSITIONS_SIZE_ONE_BUFFER.set(buffer).unwrap();
                POSITIONS_SIZE_ONE_BUFFER.get_mut().unwrap()
            }
        };
        Value::Borrowed(buffer)
    }
}

#[rustfmt::skip]
const NORMALS_TEXTURE_COORDINATES: [f32; 120] = [
    // normals
     0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0, // front
     1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0, // right
     0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0, // top
    -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // left
     0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0, // bottom
     0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0, // back
     // textures
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // front
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // right
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // top
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // left
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // bottom
     1.0, 1.0,  0.0, 1.0,  0.0, 0.0,  1.0, 0.0, // back
];

static mut NORMALS_TEXTURE_COORDINATES_BUFFER: OnceCell<Buffer> = OnceCell::new();
fn normals_texture_coordinates_buffer_descriptor() -> Value<'static, Buffer> {
    unsafe {
        let buffer = match NORMALS_TEXTURE_COORDINATES_BUFFER.get_mut() {
            Some(buffer) => buffer,
            None => {
                let buffer = buffer::Builder::default()
                    .buffer_data(TexturesNormalsBufferSource)
                    .set_memory_policy(MemoryPolicy::restorable(TexturesNormalsBufferSource))
                    .build();
                NORMALS_TEXTURE_COORDINATES_BUFFER.set(buffer).unwrap();
                NORMALS_TEXTURE_COORDINATES_BUFFER.get_mut().unwrap()
            }
        };
        Value::Borrowed(buffer)
    }
}

#[rustfmt::skip]
const INDICES: [u8; 36] = [
    0,  1,  2,  0,  2,  3, // front
    4,  5,  6,  4,  6,  7, // up
    8,  9, 10,  8, 10, 11, // back
   12, 13, 14, 12, 14, 15, // bottom
   16, 17, 18, 16, 18, 19, // left
   20, 21, 22, 20, 22, 23, // right
];

static mut INDICES_BUFFER: OnceCell<Buffer> = OnceCell::new();
fn indices_buffer_descriptor() -> Value<'static, Buffer> {
    unsafe {
        let buffer = match INDICES_BUFFER.get_mut() {
            Some(buffer) => buffer,
            None => {
                let buffer = buffer::Builder::default()
                    .buffer_data(IndicesBufferSource)
                    .set_memory_policy(MemoryPolicy::restorable(IndicesBufferSource))
                    .build();
                INDICES_BUFFER.set(buffer).unwrap();
                INDICES_BUFFER.get_mut().unwrap()
            }
        };
        Value::Borrowed(buffer)
    }
}

#[derive(Debug)]
struct PositionsBufferSource(f64);

impl BufferSource for PositionsBufferSource {
    fn data(&self) -> BufferData<'_> {
        BufferData::Bytes {
            data: Box::new(build_positions(self.0)),
            src_element_offset: None,
            src_element_length: None,
        }
    }

    fn byte_length(&self) -> usize {
        72 * 4
    }
}

#[derive(Debug)]
struct IndicesBufferSource;

impl BufferSource for IndicesBufferSource {
    fn data(&self) -> BufferData<'_> {
        BufferData::BytesBorrowed {
            data: &INDICES,
            src_element_offset: None,
            src_element_length: None,
        }
    }
    
    fn byte_length(&self) -> usize {
        36
    }
}

#[derive(Debug)]
struct TexturesNormalsBufferSource;

impl TexturesNormalsBufferSource {
    fn as_bytes(&self) -> &'static [u8] {
        unsafe { std::mem::transmute::<&[f32; 120], &[u8; 120 * 4]>(&NORMALS_TEXTURE_COORDINATES) }
    }
}

impl BufferSource for TexturesNormalsBufferSource {
    fn data(&self) -> BufferData<'_> {
        BufferData::BytesBorrowed {
            data: self.as_bytes(),
            src_element_offset: None,
            src_element_length: None,
        }
    }

    fn byte_length(&self) -> usize {
        120 * 4
    }
}
