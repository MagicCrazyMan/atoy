use std::any::Any;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    ncor::Ncor,
    render::webgl::{
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage,
        },
        draw::{Draw, DrawMode},
        program::{AttributeValue, UniformValue},
    },
};

use super::Geometry;

#[wasm_bindgen]
pub struct Plane {
    vertices_buffer: BufferDescriptor,
    texture_coordinates_buffer: BufferDescriptor,
}

#[wasm_bindgen]
impl Plane {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor() -> Self {
        Self::new()
    }
}

impl Plane {
    pub fn new() -> Plane {
        Self {
            vertices_buffer: BufferDescriptor::with_binary(
                get_vertices_buffer(),
                0,
                18 * 4,
                BufferUsage::StaticDraw,
            ),
            texture_coordinates_buffer: BufferDescriptor::with_binary(
                get_texture_coordinates(),
                0,
                12 * 4,
                BufferUsage::StaticDraw,
            ),
        }
    }
}

impl Geometry for Plane {
    fn draw<'a>(&'a self) -> Draw<'a> {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: 36,
        }
    }

    fn vertices<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.vertices_buffer),
            target: BufferTarget::Buffer,
            component_size: BufferComponentSize::Three,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        }))
    }

    fn normals<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        None
    }

    fn texture_coordinates<'a>(&'a self) -> Option<Ncor<'a, AttributeValue>> {
        Some(Ncor::Owned(AttributeValue::Buffer {
            descriptor: Ncor::Borrowed(&self.texture_coordinates_buffer),
            target: BufferTarget::Buffer,
            component_size: BufferComponentSize::Two,
            data_type: BufferDataType::Float,
            normalized: false,
            bytes_stride: 0,
            bytes_offset: 0,
        }))
    }

    fn attribute_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, AttributeValue>> {
        None
    }

    fn uniform_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, UniformValue>> {
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
fn get_vertices_buffer() -> &'static [u8] {
    unsafe { std::mem::transmute::<&[f32; 18], &[u8; 18 * 4]>(&VERTICES) }
}

#[rustfmt::skip]
const TEXTURE_COORDINATES: [f32; 12] = [
    1.0,1.0,  0.0,1.0,  0.0,0.0,
    0.0,0.0,  1.0,0.0,  1.0,1.0,
];
#[inline]
fn get_texture_coordinates() -> &'static [u8] {
    unsafe { std::mem::transmute::<&[f32; 12], &[u8; 12 * 4]>(&TEXTURE_COORDINATES) }
}
