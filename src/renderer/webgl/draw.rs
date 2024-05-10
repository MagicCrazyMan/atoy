use std::ops::Range;

use hashbrown::HashMap;
use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity, geometry::Geometry, material::webgl::StandardMaterial, value::Readonly,
};

use super::{
    attribute::AttributeValue, buffer::{Buffer, BufferStore, BufferTarget}, conversion::ToGlEnum, error::Error, framebuffer::{Framebuffer, OperableBuffer}, program::Program, uniform::{UniformBlockValue, UniformValue}
};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    FRONT,
    BACK,
    FRONT_AND_BACK,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthFunction {
    NEVER,
    LESS,
    EQUAL,
    LEQUAL,
    GREATER,
    NOTEQUAL,
    GEQUAL,
    ALWAYS,
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

enum DrawParams<'a> {
    FromGeometry(&'a dyn Geometry),
    Custom {
        mode: DrawMode,
        range: Range<usize>,
        indices: Option<(
            Readonly<'a, Buffer>,
            ElementIndicesDataType,
            Option<Range<usize>>,
        )>,
    },
}

impl<'a> DrawParams<'a> {
    fn mode(&self) -> DrawMode {
        match self {
            Self::FromGeometry(geometry) => geometry.draw_mode(),
            Self::Custom { mode, .. } => *mode,
        }
    }

    fn range(&self) -> Range<usize> {
        match self {
            Self::FromGeometry(geometry) => geometry.draw_range(),
            Self::Custom { range, .. } => range.clone(),
        }
    }

    fn indices(
        &self,
    ) -> Option<(
        Readonly<'a, Buffer>,
        ElementIndicesDataType,
        Option<Range<usize>>,
    )> {
        match self {
            Self::FromGeometry(geometry) => {
                let indexed_geometry = geometry.as_indexed_geometry()?;
                Some((
                    indexed_geometry.indices(),
                    indexed_geometry.indices_data_type(),
                    indexed_geometry.indices_range(),
                ))
            }
            Self::Custom { indices, .. } => match indices {
                Some((buffer, data_type, indices_range)) => {
                    Some((Readonly::Owned(buffer.as_ref().clone()), *data_type, indices_range.clone()))
                }
                None => None,
            },
        }
    }
}

pub struct Draw<'a> {
    // framebuffer: Option<&'a Framebuffer>,
    // draw_buffers: Option<Vec<OperableBuffer>>,
    // program: Option<&'a Program>,
    params: DrawParams<'a>,
    // entity: Option<&'a dyn Entity>,
    // geometry: Option<&'a dyn Geometry>,
    // material: Option<&'a dyn StandardMaterial>,
    // custom_attributes: Option<HashMap<&'a str, AttributeValue<'a>>>,
    // custom_uniforms: Option<HashMap<&'a str, UniformValue<'a>>>,
    // custom_uniform_blocks: Option<HashMap<&'a str, UniformBlockValue<'a>>>,
    // cull_face: Option<CullFace>,
    // depth_test: Option<bool>,
    // depth_func: Option<DepthFunction>,
    // depth_mask: Option<bool>,
    // depth_range: Option<Range<f32>>,
    // add more options in the future
}

impl<'a> Draw<'a> {
    // /// Construct a new draw command using [`WebGl2RenderingContext::draw_arrays`].
    // pub fn new_draw_arrays(mode: DrawMode, range: Range<usize>) -> Self {
    //     let params = DrawParams::Custom {
    //         mode,
    //         range,
    //         indices: None,
    //     };

    //     Self { params }
    // }

    // /// Construct a new draw command using [`WebGl2RenderingContext::draw_elements_with_i32`].
    // pub fn new_draw_elements(
    //     mode: DrawMode,
    //     range: Range<usize>,
    //     indices: Readonly<'a, Buffer>,
    //     indices_data_type: ElementIndicesDataType,
    //     indices_range: Option<Range<usize>>,
    // ) -> Self {
    //     let params = DrawParams::Custom {
    //         mode,
    //         range,
    //         indices: Some((indices, indices_data_type, indices_range)),
    //     };
    //     Self { params }
    // }

    /// Constructs a new draw command from a [`Geometry`].
    pub fn from_geometry(geometry: &'a dyn Geometry) -> Self {
        Self {
            params: DrawParams::FromGeometry(geometry),
        }
    }

    /// Executes draw command.
    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        let mode = self.params.mode().gl_enum();
        let range = self.params.range();
        let offset = range.start as i32;
        let count = range.count() as i32;
        match self.params.indices() {
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
