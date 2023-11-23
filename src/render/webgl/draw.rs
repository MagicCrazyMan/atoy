use super::{buffer::BufferDescriptor, conversion::{GLint, GLsizei, GLintptr}};

#[derive(Debug, Clone)]
pub enum Draw {
    Arrays {
        mode: DrawMode,
        first: GLint,
        count: GLsizei,
    },
    Elements {
        mode: DrawMode,
        count: GLsizei,
        element_type: DrawElementType,
        offset: GLintptr,
        indices: BufferDescriptor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawElementType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}
