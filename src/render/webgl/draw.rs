use log::{warn, info};

use crate::{geometry::Geometry, material::Material, render::pp::State};

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

pub(crate) fn draw(state: &mut State, geometry: &dyn Geometry, material: &dyn Material) {
    // draws entity
    if let Some(num_instances) = (*material).instanced() {
        // draw instanced
        match (*geometry).draw() {
            Draw::Arrays { mode, first, count } => {
                state
                    .gl()
                    .draw_arrays_instanced(mode.gl_enum(), first, count, num_instances)
            }
            Draw::Elements {
                mode,
                count,
                element_type,
                offset,
                indices,
            } => {
                let buffer_item = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "draw",
                            "use buffer store error: {}",
                            err
                        );
                        return;
                    }
                };

                state.gl().bind_buffer(
                    BufferTarget::ElementArrayBuffer.gl_enum(),
                    Some(&buffer_item.gl_buffer()),
                );
                state.gl().draw_elements_instanced_with_i32(
                    mode.gl_enum(),
                    count,
                    element_type.gl_enum(),
                    offset,
                    num_instances,
                );
            }
        }
    } else {
        // draw normally!
        match (*geometry).draw() {
            Draw::Arrays { mode, first, count } => {
                state.gl().draw_arrays(mode.gl_enum(), first, count)
            }
            Draw::Elements {
                mode,
                count,
                element_type,
                offset,
                indices,
            } => {
                let buffer_item = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "draw",
                            "use buffer store error: {}",
                            err
                        );
                        return;
                    }
                };

                state.gl().bind_buffer(
                    BufferTarget::ElementArrayBuffer.gl_enum(),
                    Some(&buffer_item.gl_buffer()),
                );
                state.gl().draw_elements_with_i32(
                    mode.gl_enum(),
                    count,
                    element_type.gl_enum(),
                    offset,
                );
            }
        }
    }
}
