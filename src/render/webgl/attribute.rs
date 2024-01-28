use std::borrow::Cow;

use super::buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget};

/// Available attribute values.
pub enum AttributeValue {
    Buffer {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        bytes_stride: usize,
        bytes_offset: usize,
    },
    InstancedBuffer {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        component_count_per_instance: usize,
        divisor: usize,
    },
    Vertex1f(f32),
    Vertex2f(f32, f32),
    Vertex3f(f32, f32, f32),
    Vertex4f(f32, f32, f32, f32),
    Vertex1fv([f32; 1]),
    Vertex2fv([f32; 2]),
    Vertex3fv([f32; 3]),
    Vertex4fv([f32; 4]),
    UnsignedInteger4(u32, u32, u32, u32),
    UnsignedIntegerVector4([u32; 4]),
    Integer4(i32, i32, i32, i32),
    IntegerVector4([i32; 4]),
}

/// Attribute binding sources.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AttributeBinding {
    GeometryPosition,
    GeometryTextureCoordinate,
    GeometryNormal,
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    FromEntity(Cow<'static, str>),
}

impl AttributeBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            AttributeBinding::GeometryPosition => "a_Position",
            AttributeBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeBinding::GeometryNormal => "a_Normal",
            AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::FromEntity(name) => name,
        }
    }
}
