use std::{any::Any, cell::OnceCell};

use gl_matrix4rust::vec3::Vec3;

use crate::{
    bounding::BoundingVolume,
    clock::Tick,
    renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            self, Buffer, BufferComponentSize, BufferDataType, BufferSource, BufferSourceData,
            BufferUsage, MemoryPolicy,
        },
        draw::{CullFace, Draw, DrawMode},
        uniform::{UniformBlockValue, UniformValue},
    },
    value::{Readonly, Value},
};

use super::Geometry;

pub struct Cube {
    size: f64,
    positions_shared: bool,
    positions: Value<'static, Buffer>,
    normals_and_textures: Value<'static, Buffer>,
    bounding_volume: BoundingVolume,
}

impl Cube {
    /// Constructs a cube with size `1.0`.
    pub fn new() -> Self {
        Self::with_size(1.0)
    }

    /// Constructs a cube with a specified size.
    pub fn with_size(size: f64) -> Self {
        let (positions, positions_shared) = if size == 1.0 {
            (positions_size_one_buffer(), true)
        } else {
            let buffer = buffer::Builder::default()
                .buffer_data(PositionsBufferSource(size))
                .set_memory_policy(MemoryPolicy::restorable(PositionsBufferSource(size)))
                .build();
            (Value::Owned(buffer), false)
        };
        let normals_and_textures = normals_texture_coordinates_buffer();

        Self {
            size,
            positions,
            positions_shared,
            normals_and_textures,
            bounding_volume: build_bounding_volume(size),
        }
    }

    /// Gets cube size.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// Sets cube size.
    pub fn set_size(&mut self, size: f64) {
        if self.positions_shared {
            self.positions = Value::Owned(
                buffer::Builder::new(BufferUsage::DYNAMIC_DRAW)
                    .buffer_data(PositionsBufferSource(size))
                    .set_memory_policy(MemoryPolicy::restorable(PositionsBufferSource(size)))
                    .build(),
            );
            self.positions_shared = false;
        } else {
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
        }

        self.size = size;
        self.bounding_volume = build_bounding_volume(size);
    }
}

impl Geometry for Cube {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::TRIANGLES,
            first: 0,
            count: 36,
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
            byte_offset: 108 * 4,
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

pub(super) fn build_bounding_volume(size: f64) -> BoundingVolume {
    let s = size / 2.0;
    BoundingVolume::BoundingSphere {
        center: Vec3::<f64>::new_zero(),
        radius: (s * s + s * s + s * s).sqrt(),
    }
}

#[rustfmt::skip]
fn build_positions(size: f64) -> [u8; 108 * 4] {
    let s = (size / 2.0) as f32;
    let positions = [
        -s,  s,  s,  -s, -s,  s,   s,  s,  s,   s,  s,  s,  -s, -s,  s,   s, -s,  s, // front
        -s,  s, -s,  -s,  s,  s,   s,  s, -s,   s,  s, -s,  -s,  s,  s,   s,  s,  s, // up
        -s,  s, -s,   s,  s, -s,  -s, -s, -s,   s,  s, -s,   s, -s, -s,  -s, -s, -s, // back
        -s, -s, -s,   s, -s, -s,  -s, -s,  s,   s, -s, -s,   s, -s,  s,  -s, -s,  s, // bottom
        -s,  s, -s,  -s, -s, -s,  -s,  s,  s,  -s,  s,  s,  -s, -s, -s,  -s, -s,  s, // left
         s,  s,  s,   s, -s,  s,   s,  s, -s,   s,  s, -s,   s, -s,  s,   s, -s, -s, // right
    ];
    unsafe {
        std::mem::transmute::<[f32; 108], [u8; 108 * 4]>(positions)
    }
}

#[rustfmt::skip]
const NORMALS_TEXTURE_COORDINATES: [f32; 108 + 48] = [
     0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0, // front
     0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0, // up
     0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0, // back
     0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0, // bottom
    -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // left
     1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0, // right

    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // front
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // up
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // back
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // bottom
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // left
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // right
];

static mut NORMALS_TEXTURE_COORDINATES_BUFFER: OnceCell<Buffer> = OnceCell::new();
fn normals_texture_coordinates_buffer() -> Value<'static, Buffer> {
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

#[derive(Debug)]
struct PositionsBufferSource(f64);

impl BufferSource for PositionsBufferSource {
    fn data(&self) -> BufferSourceData<'_> {
        BufferSourceData::Bytes(build_positions(self.0).to_vec())
    }

    fn byte_length(&self) -> usize {
        108 * 4
    }

    fn src_element_offset(&self) -> Option<usize> {
        None
    }

    fn src_element_length(&self) -> Option<usize> {
        None
    }
}

#[derive(Debug)]
struct TexturesNormalsBufferSource;

impl TexturesNormalsBufferSource {
    fn as_bytes(&self) -> &'static [u8] {
        unsafe {
            std::mem::transmute::<&[f32; 108 + 48], &[u8; (108 + 48) * 4]>(
                &NORMALS_TEXTURE_COORDINATES,
            )
        }
    }
}

impl BufferSource for TexturesNormalsBufferSource {
    fn data(&self) -> BufferSourceData<'_> {
        BufferSourceData::BytesBorrowed(self.as_bytes())
    }

    fn src_element_offset(&self) -> Option<usize> {
        None
    }

    fn src_element_length(&self) -> Option<usize> {
        None
    }
}
