use web_sys::WebGl2RenderingContext;

use super::buffer::BufferDescriptor;

pub enum Draw<'a> {
    Arrays {
        mode: DrawMode,
        first: i32,
        count: i32,
    },
    Elements {
        mode: DrawMode,
        count: i32,
        element_type: DrawElementType,
        offset: i32,
        indices: &'a BufferDescriptor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawElementType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

impl DrawElementType {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            DrawElementType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            DrawElementType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            DrawElementType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

impl DrawMode {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            DrawMode::Points => WebGl2RenderingContext::POINTS,
            DrawMode::Lines => WebGl2RenderingContext::LINES,
            DrawMode::LineLoop => WebGl2RenderingContext::LINE_LOOP,
            DrawMode::LineStrip => WebGl2RenderingContext::LINE_STRIP,
            DrawMode::Triangles => WebGl2RenderingContext::TRIANGLES,
            DrawMode::TriangleStrip => WebGl2RenderingContext::TRIANGLE_STRIP,
            DrawMode::TriangleFan => WebGl2RenderingContext::TRIANGLE_FAN,
        }
    }
}
