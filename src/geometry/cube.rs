use std::any::Any;

use gl_matrix4rust::vec3::Vec3;
use web_sys::js_sys::{ArrayBuffer, Float32Array};

use crate::{
    bounding::BoundingVolumeNative,
    entity::BorrowedMut,
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{Draw, DrawMode},
        uniform::UniformValue,
    },
};

use super::Geometry;

pub struct Cube {
    size: f64,
    data: BufferDescriptor,
    bounding_volume: BoundingVolumeNative,
    update_bounding_volume: bool,
}

impl Cube {
    /// Constructs a cube with size `1.0`.
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    /// Constructs a cube with a specified size.
    pub fn with_size(size: f64) -> Cube {
        Self {
            size,
            data: BufferDescriptor::with_memory_policy(
                BufferSource::from_array_buffer(build_data(size)),
                BufferUsage::StaticDraw,
                MemoryPolicy::new_restorable(move || {
                    BufferSource::from_array_buffer(build_data(size))
                }),
            ),
            bounding_volume: build_bounding_volume(size),
            update_bounding_volume: true,
        }
    }

    /// Gets cube size.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// Sets cube size.
    pub fn set_size(&mut self, size: f64) {
        self.size = size;
        self.data.buffer_sub_data(
            BufferSource::from_binary(
                unsafe { std::mem::transmute::<[f32; 108], [u8; 432]>(build_vertices(size)) },
                0,
                432,
            ),
            0,
        );
        self.data
            .set_memory_policy(MemoryPolicy::new_restorable(move || {
                BufferSource::from_array_buffer(build_data(size))
            }));
        self.bounding_volume = build_bounding_volume(size);
        self.update_bounding_volume = true;
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

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative> {
        Some(self.bounding_volume)
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.data.clone(),
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
            descriptor: self.data.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 108 * 4,
        })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.data.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 216 * 4,
        })
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Clone for Cube {
    fn clone(&self) -> Self {
        Self::with_size(self.size)
    }
}

fn build_data(size: f64) -> ArrayBuffer {
    let data = ArrayBuffer::new(264 * 4);
    let v = Float32Array::new_with_byte_offset_and_length(&data, 0, 108);
    v.copy_from(&build_vertices(size));
    let n = Float32Array::new_with_byte_offset_and_length(&data, 108 * 4, 108);
    n.copy_from(&NORMALS);
    let t = Float32Array::new_with_byte_offset_and_length(&data, 216 * 4, 48);
    t.copy_from(&TEXTURE_COORDINATES);
    data
}

fn build_bounding_volume(size: f64) -> BoundingVolumeNative {
    let s = size / 2.0;
    BoundingVolumeNative::BoundingSphere {
        center: Vec3::from_values(0.0, 0.0, 0.0),
        radius: (s * s + s * s + s * s).sqrt(),
    }
}

#[rustfmt::skip]
fn build_vertices(size: f64) -> [f32; 108] {
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
const NORMALS: [f32; 108] = [
     0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0,  0.0, 0.0, 1.0, // front
     0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, 0.0, // up
     0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0,  0.0, 0.0,-1.0, // back
     0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0,  0.0,-1.0, 0.0, // bottom
    -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, // left
     1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0,  1.0, 0.0, 0.0, // right
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
