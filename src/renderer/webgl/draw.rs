use crate::value::Readonly;

use super::buffer::Buffer;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    FRONT,
    BACK,
    FRONT_AND_BACK,
}

#[allow(non_camel_case_types)]
pub enum Draw<'a> {
    Arrays {
        mode: DrawMode,
        first: i32,
        count: i32,
    },
    Elements {
        mode: DrawMode,
        count: i32,
        offset: i32,
        indices: Readonly<'a, Buffer>,
        indices_data_type: ElementIndicesDataType,
    },
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementIndicesDataType {
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
