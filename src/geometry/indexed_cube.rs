use std::any::Any;

use gl_matrix4rust::vec3::Vec3;

use crate::{
    bounding::BoundingVolume,
    notify::Notifier,
    readonly::Readonly,
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        draw::{CullFace, Draw, DrawElementType, DrawMode},
        uniform::{UniformBlockValue, UniformValue},
    },
    utils::{slice_to_float32_array, slice_to_uint8_array},
};

use super::Geometry;

pub struct IndexedCube {
    size: f64,
    indices: BufferDescriptor,
    positions: BufferDescriptor,
    normals: BufferDescriptor,
    texture_coordinates: BufferDescriptor,
    notifier: Notifier<()>,
}

impl IndexedCube {
    /// Constructs a cube using elemental index with size `1.0`.
    pub fn new() -> IndexedCube {
        Self::with_size(1.0)
    }

    /// Constructs a cube using elemental index with specified size.
    pub fn with_size(size: f64) -> IndexedCube {
        Self {
            size,
            indices: BufferDescriptor::new(
                BufferSource::from_uint8_array(slice_to_uint8_array(&INDICES), 0, 36),
                BufferUsage::STATIC_DRAW,
            ),
            positions: BufferDescriptor::new(
                BufferSource::from_float32_array(
                    slice_to_float32_array(&build_positions(size)),
                    0,
                    72,
                ),
                BufferUsage::STATIC_DRAW,
            ),
            normals: BufferDescriptor::new(
                BufferSource::from_float32_array(slice_to_float32_array(&NORMALS), 0, 96),
                BufferUsage::STATIC_DRAW,
            ),
            texture_coordinates: BufferDescriptor::new(
                BufferSource::from_float32_array(
                    slice_to_float32_array(&TEXTURE_COORDINATES),
                    0,
                    48,
                ),
                BufferUsage::STATIC_DRAW,
            ),
            notifier: Notifier::new(),
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
        self.positions.buffer_sub_data(
            BufferSource::from_float32_array(slice_to_float32_array(&build_positions(size)), 0, 72),
            0,
        );
    }
}

impl Geometry for IndexedCube {
    fn draw(&self) -> Draw {
        Draw::Elements {
            mode: DrawMode::TRIANGLES,
            count: 36,
            element_type: DrawElementType::UNSIGNED_BYTE,
            offset: 0,
            indices: self.indices.clone(),
        }
    }

    fn cull_face(&self) -> Option<CullFace> {
        Some(CullFace::BACK)
    }

    fn bounding_volume(&self) -> Option<BoundingVolume> {
        let s = self.size / 2.0;
        Some(BoundingVolume::BoundingSphere {
            center: Vec3::<f64>::new(0.0, 0.0, 0.0),
            radius: (s * s + s * s + s * s).sqrt(),
        })
    }

    fn positions(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.positions.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn normals(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.normals.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Four,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.texture_coordinates.clone(),
            target: BufferTarget::ARRAY_BUFFER,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::FLOAT,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<Readonly<'_, UniformValue>> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue> {
        None
    }

    fn notifier(&mut self) -> &mut Notifier<()> {
        &mut self.notifier
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[rustfmt::skip]
fn build_positions(size: f64) -> [f32; 72] {
    let s = (size / 2.0) as f32;
    [
        s, s, s,  -s, s, s,  -s,-s, s,   s,-s, s,  // v0-v1-v2-v3 front
        s, s, s,   s,-s, s,   s,-s,-s,   s, s,-s,  // v0-v3-v4-v5 right
        s, s, s,   s, s,-s,  -s, s,-s,  -s, s, s,  // v0-v5-v6-v1 top
       -s, s, s,  -s, s,-s,  -s,-s,-s,  -s,-s, s,  // v1-v6-v7-v2 left
       -s,-s,-s,   s,-s,-s,   s,-s, s,  -s,-s, s,  // v7-v4-v3-v2 bottom
        s,-s,-s,  -s,-s,-s,  -s, s,-s,   s, s,-s,  // v4-v7-v6-v5 back
    ]
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

#[rustfmt::skip]
const NORMALS: [f32; 96] = [
     0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0, // front
     1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0, // right
     0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0, // top
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
     0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0, // bottom
     0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0, // back
];

#[rustfmt::skip]
const INDICES: [u8; 36] = [
    0,  1,  2,  0,  2,  3, // front
    4,  5,  6,  4,  6,  7, // up
    8,  9, 10,  8, 10, 11, // back
   12, 13, 14, 12, 14, 15, // bottom
   16, 17, 18, 16, 18, 19, // left
   20, 21, 22, 20, 22, 23, // right
];
