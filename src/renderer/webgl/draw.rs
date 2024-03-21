use web_sys::WebGl2RenderingContext;

use crate::value::Readonly;

use super::{
    buffer::{Buffer, BufferStore, BufferTarget},
    conversion::ToGlEnum,
    error::Error,
};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    FRONT,
    BACK,
    FRONT_AND_BACK,
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

impl<'a> Draw<'a> {
    /// Executes draw command.
    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        match self {
            Draw::Arrays { mode, first, count } => gl.draw_arrays(mode.gl_enum(), *first, *count),
            Draw::Elements {
                mode,
                count,
                offset,
                indices,
                indices_data_type,
            } => {
                match buffer_store {
                    Some(store) => {
                        store.register(&indices)?;
                    }
                    None => {
                        indices.init(gl)?;
                    }
                };

                indices.bind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;
                gl.draw_elements_with_i32(
                    mode.gl_enum(),
                    *count,
                    indices_data_type.gl_enum(),
                    *offset,
                );
                indices.unbind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;
            }
        }

        Ok(())
    }
}
