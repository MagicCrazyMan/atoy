use std::any::Any;

use crate::{
    bounding::BoundingVolumeKind,
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        draw::{Draw, DrawMode},
        uniform::UniformValue,
    },
    utils::slice_to_float32_array,
};

use super::{Geometry, GeometryRenderEntity};

pub struct Plane {
    vertices: BufferDescriptor,
    texture_coordinates: BufferDescriptor,
}

impl Plane {
    pub fn new() -> Plane {
        Self {
            vertices: BufferDescriptor::new(
                BufferSource::from_float32_array(slice_to_float32_array(&VERTICES), 0, 18 * 4),
                BufferUsage::StaticDraw,
            ),
            texture_coordinates: BufferDescriptor::new(
                BufferSource::from_float32_array(
                    slice_to_float32_array(&TEXTURE_COORDINATES),
                    0,
                    12 * 4,
                ),
                BufferUsage::StaticDraw,
            ),
        }
    }
}

impl Geometry for Plane {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: 36,
        }
    }

    fn bounding_volume(&self) -> Option<BoundingVolumeKind> {
        None
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.vertices.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn normals(&self) -> Option<AttributeValue> {
        None
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.texture_coordinates.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn attribute_value(&self, _: &str, _: &GeometryRenderEntity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &GeometryRenderEntity) -> Option<UniformValue> {
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
const VERTICES: [f32; 18] = [
     0.5, 0.5, 0.0,  -0.5, 0.5, 0.0,  -0.5,-0.5, 0.0,
    -0.5,-0.5, 0.0,   0.5,-0.5, 0.0,   0.5, 0.5, 0.0,
];

#[rustfmt::skip]
const TEXTURE_COORDINATES: [f32; 12] = [
    1.0,1.0,  0.0,1.0,  0.0,0.0,
    0.0,0.0,  1.0,0.0,  1.0,1.0,
];
