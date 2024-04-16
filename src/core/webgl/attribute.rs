use std::borrow::Cow;

use uuid::Uuid;

use super::buffer::BufferData;

pub enum AttributeBufferData {}

/// Available attribute values.
pub enum AttributeValue<'a> {
    ArrayBuffer {
        data: BufferData<'a>,
        component_size: ArrayBufferComponentSize,
        data_type: ArrayBufferDataType,
        normalized: bool,
        bytes_stride: usize,
        byte_offset: usize,
    },
    // InstancedBuffer {
    //     data: BufferData<'a>,
    //     component_size: BufferComponentSize,
    //     data_type: BufferDataType,
    //     normalized: bool,
    //     component_count_per_instance: usize,
    //     divisor: usize,
    // },
    Vertex1f(f32),
    Vertex2f(f32, f32),
    Vertex3f(f32, f32, f32),
    Vertex4f(f32, f32, f32, f32),
    Integer4(i32, i32, i32, i32),
    UnsignedInteger4(u32, u32, u32, u32),
    Vertex1fv([f32; 1]),
    Vertex2fv([f32; 2]),
    Vertex3fv([f32; 3]),
    Vertex4fv([f32; 4]),
    IntegerVector4([i32; 4]),
    UnsignedIntegerVector4([u32; 4]),
}

/// Available component size of a value get from buffer.
/// According to WebGL definition, it should only be `1`, `2`, `3` or `4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum ArrayBufferComponentSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

/// Available buffer data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayBufferDataType {
    FLOAT,
    BYTE,
    SHORT,
    INT,
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
    HALF_FLOAT,
    INT_2_10_10_10_REV,
    UNSIGNED_INT_2_10_10_10_REV,
}

// impl ArrayBufferDataType {
//     /// Gets bytes length of a data type.
//     pub fn byte_length(&self) -> usize {
//         match self {
//             ArrayBufferDataType::FLOAT => 4,
//             ArrayBufferDataType::BYTE => 1,
//             ArrayBufferDataType::SHORT => 2,
//             ArrayBufferDataType::INT => 4,
//             ArrayBufferDataType::UNSIGNED_BYTE => 1,
//             ArrayBufferDataType::UNSIGNED_SHORT => 2,
//             ArrayBufferDataType::UNSIGNED_INT => 4,
//             ArrayBufferDataType::HALF_FLOAT => 2,
//             ArrayBufferDataType::INT_2_10_10_10_REV => 4,
//             ArrayBufferDataType::UNSIGNED_INT_2_10_10_10_REV => 4,
//         }
//     }
// }

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndicesDataType {
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
}

pub struct Attribute<'a> {
    id: Uuid,
    variable_name: Cow<'a, str>,
    value: AttributeValue<'a>,
}
