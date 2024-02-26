use std::{any::Any, cell::OnceCell};

use crate::{
    bounding::BoundingVolume, clock::Tick, readonly::Readonly, renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{CullFace, Draw, DrawMode, ElementIndicesDataType},
        uniform::{UniformBlockValue, UniformValue},
    }
};

use super::{cube::build_bounding_volume, Geometry};

pub struct IndexedCube {
    size: f64,
    indices: BufferDescriptor,
    positions: BufferDescriptor,

    positions_attribute: AttributeValue,
    normals_attribute: AttributeValue,
    textures_attribute: AttributeValue,
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
            buffer_descriptor_with_size_one()
        } else {
            BufferDescriptor::with_memory_policy(
                BufferSource::from_binary(build_positions(size), 0, 72 * 4),
                BufferUsage::STATIC_DRAW,
                MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 72 * 4)
                }),
            )
        };
        let positions_attribute = AttributeValue::Buffer {
            descriptor: positions.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        };

        let normals_and_textures = normals_texture_coordinates_buffer_descriptor();
        let normals_attribute = AttributeValue::Buffer {
            descriptor: normals_and_textures.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        };
        let textures_attribute = AttributeValue::Buffer {
            descriptor: normals_and_textures,
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 72 * 4,
        };

        Self {
            size,
            indices: indices_buffer_descriptor(),
            positions,
            normals_attribute,
            positions_attribute,
            textures_attribute,
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
        self.size = size;
        self.bounding_volume = build_bounding_volume(size);
        if let BufferUsage::STATIC_DRAW = self.positions.usage() {
            self.positions = BufferDescriptor::with_memory_policy(
                BufferSource::from_binary(build_positions(size), 0, 72 * 4),
                BufferUsage::DYNAMIC_DRAW,
                MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 72 * 4)
                }),
            );
            self.positions_attribute = AttributeValue::Buffer {
                descriptor: self.positions.clone(),
                target: BufferTarget::ARRAY_BUFFER,
                component_size: BufferComponentSize::Three,
                data_type: BufferDataType::FLOAT,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            }
        } else {
            self.positions.buffer_sub_data(
                BufferSource::from_binary(build_positions(size), 0, 72 * 4),
                0,
            );
            self.positions
                .set_memory_policy(MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 108 * 4)
                }));
        }
    }
}

impl Geometry for IndexedCube {
    fn draw(&self) -> Draw {
        Draw::Elements {
            mode: DrawMode::TRIANGLES,
            count: 36,
            offset: 0,
            indices: self.indices.clone(),
            indices_data_type: ElementIndicesDataType::UNSIGNED_BYTE,
        }
    }

    fn cull_face(&self) -> Option<CullFace> {
        Some(CullFace::BACK)
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>> {
        Some(Readonly::Borrowed(&self.bounding_volume))
    }

    fn positions(&self) -> Readonly<'_, AttributeValue> {
        Readonly::Borrowed(&self.positions_attribute)
    }

    fn normals(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.normals_attribute))
    }

    fn tangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn bitangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn texture_coordinates(&self) -> Option<Readonly<'_, AttributeValue>> {
        Some(Readonly::Borrowed(&self.textures_attribute))
    }

    fn attribute_value(&self, _: &str) -> Option<Readonly<'_, AttributeValue>> {
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

static mut NORMALS_TEXTURE_COORDINATES_BUFFER_DESCRIPTOR: OnceCell<BufferDescriptor> =
    OnceCell::new();

fn normals_texture_coordinates_buffer_descriptor() -> BufferDescriptor {
    unsafe {
        NORMALS_TEXTURE_COORDINATES_BUFFER_DESCRIPTOR
            .get_or_init(|| {
                BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(
                        std::mem::transmute::<&[f32; 120], &[u8; 120 * 4]>(
                            &NORMALS_TEXTURE_COORDINATES,
                        ),
                        0,
                        120 * 4,
                    ),
                    BufferUsage::STATIC_DRAW,
                    MemoryPolicy::restorable(|| {
                        BufferSource::from_binary(
                            std::mem::transmute::<&[f32; 120], &[u8; 120 * 4]>(
                                &NORMALS_TEXTURE_COORDINATES,
                            ),
                            0,
                            120 * 4,
                        )
                    }),
                )
            })
            .clone()
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

static mut INDICES_BUFFER_DESCRIPTOR: OnceCell<BufferDescriptor> = OnceCell::new();

fn indices_buffer_descriptor() -> BufferDescriptor {
    unsafe {
        INDICES_BUFFER_DESCRIPTOR
            .get_or_init(|| {
                BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(&INDICES, 0, 36),
                    BufferUsage::STATIC_DRAW,
                    MemoryPolicy::restorable(|| BufferSource::from_binary(&INDICES, 0, 36)),
                )
            })
            .clone()
    }
}

/// Positions buffer descriptor cache for cube with size 1, for debug purpose
static mut POSITIONS_BUFFER_DESCRIPTOR_SIZE_ONE: OnceCell<BufferDescriptor> = OnceCell::new();
fn buffer_descriptor_with_size_one() -> BufferDescriptor {
    unsafe {
        POSITIONS_BUFFER_DESCRIPTOR_SIZE_ONE
            .get_or_init(|| {
                BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(build_positions(1.0), 0, 72 * 4),
                    BufferUsage::STATIC_DRAW,
                    MemoryPolicy::restorable(move || {
                        BufferSource::from_binary(build_positions(1.0), 0, 72 * 4)
                    }),
                )
            })
            .clone()
    }
}
