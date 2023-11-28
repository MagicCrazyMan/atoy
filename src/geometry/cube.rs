use std::any::Any;

use crate::{
    render::webgl::{
        attribute::AttributeValue,
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage,
        },
        draw::{Draw, DrawMode},
        uniform::UniformValue,
        RenderEntity,
    },
    utils::slice_to_float32_array,
};

use super::Geometry;

pub struct Cube {
    size: f64,
    vertices: BufferDescriptor,
    normals: BufferDescriptor,
    texture_coordinates: BufferDescriptor,
}

impl Cube {
    /// Constructs a cube with size `1.0`.
    pub fn new() -> Cube {
        Self::with_size(1.0)
    }

    /// Constructs a cube with a specified size.
    pub fn with_size(size: f64) -> Cube {
        let vertices = BufferDescriptor::new(
            BufferSource::from_float32_array(
                slice_to_float32_array(&calculate_vertices(size)),
                0,
                108,
            ),
            BufferUsage::StaticDraw,
        );
        // vertices.enable_restore(move || {
        //     BufferSource::from_float32_array(
        //         slice_to_float32_array(&calculate_vertices(size)),
        //         0,
        //         108,
        //     )
        // });

        Self {
            size,
            vertices,
            normals: BufferDescriptor::new(
                BufferSource::from_float32_array(slice_to_float32_array(&NORMALS), 0, 144),
                BufferUsage::StaticDraw,
            ),
            texture_coordinates: BufferDescriptor::new(
                BufferSource::from_float32_array(
                    slice_to_float32_array(&TEXTURE_COORDINATES),
                    0,
                    48,
                ),
                BufferUsage::StaticDraw,
            ),
        }
    }
}

impl Cube {
    /// Gets cube size.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// Sets cube size.
    pub fn set_size(&mut self, size: f64) {
        self.size = size;

        self.vertices
            .buffer_sub_data(BufferSource::from_float32_array(
                slice_to_float32_array(&calculate_vertices(size)),
                0,
                108,
            ));
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
        Some(AttributeValue::Buffer {
            descriptor: self.normals.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Four,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
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

    fn attribute_value(&self, _: &str, _: &RenderEntity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &RenderEntity) -> Option<UniformValue> {
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
pub fn calculate_vertices(size: f64) -> [f32; 108] {
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
pub const NORMALS: [f32; 144] = [
     0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0, // front
     0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0, // up
     0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0, // back
     0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0, // bottom
    -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, // left
     1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0, // right
];

#[rustfmt::skip]
pub const TEXTURE_COORDINATES: [f32; 48] = [
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // front
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // up
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // back
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // bottom
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // left
    1.5, 1.5,  -0.5, 1.5,  -0.5, -0.5,  1.5, -0.5, // right
];
