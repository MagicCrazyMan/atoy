use super::buffer::BufferDescriptor;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    FRONT,
    BACK,
    FRONT_AND_BACK,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum Draw {
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
        indices: BufferDescriptor,
    },
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawElementType {
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMode {
    POINTS,
    LINES,
    LINE_LOOP,
    LINE_STRIP,
    TRIANGLES,
    TRIANGLE_STRIP,
    TRIANGLE_FAN,
}
