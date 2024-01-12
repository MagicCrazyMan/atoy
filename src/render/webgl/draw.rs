use log::warn;

use crate::render::pp::State;

use super::{
    buffer::{BufferDescriptor, BufferTarget},
    conversion::{GLint, GLintptr, GLsizei, ToGlEnum},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front,
    Back,
    Both,
}

#[derive(Clone)]
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

/// Invokes WebGL draw call by a geometry and material.
pub fn draw(state: &mut State, draw: &Draw) {
    // draw normally!
    match draw {
        Draw::Arrays { mode, first, count } => {
            state.gl().draw_arrays(mode.gl_enum(), *first, *count)
        }
        Draw::Elements {
            mode,
            count,
            element_type,
            offset,
            indices,
        } => {
            let buffer = match state
                .buffer_store_mut()
                .use_buffer(&indices, BufferTarget::ElementArrayBuffer)
            {
                Ok(buffer) => buffer,
                Err(err) => {
                    warn!(
                        target: "Draw",
                        "use buffer store error: {}",
                        err
                    );
                    return;
                }
            };

            state
                .gl()
                .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
            state.gl().draw_elements_with_i32(
                mode.gl_enum(),
                *count,
                element_type.gl_enum(),
                *offset,
            );
            state
                .gl()
                .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), None);
            state.buffer_store_mut().unuse_buffer(&indices);
        }
    }
}
