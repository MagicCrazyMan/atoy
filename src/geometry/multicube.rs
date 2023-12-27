use std::any::Any;

use gl_matrix4rust::{
    mat4::Mat4,
    vec3::{AsVec3, Vec3},
};
use web_sys::js_sys::Float32Array;

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

pub struct MultiCube {
    size: f64,
    count: usize,
    vertices: BufferDescriptor,
    // normals: BufferDescriptor,
    // texture_coordinates: BufferDescriptor,
    // non-clone fields
    update_bounding_volume: bool,
}

impl MultiCube {
    /// Constructs a cube with size `1.0`.
    pub fn new(count: usize) -> MultiCube {
        Self::with_size(count, 1.0)
    }

    /// Constructs a cube with a specified size.
    pub fn with_size(count: usize, size: f64) -> MultiCube {
        let vertices = BufferDescriptor::with_memory_policy(
            BufferSource::from_float32_array(calculate_vertices(count, size), 0, 0),
            BufferUsage::StaticDraw,
            MemoryPolicy::restorable(move || {
                BufferSource::from_float32_array(calculate_vertices(count, size), 0, 0)
            }),
        );

        Self {
            size,
            count,
            vertices,
            // normals: BufferDescriptor::with_memory_policy(
            //     BufferSource::from_float32_array(slice_to_float32_array(&NORMALS), 0, 144),
            //     BufferUsage::StaticDraw,
            //     MemoryPolicy::restorable(|| {
            //         BufferSource::from_float32_array(slice_to_float32_array(&NORMALS), 0, 144)
            //     }),
            // ),
            // texture_coordinates: BufferDescriptor::with_memory_policy(
            //     BufferSource::from_float32_array(
            //         slice_to_float32_array(&TEXTURE_COORDINATES),
            //         0,
            //         48,
            //     ),
            //     BufferUsage::StaticDraw,
            //     MemoryPolicy::restorable(|| {
            //         BufferSource::from_float32_array(
            //             slice_to_float32_array(&TEXTURE_COORDINATES),
            //             0,
            //             144,
            //         )
            //     }),
            // ),
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
        self.vertices.buffer_sub_data(
            BufferSource::from_float32_array(calculate_vertices(self.count, size), 0, 108),
            0,
        );
        self.update_bounding_volume = true;
    }
}

impl Geometry for MultiCube {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: (self.count * 36) as i32,
        }
    }

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative> {
        let s = self.size / 2.0;
        Some(BoundingVolumeNative::BoundingSphere {
            center: Vec3::from_values(0.0, 0.0, 0.0),
            radius: (s * s + s * s + s * s).sqrt(),
        })
        // Some(BoundingVolumeNative::AxisAlignedBoundingBox {
        //     min_x: -s,
        //     min_y: -s,
        //     min_z: -s,
        //     max_x: s,
        //     max_y: s,
        //     max_z: s,
        // })
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
        // Some(AttributeValue::Buffer {
        //     descriptor: self.normals.clone(),
        //     target: BufferTarget::ArrayBuffer,
        //     component_size: BufferComponentSize::Four,
        //     data_type: BufferDataType::Float,
        //     normalized: false,
        //     bytes_stride: 0,
        //     bytes_offset: 0,
        // })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        None
        // Some(AttributeValue::Buffer {
        //     descriptor: self.texture_coordinates.clone(),
        //     target: BufferTarget::ArrayBuffer,
        //     component_size: BufferComponentSize::Two,
        //     data_type: BufferDataType::Float,
        //     normalized: false,
        //     bytes_stride: 0,
        //     bytes_offset: 0,
        // })
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

impl Clone for MultiCube {
    fn clone(&self) -> Self {
        Self {
            size: self.size.clone(),
            count: self.count.clone(),
            vertices: self.vertices.clone(),
            // normals: self.normals.clone(),
            // texture_coordinates: self.texture_coordinates.clone(),
            update_bounding_volume: true,
        }
    }
}

pub fn calculate_vertices(count: usize, size: f64) -> Float32Array {
    let s = (size / 2.0) as f32;
    #[rustfmt::skip]
    let vertices = [
        Vec3::from_values(-s,  s,  s),  Vec3::from_values(-s, -s,  s),  Vec3::from_values( s,  s,  s),  Vec3::from_values( s,  s,  s), Vec3::from_values( -s, -s,  s),  Vec3::from_values( s, -s,  s), // front
        Vec3::from_values(-s,  s, -s),  Vec3::from_values(-s,  s,  s),  Vec3::from_values( s,  s, -s),  Vec3::from_values( s,  s, -s), Vec3::from_values( -s,  s,  s),  Vec3::from_values( s,  s,  s), // up
        Vec3::from_values(-s,  s, -s),  Vec3::from_values( s,  s, -s),  Vec3::from_values(-s, -s, -s),  Vec3::from_values( s,  s, -s), Vec3::from_values(  s, -s, -s),  Vec3::from_values(-s, -s, -s), // back
        Vec3::from_values(-s, -s, -s),  Vec3::from_values( s, -s, -s),  Vec3::from_values(-s, -s,  s),  Vec3::from_values( s, -s, -s), Vec3::from_values(  s, -s,  s),  Vec3::from_values(-s, -s,  s), // bottom
        Vec3::from_values(-s,  s, -s),  Vec3::from_values(-s, -s, -s),  Vec3::from_values(-s,  s,  s),  Vec3::from_values(-s,  s,  s), Vec3::from_values( -s, -s, -s),  Vec3::from_values(-s, -s,  s), // left
        Vec3::from_values( s,  s,  s),  Vec3::from_values( s, -s,  s),  Vec3::from_values( s,  s, -s),  Vec3::from_values( s,  s, -s), Vec3::from_values(  s, -s,  s),  Vec3::from_values( s, -s, -s), // right
    ];

    let multiples = Float32Array::new_with_length((count * vertices.len() * 3) as u32);

    let width = 500.0;
    let height = 500.0;
    let grid = 200;
    let cell_width = width / (grid as f32);
    let cell_height = height / (grid as f32);
    let start_x = width / 2.0 - cell_width / 2.0;
    let start_z = height / 2.0 - cell_height / 2.0;
    for index in 0..count {
        let row = index / grid;
        let col = index % grid;
        let center_x = start_x - col as f32 * cell_width;
        let center_z = start_z - row as f32 * cell_height;

        let matrix = Mat4::from_translation(&(center_x, 0.0, center_z));
        for (i, p) in vertices.iter().enumerate() {
            let p = p.transform_mat4(&matrix);
            multiples.set_index((index * vertices.len() + i * 3 + 0) as u32, p.0[0]);
            multiples.set_index((index * vertices.len() + i * 3 + 1) as u32, p.0[1]);
            multiples.set_index((index * vertices.len() + i * 3 + 2) as u32, p.0[2]);
        }
    }

    multiples
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
