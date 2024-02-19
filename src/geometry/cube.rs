use std::{any::Any, cell::OnceCell};

use gl_matrix4rust::vec3::Vec3;

use crate::{
    bounding::BoundingVolume,
    readonly::Readonly,
    renderer::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{CullFace, Draw, DrawMode},
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::Geometry;

pub struct Cube {
    size: f64,
    positions: BufferDescriptor,

    positions_attribute: AttributeValue,
    normals_attribute: AttributeValue,
    textures_attribute: AttributeValue,
    bounding_volume: BoundingVolume,
}

impl Cube {
    /// Constructs a cube with size `1.0`.
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    /// Constructs a cube with a specified size.
    pub fn with_size(size: f64) -> Cube {
        let positions = if size == 1.0 {
            buffer_descriptor_with_size_one()
        } else {
            BufferDescriptor::with_memory_policy(
                BufferSource::from_function(
                    move || BufferSource::from_binary(build_positions(size), 0, 108 * 4),
                    108 * 4,
                    0,
                    108 * 4,
                ),
                BufferUsage::STATIC_DRAW,
                MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 108 * 4)
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
            bytes_offset: 108 * 4,
        };

        Self {
            size,
            positions,
            positions_attribute,
            normals_attribute,
            textures_attribute,
            bounding_volume: build_bounding_volume(size),
        }
    }

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
                BufferSource::from_binary(build_positions(size), 0, 108 * 4),
                BufferUsage::DYNAMIC_DRAW,
                MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 108 * 4)
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
                BufferSource::from_binary(build_positions(size), 0, 108 * 4),
                0,
            );
            self.positions
                .set_memory_policy(MemoryPolicy::restorable(move || {
                    BufferSource::from_binary(build_positions(size), 0, 108 * 4)
                }));
        }
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

static mut NORMALS_TEXTURE_COORDINATES_BUFFER_DESCRIPTOR: OnceCell<BufferDescriptor> =
    OnceCell::new();

fn normals_texture_coordinates_buffer_descriptor() -> BufferDescriptor {
    unsafe {
        NORMALS_TEXTURE_COORDINATES_BUFFER_DESCRIPTOR
            .get_or_init(|| {
                BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(
                        std::mem::transmute::<&[f32; 108 + 48], &[u8; (108 + 48) * 4]>(
                            &NORMALS_TEXTURE_COORDINATES,
                        ),
                        0,
                        (108 + 48) * 4,
                    ),
                    BufferUsage::STATIC_DRAW,
                    MemoryPolicy::restorable(|| {
                        BufferSource::from_binary(
                            std::mem::transmute::<&[f32; 108 + 48], &[u8; (108 + 48) * 4]>(
                                &NORMALS_TEXTURE_COORDINATES,
                            ),
                            0,
                            (108 + 48) * 4,
                        )
                    }),
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
                    BufferSource::from_binary(build_positions(1.0), 0, 108 * 4),
                    BufferUsage::STATIC_DRAW,
                    MemoryPolicy::restorable(move || {
                        BufferSource::from_binary(build_positions(1.0), 0, 108 * 4)
                    }),
                )
            })
            .clone()
    }
}
