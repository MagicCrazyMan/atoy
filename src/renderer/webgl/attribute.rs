use std::borrow::Cow;

use super::{
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget},
    program::CustomBinding,
};

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

/// Attribute internal bindings.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AttributeInternalBinding {
    GeometryPosition,
    GeometryTextureCoordinate,
    GeometryNormal,
    GeometryTangent,
    GeometryBitangent,
}

impl AttributeInternalBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            AttributeInternalBinding::GeometryPosition => "a_Position",
            AttributeInternalBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeInternalBinding::GeometryNormal => "a_Normal",
            AttributeInternalBinding::GeometryTangent => "a_Tangent",
            AttributeInternalBinding::GeometryBitangent => "a_Bitangent",
        }
    }

    /// Tries to find attribute internal binding from a variable name.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "a_Position" => Some(AttributeInternalBinding::GeometryPosition),
            "a_TexCoord" => Some(AttributeInternalBinding::GeometryTextureCoordinate),
            "a_Normal" => Some(AttributeInternalBinding::GeometryNormal),
            "a_Tangent" => Some(AttributeInternalBinding::GeometryTangent),
            "a_Bitangent" => Some(AttributeInternalBinding::GeometryBitangent),
            _ => None,
        }
    }

    /// Returns this attribute internal binding as a [`CustomBinding`].
    pub fn as_custom_binding(&self) -> CustomBinding {
        match self {
            AttributeInternalBinding::GeometryPosition
            | AttributeInternalBinding::GeometryTextureCoordinate
            | AttributeInternalBinding::GeometryNormal
            | AttributeInternalBinding::GeometryTangent
            | AttributeInternalBinding::GeometryBitangent => {
                CustomBinding::FromGeometry(Cow::Borrowed(self.variable_name()))
            }
        }
    }
}