use nalgebra::{Vector1, Vector2, Vector3, Vector4};
use proc::GlEnum;

use crate::anewthing::buffer::Buffer;

use super::buffer::WebGlBufferData;

/// Available number of components per vertex attribute.
/// According to WebGL definition, it should only be `1`, `2`, `3` or `4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum WebGlArrayBufferComponentSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

/// Available buffer data types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlArrayBufferDataType {
    Float,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    HalfFloat,
    Int2_10_10_10Rev,
    UnsignedInt2_10_10_10Rev,
}

/// Available elemental index buffer data types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlIndicesDataType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

/// Available attribute values.
pub enum WebGlAttributeValue<'a> {
    ArrayBuffer {
        buffer: &'a mut Buffer<WebGlBufferData>,
        component_size: WebGlArrayBufferComponentSize,
        data_type: WebGlArrayBufferDataType,
        normalized: bool,
        byte_stride: usize,
        byte_offset: usize,
    },
    InstancedBuffer {
        buffer: &'a mut Buffer<WebGlBufferData>,
        component_size: WebGlArrayBufferComponentSize,
        instance_size: usize,
        data_type: WebGlArrayBufferDataType,
        normalized: bool,
        byte_stride: usize,
        byte_offset: usize,
    },
    Float1(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    Integer4(i32, i32, i32, i32),
    UnsignedInteger4(u32, u32, u32, u32),
    FloatVector1(&'a Vector1<f32>),
    FloatVector2(&'a Vector2<f32>),
    FloatVector3(&'a Vector3<f32>),
    FloatVector4(&'a Vector4<f32>),
    IntegerVector4(&'a Vector4<i32>),
    UnsignedIntegerVector4(&'a Vector4<u32>),
}
