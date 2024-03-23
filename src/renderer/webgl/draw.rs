use std::ops::Range;

use web_sys::WebGl2RenderingContext;

use crate::{geometry::Geometry, value::Readonly};

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

#[derive(Debug)]
pub struct Draw<'a> {
    mode: DrawMode,
    range: Range<usize>,
    indices: Option<(
        Readonly<'a, Buffer>,
        ElementIndicesDataType,
        Option<Range<usize>>,
    )>,
}

impl<'a> Draw<'a> {
    /// Construct a new draw command using [`WebGl2RenderingContext::draw_arrays`].
    pub fn new_draw_arrays(mode: DrawMode, range: Range<usize>) -> Self {
        Self {
            mode,
            range,
            indices: None,
        }
    }

    /// Construct a new draw command using [`WebGl2RenderingContext::draw_elements_with_i32`].
    pub fn new_draw_elements(
        mode: DrawMode,
        range: Range<usize>,
        indices: Readonly<'a, Buffer>,
        indices_data_type: ElementIndicesDataType,
    ) -> Self {
        Self {
            mode,
            range,
            indices: Some((indices, indices_data_type, None)),
        }
    }

    /// Construct a new draw command using [`WebGl2RenderingContext::draw_range_elements_with_i32`].
    pub fn new_draw_range_elements(
        mode: DrawMode,
        range: Range<usize>,
        indices: Readonly<'a, Buffer>,
        indices_data_type: ElementIndicesDataType,
        indices_range: Option<Range<usize>>,
    ) -> Self {
        Self {
            mode,
            range,
            indices: Some((indices, indices_data_type, indices_range)),
        }
    }

    /// Constructs a new draw command from a [`Geometry`].
    pub fn from_geometry<G>(geometry: &'a G) -> Self
    where
        G: Geometry + ?Sized,
    {
        let mode = geometry.draw_mode();
        let range = geometry.draw_range();
        let indices = geometry
            .as_indexed_geometry()
            .map(|g| (g.indices(), g.indices_data_type(), g.indices_range()));

        Self {
            mode,
            range,
            indices,
        }
    }

    /// Executes draw command.
    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        let mode = self.mode.gl_enum();
        let offset = self.range.start as i32;
        let count = self.range.clone().count() as i32;
        match self.indices.as_ref() {
            Some((indices, indices_type, indices_range)) => {
                match buffer_store {
                    Some(store) => {
                        store.register(&indices)?;
                    }
                    None => {
                        indices.init(gl)?;
                    }
                };

                indices.bind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;

                let indices_type = indices_type.gl_enum();
                match indices_range {
                    Some(indices_range) => {
                        let start = indices_range.start as u32;
                        let end = indices_range.end as u32;
                        gl.draw_range_elements_with_i32(
                            mode,
                            start,
                            end,
                            count,
                            indices_type,
                            offset,
                        );
                    }
                    None => gl.draw_elements_with_i32(mode, count, indices_type, offset),
                }

                indices.unbind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;
            }
            None => {
                gl.draw_arrays(mode, offset, count);
            }
        }

        Ok(())
    }
}
