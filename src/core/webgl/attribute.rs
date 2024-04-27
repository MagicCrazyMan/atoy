use super::buffer::Buffer;

/// Available attribute values.
pub enum AttributeValue {
    ArrayBuffer {
        buffer: Buffer,
        component_size: ArrayBufferComponentSize,
        data_type: ArrayBufferDataType,
        normalized: bool,
        bytes_stride: usize,
        byte_offset: usize,
    },
    InstancedBuffer {
        buffer: Buffer,
        component_size: ArrayBufferComponentSize,
        data_type: ArrayBufferDataType,
        normalized: bool,
        component_count_per_instance: usize,
        divisor: usize,
    },
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayBufferDataType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndicesDataType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}
