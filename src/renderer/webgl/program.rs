use std::{borrow::Cow, cell::RefCell, iter::FromIterator, rc::Rc};

use gl_matrix4rust::GLF32;
use hashbrown::{HashMap, HashSet};
use log::warn;
use regex::Regex;
use web_sys::{
    WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation, WebGlVertexArrayObject,
};

use crate::{entity::Entity, geometry::Geometry, material::webgl::StandardMaterial};

use super::{
    attribute::{AttributeBinding, AttributeValue, VertexAttributeArrayUnbinder},
    buffer::{BufferStore, BufferTarget},
    conversion::ToGlEnum,
    error::Error,
    state::FrameState,
    texture::{TextureStore, TextureUnbinder},
    uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
};

/// Replacement derivative name for injecting [`ShaderProvider::vertex_defines`] and
/// [`ShaderProvider::fragment_defines`] when creating program using [`ProgramStore`].
const GLSL_REPLACEMENT_DEFINES: &'static str = "Defines";
/// Regular expression for matching replacement macro `#include <snippet_name>;`.
const GLSL_REPLACEMENT_DERIVATIVE_REGEX: &'static str = "^\\s*#include\\s+(.+)\\s*$";

/// GLSL `#define` macro definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Define<'a> {
    /// Define macro with value, build to `#define <name> <value>`.
    WithValue(Cow<'a, str>, Cow<'a, str>),
    /// Define macro without value, build to `#define <name>`.
    WithoutValue(Cow<'a, str>),
}

impl<'a> Define<'a> {
    /// Returns name of define macro.
    pub fn name(&self) -> &str {
        match self {
            Define::WithValue(name, _) | Define::WithoutValue(name) => &name,
        }
    }

    /// Returns value of define macro.
    pub fn value(&self) -> Option<&str> {
        match self {
            Define::WithValue(_, value) => Some(&value),
            Define::WithoutValue(_) => None,
        }
    }

    /// Builds to GLSL define macro derivative.
    pub fn build(&self) -> String {
        match self {
            Define::WithValue(name, value) => format!("#define {} {}", name, value),
            Define::WithoutValue(name) => format!("#define {}", name),
        }
    }
}

/// A source providing data for compiling a [`WebGlProgram`].
pub trait ProgramSource {
    /// Global unique name for the program source.
    fn name(&self) -> Cow<'_, str>;

    /// Returns source code of vertex shader.
    fn vertex_source(&self) -> Cow<'_, str>;

    /// Returns source code of fragment shader.
    fn fragment_source(&self) -> Cow<'_, str>;

    /// Returns universal defines macros for both vertex and fragment shaders.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of vertex shader to make this work.
    fn universal_defines(&self) -> Cow<'_, [Define<'_>]>;

    /// Returns defines macros for vertex shader.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of vertex shader to make this work.
    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]>;

    /// Returns defines macros for fragment shader.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of fragment shader to make this work.
    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]>;

    /// Returns self-associated GLSL code snippet by name.
    fn snippet(&self, name: &str) -> Option<Cow<'_, str>>;
}

/// Compiled program.
#[derive(Debug, Clone)]
pub struct Program {
    gl: WebGl2RenderingContext,
    name: String,
    program: WebGlProgram,
    vertex_shader: WebGlShader,
    fragment_shader: WebGlShader,

    attribute_locations: Rc<HashMap<AttributeBinding, u32>>,
    uniform_locations: Rc<HashMap<UniformBinding, WebGlUniformLocation>>,
    uniform_block_indices: Rc<HashMap<UniformBlockBinding, u32>>,

    using: Rc<RefCell<Option<WebGlProgram>>>,
    vao: Rc<RefCell<Option<WebGlVertexArrayObject>>>,
    attribute_unbinders: Rc<RefCell<Option<Vec<VertexAttributeArrayUnbinder>>>>,
    uniform_unbinders: Rc<RefCell<Option<Vec<TextureUnbinder>>>>,
}

impl Program {
    /// Returns program source name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns `true` if this program is using.
    pub fn is_using(&self) -> bool {
        let using = self.using.borrow();
        if let Some(using_program) = using.as_ref() {
            using_program == &self.program
        } else {
            false
        }
    }

    /// Uses this program.
    pub fn use_program(&self) -> Result<(), Error> {
        let mut using = self.using.borrow_mut();
        match using.as_ref() {
            Some(using) => {
                if using == &self.program {
                    Ok(())
                } else {
                    Err(Error::ProgramOccupied)
                }
            }
            None => {
                self.gl.use_program(Some(&self.program));
                *using = Some(self.program.clone());
                Ok(())
            }
        }
    }

    /// Unuses this program.
    pub fn unuse_program(&self) -> Result<(), Error> {
        self.unbind_vertex_array_object()?;
        self.unbind_attributes()?;
        self.unbind_uniforms()?;

        self.gl.use_program(None);
        *self.using.borrow_mut() = None;

        Ok(())
    }

    /// Binds vertex array object
    pub fn bind_vertex_array_object(&self, vao: WebGlVertexArrayObject) -> Result<(), Error> {
        let mut v = self.vao.borrow_mut();
        match v.as_ref() {
            Some(v) => {
                if v == &vao {
                    Ok(())
                } else {
                    Err(Error::VertexArrayObjectOccupied)
                }
            }
            None => {
                self.gl.bind_vertex_array(Some(&vao));
                *v = Some(vao);
                Ok(())
            }
        }
    }

    /// Binds a [`AttributeValue`] to a uniform location by [`AttributeBinding`].
    pub fn bind_attribute_value_by_binding(
        &self,
        binding: &AttributeBinding,
        value: &AttributeValue,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        let Some(location) = self.attribute_locations.get(binding) else {
            return Err(Error::NoSuchAttribute(binding.clone()));
        };

        self.bind_attribute_value_by_location(*location, value, buffer_store)?;

        Ok(())
    }

    /// Binds a [`AttributeValue`] to a uniform location by [`AttributeBinding`].
    pub fn bind_attribute_value_by_location(
        &self,
        location: u32,
        value: &AttributeValue,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        let mut unbinders = self.attribute_unbinders.borrow_mut();
        let unbinders = unbinders.get_or_insert_with(Vec::new);

        match value {
            AttributeValue::ArrayBuffer {
                buffer,
                component_size,
                data_type,
                normalized,
                bytes_stride,
                byte_offset,
            } => {
                match buffer_store {
                    Some(store) => {
                        store.register(buffer)?;
                    }
                    None => {
                        buffer.init(&self.gl)?;
                    }
                }

                buffer.bind(BufferTarget::ARRAY_BUFFER)?;
                self.gl.vertex_attrib_pointer_with_i32(
                    location,
                    *component_size as i32,
                    data_type.gl_enum(),
                    *normalized,
                    *bytes_stride as i32,
                    *byte_offset as i32,
                );
                self.gl.enable_vertex_attrib_array(location);
                buffer.unbind(BufferTarget::ARRAY_BUFFER)?;

                unbinders.push(VertexAttributeArrayUnbinder::new(location, self.gl.clone()));
            }
            AttributeValue::InstancedBuffer {
                buffer,
                component_size,
                data_type,
                normalized,
                component_count_per_instance,
                divisor,
            } => {
                match buffer_store {
                    Some(store) => {
                        store.register(buffer)?;
                    }
                    None => {
                        buffer.init(&self.gl)?;
                    }
                }

                buffer.bind(BufferTarget::ARRAY_BUFFER)?;
                let component_size = *component_size as usize;
                // binds each instance
                for i in 0..*component_count_per_instance {
                    let offset_location = location + i as u32;
                    let stride =
                        data_type.byte_length() * component_size * component_count_per_instance;
                    let offset = i * data_type.byte_length() * component_size;
                    self.gl.vertex_attrib_pointer_with_i32(
                        offset_location,
                        component_size as i32,
                        data_type.gl_enum(),
                        *normalized,
                        stride as i32,
                        offset as i32,
                    );
                    self.gl.enable_vertex_attrib_array(offset_location);
                    self.gl
                        .vertex_attrib_divisor(offset_location, *divisor as u32);

                    unbinders.push(VertexAttributeArrayUnbinder::new(
                        offset_location,
                        self.gl.clone(),
                    ));
                }
                buffer.unbind(BufferTarget::ARRAY_BUFFER)?;
            }
            AttributeValue::Vertex1f(x) => self.gl.vertex_attrib1f(location, *x),
            AttributeValue::Vertex2f(x, y) => self.gl.vertex_attrib2f(location, *x, *y),
            AttributeValue::Vertex3f(x, y, z) => self.gl.vertex_attrib3f(location, *x, *y, *z),
            AttributeValue::Vertex4f(x, y, z, w) => {
                self.gl.vertex_attrib4f(location, *x, *y, *z, *w)
            }
            AttributeValue::Vertex1fv(v) => self.gl.vertex_attrib1fv_with_f32_array(location, v),
            AttributeValue::Vertex2fv(v) => self.gl.vertex_attrib2fv_with_f32_array(location, v),
            AttributeValue::Vertex3fv(v) => self.gl.vertex_attrib3fv_with_f32_array(location, v),
            AttributeValue::Vertex4fv(v) => self.gl.vertex_attrib4fv_with_f32_array(location, v),
            AttributeValue::UnsignedInteger4(x, y, z, w) => {
                self.gl.vertex_attrib_i4ui(location, *x, *y, *z, *w)
            }
            AttributeValue::Integer4(x, y, z, w) => {
                self.gl.vertex_attrib_i4i(location, *x, *y, *z, *w)
            }
            AttributeValue::IntegerVector4(mut values) => self
                .gl
                .vertex_attrib_i4iv_with_i32_array(location, &mut values),
            AttributeValue::UnsignedIntegerVector4(mut values) => self
                .gl
                .vertex_attrib_i4uiv_with_u32_array(location, &mut values),
        };

        Ok(())
    }

    /// Binds attributes.
    pub fn bind_attributes(
        &self,
        state: Option<&FrameState>,
        entity: Option<&dyn Entity>,
        geometry: Option<&dyn Geometry>,
        material: Option<&dyn StandardMaterial>,
    ) -> Result<(), Error> {
        for (binding, location) in self.attribute_locations.iter() {
            let value = match binding {
                AttributeBinding::GeometryPosition => {
                    geometry.and_then(|geometry| geometry.positions())
                }
                AttributeBinding::GeometryTextureCoordinate => {
                    geometry.and_then(|geometry| geometry.texture_coordinates())
                }
                AttributeBinding::GeometryNormal => {
                    geometry.and_then(|geometry| geometry.normals())
                }
                AttributeBinding::GeometryTangent => {
                    geometry.and_then(|geometry| geometry.tangents())
                }
                AttributeBinding::GeometryBitangent => {
                    geometry.and_then(|geometry| geometry.bitangents())
                }
                AttributeBinding::FromGeometry(name) => {
                    geometry.and_then(|geometry| geometry.attribute_value(name))
                }
                AttributeBinding::FromMaterial(name) => {
                    material.and_then(|material| material.attribute_value(name))
                }
                AttributeBinding::FromEntity(name) => {
                    entity.and_then(|entity| entity.attribute_value(name))
                }
                AttributeBinding::Custom(_) => {
                    continue;
                }
            };

            let Some(value) = value else {
                warn!(
                    target: "Program",
                    "no value specified for attribute {}",
                    binding.variable_name()
                );
                continue;
            };

            self.bind_attribute_value_by_location(
                *location,
                &value,
                state.map(|state| state.buffer_store()),
            )?;
        }

        Ok(())
    }

    /// Binds a [`UniformValue`] to a uniform location by [`UniformBinding`].
    pub fn bind_uniform_value_by_binding(
        &self,
        binding: &UniformBinding,
        value: &UniformValue,
        texture_store: Option<&TextureStore>,
    ) -> Result<(), Error> {
        let Some(location) = self.uniform_locations.get(binding) else {
            return Err(Error::NoSuchUniform(binding.clone()));
        };

        self.bind_uniform_value_by_location(location, value, texture_store)?;

        Ok(())
    }

    /// Binds a [`UniformValue`] to a uniform location.
    /// No error thrown if location is not associated with this program.
    pub fn bind_uniform_value_by_location(
        &self,
        location: &WebGlUniformLocation,
        value: &UniformValue,
        texture_store: Option<&TextureStore>,
    ) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        let mut unbinders = self.uniform_unbinders.borrow_mut();
        let unbinders = unbinders.get_or_insert_with(Vec::new);

        match value {
            UniformValue::Bool(v) => {
                if *v {
                    self.gl.uniform1i(Some(location), 1);
                } else {
                    self.gl.uniform1i(Some(location), 0);
                };
            }
            UniformValue::UnsignedInteger1(x) => {
                self.gl.uniform1ui(Some(location), *x);
            }
            UniformValue::UnsignedInteger2(x, y) => {
                self.gl.uniform2ui(Some(location), *x, *y);
            }
            UniformValue::UnsignedInteger3(x, y, z) => {
                self.gl.uniform3ui(Some(location), *x, *y, *z);
            }
            UniformValue::UnsignedInteger4(x, y, z, w) => {
                self.gl.uniform4ui(Some(location), *x, *y, *z, *w);
            }
            UniformValue::Float1(x) => {
                self.gl.uniform1f(Some(location), *x);
            }
            UniformValue::Float2(x, y) => {
                self.gl.uniform2f(Some(location), *x, *y);
            }
            UniformValue::Float3(x, y, z) => {
                self.gl.uniform3f(Some(location), *x, *y, *z);
            }
            UniformValue::Float4(x, y, z, w) => {
                self.gl.uniform4f(Some(location), *x, *y, *z, *w);
            }
            UniformValue::Integer1(x) => {
                self.gl.uniform1i(Some(location), *x);
            }
            UniformValue::Integer2(x, y) => {
                self.gl.uniform2i(Some(location), *x, *y);
            }
            UniformValue::Integer3(x, y, z) => {
                self.gl.uniform3i(Some(location), *x, *y, *z);
            }
            UniformValue::Integer4(x, y, z, w) => {
                self.gl.uniform4i(Some(location), *x, *y, *z, *w);
            }
            UniformValue::FloatVector1(data) => {
                self.gl.uniform1fv_with_f32_array(Some(location), data);
            }
            UniformValue::FloatVector2(data) => {
                self.gl.uniform2fv_with_f32_array(Some(location), data);
            }
            UniformValue::FloatVector3(data) => {
                self.gl.uniform3fv_with_f32_array(Some(location), data);
            }
            UniformValue::FloatVector4(data) => {
                self.gl.uniform4fv_with_f32_array(Some(location), data);
            }
            UniformValue::IntegerVector1(data) => {
                self.gl.uniform1iv_with_i32_array(Some(location), data);
            }
            UniformValue::IntegerVector2(data) => {
                self.gl.uniform2iv_with_i32_array(Some(location), data);
            }
            UniformValue::IntegerVector3(data) => {
                self.gl.uniform3iv_with_i32_array(Some(location), data);
            }
            UniformValue::IntegerVector4(data) => {
                self.gl.uniform4iv_with_i32_array(Some(location), data);
            }
            UniformValue::UnsignedIntegerVector1(data) => {
                self.gl.uniform1uiv_with_u32_array(Some(location), data);
            }
            UniformValue::UnsignedIntegerVector2(data) => {
                self.gl.uniform2uiv_with_u32_array(Some(location), data);
            }
            UniformValue::UnsignedIntegerVector3(data) => {
                self.gl.uniform3uiv_with_u32_array(Some(location), data);
            }
            UniformValue::UnsignedIntegerVector4(data) => {
                self.gl.uniform4uiv_with_u32_array(Some(location), data);
            }
            UniformValue::Matrix2 { data, transpose } => {
                self.gl
                    .uniform_matrix2fv_with_f32_array(Some(location), *transpose, data);
            }
            UniformValue::Matrix3 { data, transpose } => {
                self.gl
                    .uniform_matrix3fv_with_f32_array(Some(location), *transpose, data);
            }
            UniformValue::Matrix4 { data, transpose } => {
                self.gl
                    .uniform_matrix4fv_with_f32_array(Some(location), *transpose, data);
            }
            UniformValue::Texture2D { unit, .. }
            | UniformValue::Texture2DArray { unit, .. }
            | UniformValue::Texture3D { unit, .. }
            | UniformValue::TextureCubeMap { unit, .. } => {
                let unbinder = match value {
                    UniformValue::Texture2D { texture, .. } => {
                        match texture_store {
                            Some(store) => {
                                store.register(texture)?;
                            }
                            None => {
                                texture.init(&self.gl)?;
                            }
                        };

                        texture.bind(*unit)?
                    }
                    UniformValue::Texture2DArray { texture, .. } => {
                        match texture_store {
                            Some(store) => {
                                store.register(texture)?;
                            }
                            None => {
                                texture.init(&self.gl)?;
                            }
                        };

                        texture.bind(*unit)?
                    }
                    UniformValue::Texture3D { texture, .. } => {
                        match texture_store {
                            Some(store) => {
                                store.register(texture)?;
                            }
                            None => {
                                texture.init(&self.gl)?;
                            }
                        };

                        texture.bind(*unit)?
                    }
                    UniformValue::TextureCubeMap { texture, .. } => {
                        match texture_store {
                            Some(store) => {
                                store.register(texture)?;
                            }
                            None => {
                                texture.init(&self.gl)?;
                            }
                        };

                        texture.bind(*unit)?
                    }
                    _ => unreachable!(),
                };

                self.gl.uniform1i(Some(location), unit.unit_index() as i32);

                unbinders.push(unbinder);
            }
        };

        Ok(())
    }

    /// Binds uniforms.
    pub fn bind_uniforms(
        &self,
        state: Option<&FrameState>,
        entity: Option<&dyn Entity>,
        geometry: Option<&dyn Geometry>,
        material: Option<&dyn StandardMaterial>,
    ) -> Result<(), Error> {
        for (binding, location) in self.uniform_locations.iter() {
            let value = match binding {
                UniformBinding::ModelMatrix
                | UniformBinding::ViewMatrix
                | UniformBinding::ProjMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ViewProjMatrix => {
                    let data = match binding {
                        UniformBinding::ModelMatrix => {
                            entity.map(|entity| entity.compose_model_matrix().gl_f32())
                        }
                        UniformBinding::NormalMatrix => {
                            entity.map(|entity| entity.compose_normal_matrix().gl_f32())
                        }
                        UniformBinding::ViewMatrix => {
                            state.map(|state| state.camera().view_matrix().gl_f32())
                        }
                        UniformBinding::ProjMatrix => {
                            state.map(|state| state.camera().proj_matrix().gl_f32())
                        }
                        UniformBinding::ViewProjMatrix => {
                            state.map(|state| state.camera().view_proj_matrix().gl_f32())
                        }
                        _ => unreachable!(),
                    };

                    match data {
                        Some(data) => Some(UniformValue::Matrix4 {
                            data,
                            transpose: false,
                        }),
                        None => None,
                    }
                }
                UniformBinding::CameraPosition => state
                    .map(|state| UniformValue::FloatVector3(state.camera().position().gl_f32())),
                UniformBinding::RenderTime => {
                    state.map(|state| UniformValue::Float1(state.timestamp() as f32))
                }
                UniformBinding::CanvasSize => state.map(|state| {
                    UniformValue::UnsignedIntegerVector2([
                        state.canvas().width(),
                        state.canvas().height(),
                    ])
                }),
                UniformBinding::DrawingBufferSize => Some(UniformValue::IntegerVector2([
                    self.gl.drawing_buffer_width(),
                    self.gl.drawing_buffer_width(),
                ])),
                UniformBinding::FromEntity(name) => {
                    entity.and_then(|entity| entity.uniform_value(name))
                }
                UniformBinding::FromGeometry(name) => {
                    geometry.and_then(|geometry| geometry.uniform_value(name))
                }
                UniformBinding::FromMaterial(name) => {
                    material.and_then(|material| material.uniform_value(name))
                }
                UniformBinding::Custom(_) => {
                    continue;
                }
            };

            let Some(value) = value else {
                warn!(
                    target: "Program",
                    "no value specified for uniform {}",
                    binding.variable_name()
                );
                continue;
            };

            self.bind_uniform_value_by_location(
                location,
                &value,
                state.map(|state| state.texture_store()),
            )?;
        }

        Ok(())
    }

    /// Gets a uniform block index by [`UniformBinding`] and mounts it to Uniform Buffer Object mount point.
    pub fn mount_uniform_block_by_binding(
        &self,
        binding: &UniformBlockBinding,
        mount_point: u32,
    ) -> Result<(), Error> {
        let Some(index) = self.uniform_block_indices.get(binding) else {
            return Err(Error::NoSuchUniformBlock(binding.clone()));
        };

        self.mount_uniform_block_by_index(*index, mount_point)?;

        Ok(())
    }

    /// Mounts a uniform block index to Uniform Buffer Object mount point.
    /// No error thrown if uniform block index is not associated with this program.
    pub fn mount_uniform_block_by_index(&self, index: u32, mount_point: u32) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        self.gl
            .uniform_block_binding(&self.program, index, mount_point);

        Ok(())
    }

    /// Binds a [`UniformBlockValue`] to a uniform location by [`UniformBinding`].
    pub fn bind_uniform_block_value_by_binding(
        &self,
        binding: &UniformBlockBinding,
        value: &UniformBlockValue,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        let Some(index) = self.uniform_block_indices.get(binding) else {
            return Err(Error::NoSuchUniformBlock(binding.clone()));
        };

        self.bind_uniform_block_value_by_index(*index, value, buffer_store)?;

        Ok(())
    }

    /// Binds a [`UniformBlockValue`] to a uniform block index.
    /// No error thrown if location is not associated with this program.
    pub fn bind_uniform_block_value_by_index(
        &self,
        index: u32,
        value: &UniformBlockValue,
        buffer_store: Option<&BufferStore>,
    ) -> Result<(), Error> {
        let mount_point = match value {
            UniformBlockValue::BufferBase {
                buffer,
                mount_point,
            } => {
                match buffer_store {
                    Some(store) => {
                        store.register(buffer)?;
                    }
                    None => {
                        buffer.init(&self.gl)?;
                    }
                }

                buffer.bind_ubo(*mount_point)?;
                mount_point
            }
            UniformBlockValue::BufferRange {
                buffer,
                mount_point,
                offset,
                size,
            } => {
                match buffer_store {
                    Some(store) => {
                        store.register(buffer)?;
                    }
                    None => {
                        buffer.init(&self.gl)?;
                    }
                }

                buffer.bind_ubo_range(*mount_point, *offset, *size)?;
                mount_point
            }
        };

        self.mount_uniform_block_by_index(index, *mount_point)?;

        Ok(())
    }

    /// Binds uniform blocks.
    pub fn bind_uniform_blocks(
        &self,
        state: Option<&FrameState>,
        entity: Option<&dyn Entity>,
        geometry: Option<&dyn Geometry>,
        material: Option<&dyn StandardMaterial>,
    ) -> Result<(), Error> {
        for (binding, index) in self.uniform_block_indices.iter() {
            let value = match binding {
                UniformBlockBinding::FromGeometry(name) => {
                    geometry.and_then(|geometry| geometry.uniform_block_value(name))
                }
                UniformBlockBinding::FromMaterial(name) => {
                    material.and_then(|material| material.uniform_block_value(name))
                }
                UniformBlockBinding::FromEntity(name) => {
                    entity.and_then(|entity| entity.uniform_block_value(name))
                }
                UniformBlockBinding::Custom(_) => {
                    continue;
                }
            };

            let Some(value) = value else {
                warn!(
                    target: "Program",
                    "no value specified for uniform block {}",
                    binding.variable_name()
                );
                continue;
            };

            self.bind_uniform_block_value_by_index(
                *index,
                &value,
                state.map(|state| state.buffer_store()),
            )?;
        }

        Ok(())
    }

    /// Unbinds vertex array object.
    pub fn unbind_vertex_array_object(&self) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        let vao = self.vao.borrow_mut().take();
        if vao.is_some() {
            self.gl.bind_vertex_array(None);
        }
        Ok(())
    }

    /// Unbinds attributes.
    pub fn unbind_attributes(&self) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        let Some(unbinders) = self.attribute_unbinders.borrow_mut().take() else {
            return Ok(());
        };
        unbinders.into_iter().for_each(|unbinder| unbinder.unbind());

        Ok(())
    }

    /// Unbinds uniform.
    pub fn unbind_uniforms(&self) -> Result<(), Error> {
        if !self.is_using() {
            return Err(Error::ProgramUnused);
        };

        let Some(unbinders) = self.uniform_unbinders.borrow_mut().take() else {
            return Ok(());
        };
        unbinders.into_iter().for_each(|unbinder| unbinder.unbind());

        Ok(())
    }

    /// Returns attribute locations by a [`AttributeBinding`].
    pub fn attribute_locations(&self) -> &HashMap<AttributeBinding, u32> {
        &self.attribute_locations
    }

    /// Returns uniform locations by a [`UniformBinding`].
    pub fn uniform_locations(&self) -> &HashMap<UniformBinding, WebGlUniformLocation> {
        &self.uniform_locations
    }

    /// Returns uniform block index by a uniform block name.
    pub fn uniform_block_indices(&self) -> &HashMap<UniformBlockBinding, u32> {
        &self.uniform_block_indices
    }
}

/// A centralized program store for storing and caching compiled [`ShaderProvider`].
pub struct ProgramStore {
    gl: WebGl2RenderingContext,
    store: HashMap<String, Program>,

    replacement_regex: Regex,
    snippets: HashMap<String, String>,

    using: Rc<RefCell<Option<WebGlProgram>>>,
}

impl ProgramStore {
    /// Constructs a new program store.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_snippets(gl, [])
    }

    /// Constructs a new program store with GLSL code snippets.
    pub fn with_snippets<'a, I>(gl: WebGl2RenderingContext, snippets: I) -> Self
    where
        I: IntoIterator<Item = (Cow<'a, str>, Cow<'a, str>)>,
    {
        let snippets = snippets
            .into_iter()
            .map(|(name, snippet)| (name.into_owned(), snippet.into_owned()));

        Self {
            gl,
            store: HashMap::new(),

            replacement_regex: Regex::new(GLSL_REPLACEMENT_DERIVATIVE_REGEX).unwrap(),
            snippets: HashMap::from_iter(snippets),

            using: Rc::new(RefCell::new(None)),
        }
    }

    /// Returns GLSL code snippet by name.
    pub fn snippet(&self, name: &str) -> Option<&str> {
        match self.snippets.get(name) {
            Some(snippet) => Some(snippet.as_str()),
            None => None,
        }
    }

    /// Adds new GLSL code snippet with an unique name.
    /// Returns the old one if exists.
    pub fn add_snippet<N, S>(&mut self, name: N, snippet: S) -> Option<String>
    where
        N: Into<String>,
        S: Into<String>,
    {
        let name: String = name.into();
        let name = name.trim().to_string();
        self.snippets.insert(name, snippet.into())
    }

    /// Removes GLSL code snippet by name.
    pub fn remove_snippet(&mut self, name: &str) -> Option<String> {
        self.snippets.remove(name.trim())
    }

    /// Clears all code snippets.
    pub fn clear_snippets(&mut self) {
        self.snippets.clear();
    }

    fn replace_snippets<'a, 'b, S>(&self, source: &'b S, is_vertex: bool) -> String
    where
        S: ProgramSource + ?Sized,
    {
        let universal_defines = source.universal_defines();
        let (code, defines) = match is_vertex {
            true => (source.vertex_source(), source.vertex_defines()),
            false => (source.fragment_source(), source.fragment_defines()),
        };

        // evaluated output code length
        let mut evaluated_len = code.len();
        for define in universal_defines.iter().chain(defines.iter()) {
            evaluated_len +=
                define.name().len() + define.value().map(|value| value.len()).unwrap_or(0) + 10;
        }
        let mut output = String::with_capacity(evaluated_len);

        let mut appended_snippets = HashSet::new();
        for line in code.lines() {
            let Some(matched) = self
                .replacement_regex
                .captures(line)
                .and_then(|captures| captures.get(1))
            else {
                output.push_str(line);
                if !line.ends_with("\n") {
                    output.push('\n');
                }
                continue;
            };

            let name = matched.as_str().trim();
            if appended_snippets.contains(name) {
                continue;
            }

            if name == GLSL_REPLACEMENT_DEFINES {
                for define in universal_defines.iter().chain(defines.iter()) {
                    output.push_str(&define.build());
                    output.push('\n');
                }
            } else {
                // finds snippet, finds from source first, finds from store otherwise
                let Some(snippet) = source.snippet(name).or_else(|| {
                    self.snippets
                        .get(name)
                        .map(|snippet| Cow::Borrowed(snippet.as_str()))
                }) else {
                    warn!(
                        target: "ProgramStore",
                        "code snippet with name `{}` not found",
                        name
                    );
                    continue;
                };

                output.push_str(&snippet);
                output.push('\n');
            }
            appended_snippets.insert(name);
        }
        output
    }

    fn compile<'a, 'b, S>(&'a mut self, name: String, source: &'b S) -> Result<Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        let vertex_code = self.replace_snippets(source, true);
        let vertex_shader = compile_shader(&self.gl, true, &vertex_code)?;

        let fragment_code = self.replace_snippets(source, false);
        let fragment_shader: WebGlShader = compile_shader(&self.gl, false, &fragment_code)?;

        let program = create_program(&self.gl, &vertex_shader, &fragment_shader)?;
        let attribute_locations = collects_attributes(&self.gl, &program);
        let uniform_locations = collects_uniforms(&self.gl, &program);
        let uniform_block_indices = collects_uniform_block_indices(&self.gl, &program);

        Ok(Program {
            gl: self.gl.clone(),
            name,

            program,
            vertex_shader,
            fragment_shader,

            attribute_locations: Rc::new(attribute_locations),
            uniform_locations: Rc::new(uniform_locations),
            uniform_block_indices: Rc::new(uniform_block_indices),

            using: Rc::clone(&self.using),
            vao: Rc::new(RefCell::new(None)),
            attribute_unbinders: Rc::new(RefCell::new(None)),
            uniform_unbinders: Rc::new(RefCell::new(None)),
        })
    }

    /// Uses a program from a program source.
    /// Program will be compiled if it is used for the first time.
    pub fn get_or_compile_program<'a, 'b, 'c, S>(
        &'a mut self,
        source: &'b S,
    ) -> Result<Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        let name = source.name();

        // checks cache
        if let Some(program) = self.store.get(name.as_ref()) {
            return Ok(program.clone());
        }

        let name = name.to_string();
        let program = self.compile(name.clone(), source)?;
        self.store.insert_unique_unchecked(name, program.clone());

        Ok(program)
    }

    /// Unuses and then deletes a cached program by unique name.
    pub fn delete_program(&mut self, name: &str) -> Result<(), Error> {
        let removed = self.store.remove(name);
        if let Some(program) = removed {
            program.unuse_program()?;
            delete_program(&self.gl, program);
        }

        Ok(())
    }
}

fn delete_program(gl: &WebGl2RenderingContext, program: Program) {
    let Program {
        program,
        vertex_shader,
        fragment_shader,
        ..
    } = program;
    gl.delete_shader(Some(&vertex_shader));
    gl.delete_shader(Some(&fragment_shader));
    gl.delete_program(Some(&program));
}

/// Compiles [`WebGlShader`] by [`ShaderSource`].
pub fn compile_shader(
    gl: &WebGl2RenderingContext,
    is_vertex: bool,
    code: &str,
) -> Result<WebGlShader, Error> {
    // log::info!("{}", code);
    let shader = gl
        .create_shader(if is_vertex {
            WebGl2RenderingContext::VERTEX_SHADER
        } else {
            WebGl2RenderingContext::FRAGMENT_SHADER
        })
        .ok_or(Error::CreateFragmentShaderFailure)?;

    // attaches shader source
    gl.shader_source(&shader, &code);
    // compiles shader
    gl.compile_shader(&shader);

    let success = gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap();
    if !success {
        let err = gl.get_shader_info_log(&shader).map(|err| err);
        gl.delete_shader(Some(&shader));
        Err(Error::CompileShaderFailure(err))
    } else {
        Ok(shader)
    }
}

/// Creates a [`WebGlProgram`], and links compiled [`WebGlShader`] to the program.
pub fn create_program(
    gl: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, Error> {
    let program = gl.create_program().ok_or(Error::CreateProgramFailure)?;

    // attaches shader to program
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    // links program
    gl.link_program(&program);
    // validates program
    gl.validate_program(&program);

    let success = gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap();
    if !success {
        let err = gl.get_program_info_log(&program).map(|err| err);
        gl.delete_program(Some(&program));
        return Err(Error::CompileProgramFailure(err));
    }

    Ok(program)
}

/// Collects active attribute locations and bindings.
pub fn collects_attributes(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<AttributeBinding, u32> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);
    for location in 0..num {
        let Some(info) = gl.get_active_attrib(&program, location) else {
            continue;
        };

        let name = info.name();

        locations.insert(AttributeBinding::from(&name), location);
    }

    locations
}

/// Collects active uniform locations and bindings.
pub fn collects_uniforms(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<UniformBinding, WebGlUniformLocation> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);
    for index in 0..num {
        let Some(info) = gl.get_active_uniform(&program, index) else {
            continue;
        };
        // if we have uniform block in code, getActiveUniform may return index of uniform inside uniform block,
        // while getUniformLocation can not get its location.
        let Some(location) = gl.get_uniform_location(&program, &info.name()) else {
            continue;
        };

        let name = info.name();

        locations.insert(UniformBinding::from(&name), location);
    }

    locations
}

/// Collects active uniform block indices.
pub fn collects_uniform_block_indices(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<UniformBlockBinding, u32> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_UNIFORM_BLOCKS)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);

    for location in 0..num {
        let Some(name) = gl.get_active_uniform_block_name(&program, location) else {
            continue;
        };

        locations.insert(UniformBlockBinding::from(name), location);
    }

    locations
}
