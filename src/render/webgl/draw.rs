use wasm_bindgen_test::console_log;

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
            Draw::Arrays {
                mode,
                first,
                count: num_vertices,
            } => {
                state
                    .gl()
                    .draw_arrays_instanced(mode.gl_enum(), first, num_vertices, num_instances)
            }
            Draw::Elements {
                mode,
                count: num_vertices,
                element_type,
                offset,
                indices,
            } => {
                let item = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log warning
                        console_log!("{}", err);
                        return;
                    }
                };

                state.gl().bind_buffer(
                    BufferTarget::ElementArrayBuffer.gl_enum(),
                    Some(&item.gl_buffer()),
                );
                state.gl().draw_elements_instanced_with_i32(
                    mode.gl_enum(),
                    num_vertices,
                    element_type.gl_enum(),
                    offset,
                    num_instances,
                );
            }
        }
    } else {
        // draw normally!
        match (*geometry).draw() {
            Draw::Arrays {
                mode,
                first,
                count: num_vertices,
            } => state.gl().draw_arrays(mode.gl_enum(), first, num_vertices),
            Draw::Elements {
                mode,
                count: num_vertices,
                element_type,
                offset,
                indices,
            } => {
                let item = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log warning
                        console_log!("{}", err);
                        return;
                    }
                };

                state.gl().bind_buffer(
                    BufferTarget::ElementArrayBuffer.gl_enum(),
                    Some(&item.gl_buffer()),
                );
                state.gl().draw_elements_with_i32(
                    mode.gl_enum(),
                    num_vertices,
                    element_type.gl_enum(),
                    offset,
                );
            }
        }
    }
}
