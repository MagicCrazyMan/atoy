use std::{any::Any, cell::OnceCell, collections::HashMap};

use gl_matrix4rust::vec3::Vec3;
use ordered_float::OrderedFloat;

use crate::{
    bounding::BoundingVolume,
    event::EventAgency,
    render::webgl::{
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
    vertices: BufferDescriptor,
    normals_textures: BufferDescriptor,
    bounding_volume: BoundingVolume,
    changed_event: EventAgency<()>,
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
            vertices: get_or_cache_vertices(size),
            normals_textures: normals_texture_coordinates_buffer_descriptor(),
            bounding_volume: build_bounding_volume(size),
            changed_event: EventAgency::new(),
        }
    }

    /// Gets cube size.
    pub fn size(&self) -> f64 {
        self.size
    }

    /// Sets cube size.
    pub fn set_size(&mut self, size: f64) {
        self.size = size;
        self.vertices = get_or_cache_vertices(size);
        self.bounding_volume = build_bounding_volume(size);
        self.changed_event.raise(());
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

    fn cull_face(&self) -> Option<CullFace> {
        Some(CullFace::Back)
    }

    fn bounding_volume(&self) -> Option<BoundingVolume> {
        Some(self.bounding_volume)
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
            descriptor: self.normals_textures.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        })
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(AttributeValue::Buffer {
            descriptor: self.normals_textures.clone(),
            target: BufferTarget::ArrayBuffer,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 108 * 4,
        })
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<UniformValue> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue> {
        None
    }

    fn changed_event(&self) -> &EventAgency<()> {
        &self.changed_event
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn build_bounding_volume(size: f64) -> BoundingVolume {
    let s = size / 2.0;
    BoundingVolume::BoundingSphere {
        center: Vec3::from_values(0.0, 0.0, 0.0),
        radius: (s * s + s * s + s * s).sqrt(),
    }
}

#[rustfmt::skip]
fn build_vertices(size: f64) -> [u8; 108 * 4] {
    let s = (size / 2.0) as f32;
    let vertices = [
        -s,  s,  s,  -s, -s,  s,   s,  s,  s,   s,  s,  s,  -s, -s,  s,   s, -s,  s, // front
        -s,  s, -s,  -s,  s,  s,   s,  s, -s,   s,  s, -s,  -s,  s,  s,   s,  s,  s, // up
        -s,  s, -s,   s,  s, -s,  -s, -s, -s,   s,  s, -s,   s, -s, -s,  -s, -s, -s, // back
        -s, -s, -s,   s, -s, -s,  -s, -s,  s,   s, -s, -s,   s, -s,  s,  -s, -s,  s, // bottom
        -s,  s, -s,  -s, -s, -s,  -s,  s,  s,  -s,  s,  s,  -s, -s, -s,  -s, -s,  s, // left
         s,  s,  s,   s, -s,  s,   s,  s, -s,   s,  s, -s,   s, -s,  s,   s, -s, -s, // right
    ];
    unsafe {
        std::mem::transmute::<[f32; 108], [u8; 108 * 4]>(vertices)
    }
}

#[rustfmt::skip]
static NORMALS_TEXTURE_COORDINATES: [f32; 108 + 48] = [
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
                        std::mem::transmute_copy::<[f32; 108 + 48], [u8; (108 + 48) * 4]>(
                            &NORMALS_TEXTURE_COORDINATES,
                        ),
                        0,
                        (108 + 48) * 4,
                    ),
                    BufferUsage::StaticDraw,
                    MemoryPolicy::restorable(|| {
                        BufferSource::from_binary(
                            std::mem::transmute_copy::<[f32; 108 + 48], [u8; (108 + 48) * 4]>(
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

static mut VERTICES_BUFFER_DESCRIPTOR_CACHES: OnceCell<
    HashMap<OrderedFloat<f64>, BufferDescriptor>,
> = OnceCell::new();

fn get_or_cache_vertices(size: f64) -> BufferDescriptor {
    unsafe {
        let caches = if VERTICES_BUFFER_DESCRIPTOR_CACHES.get().is_some() {
            VERTICES_BUFFER_DESCRIPTOR_CACHES.get_mut().unwrap()
        } else {
            let _ = VERTICES_BUFFER_DESCRIPTOR_CACHES.set(HashMap::new());
            VERTICES_BUFFER_DESCRIPTOR_CACHES.get_mut().unwrap()
        };

        match caches.entry(OrderedFloat(size)) {
            std::collections::hash_map::Entry::Occupied(v) => v.get().clone(),
            std::collections::hash_map::Entry::Vacant(v) => {
                let descriptor = BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(build_vertices(size), 0, 108 * 4),
                    BufferUsage::StaticDraw,
                    MemoryPolicy::restorable(move || {
                        BufferSource::from_binary(build_vertices(size), 0, 108 * 4)
                    }),
                );
                let cache = v.insert(descriptor);
                cache.clone()
            }
        }
    }
}
