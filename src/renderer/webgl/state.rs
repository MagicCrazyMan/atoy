use std::{iter::FromIterator, ptr::NonNull};

use gl_matrix4rust::GLF32;
use log::warn;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture,
    WebGlUniformLocation,
};

use crate::{
    camera::Camera, entity::Entity, geometry::Geometry, material::webgl::StandardMaterial,
    readonly::Readonly,
};

use super::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferDescriptor, BufferStore, BufferTarget},
    capabilities::Capabilities,
    conversion::ToGlEnum,
    draw::Draw,
    error::Error,
    framebuffer::{
        AttachmentProvider, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment,
        FramebufferBuilder, FramebufferTarget, OperableBuffer, SizePolicy,
    },
    program::{Program, ProgramStore},
    texture::{
        texture2d::Texture2D, texture2darray::Texture2DArray, texture3d::Texture3D,
        texture_cubemap::TextureCubeMap, TextureDescriptor, TextureStore, TextureUnit,
    },
    uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
};

pub struct BoundAttribute {
    location: u32,
    descriptor: BufferDescriptor,
}

enum TextureKind {
    Texture2D(TextureDescriptor<Texture2D>),
    Texture2DArray(TextureDescriptor<Texture2DArray>),
    Texture3D(TextureDescriptor<Texture3D>),
    TextureCubeMap(TextureDescriptor<TextureCubeMap>),
}
pub struct BoundUniform {
    unit: TextureUnit,
    kind: TextureKind,
}

pub struct FrameState {
    timestamp: f64,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    camera: NonNull<(dyn Camera + 'static)>,

    program_store: NonNull<ProgramStore>,
    buffer_store: NonNull<BufferStore>,
    texture_store: NonNull<TextureStore>,
    capabilities: NonNull<Capabilities>,
}

impl FrameState {
    /// Constructs a new rendering state.
    pub(crate) fn new(
        timestamp: f64,
        camera: &mut (dyn Camera + 'static),
        gl: WebGl2RenderingContext,
        canvas: HtmlCanvasElement,
        program_store: &mut ProgramStore,
        buffer_store: &mut BufferStore,
        texture_store: &mut TextureStore,
        capabilities: &mut Capabilities,
    ) -> Self {
        unsafe {
            Self {
                timestamp,
                gl,
                canvas,
                camera: NonNull::new_unchecked(camera),
                program_store: NonNull::new_unchecked(program_store),
                buffer_store: NonNull::new_unchecked(buffer_store),
                texture_store: NonNull::new_unchecked(texture_store),
                capabilities: NonNull::new_unchecked(capabilities),
            }
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Returns the [`Camera`].
    #[inline]
    pub fn camera(&self) -> &dyn Camera {
        unsafe { self.camera.as_ref() }
    }

    /// Returns the [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn program_store(&self) -> &ProgramStore {
        unsafe { self.program_store.as_ref() }
    }

    /// Returns the mutable [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        unsafe { self.program_store.as_mut() }
    }

    /// Returns the [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn buffer_store(&self) -> &BufferStore {
        unsafe { self.buffer_store.as_ref() }
    }

    /// Returns the mutable [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        unsafe { self.buffer_store.as_mut() }
    }

    /// Returns the [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn texture_store(&self) -> &TextureStore {
        unsafe { self.texture_store.as_ref() }
    }

    /// Returns the mutable [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn texture_store_mut(&mut self) -> &mut TextureStore {
        unsafe { self.texture_store.as_mut() }
    }

    /// Returns the [`Capabilities`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn capabilities(&self) -> &Capabilities {
        unsafe { self.capabilities.as_ref() }
    }

    /// Binds attribute values from a entity, geometry and material.
    pub fn bind_attributes(
        &mut self,
        program: &mut Program,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn StandardMaterial,
    ) -> Result<Vec<BoundAttribute>, Error> {
        let attribute_bindings = material.attribute_bindings();
        let mut bounds = Vec::with_capacity(attribute_bindings.len());
        for binding in attribute_bindings {
            let Some(location) = program.attribute_locations().get(binding.variable_name()) else {
                warn!(
                    target: "BindAttributes",
                    "failed to get attribute location {}",
                    binding.variable_name()
                );
                continue;
            };

            let value = match binding {
                AttributeBinding::GeometryPosition => Some(geometry.positions()),
                AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                AttributeBinding::GeometryNormal => geometry.normals(),
                AttributeBinding::GeometryTangent => geometry.tangents(),
                AttributeBinding::GeometryBitangent => geometry.bitangents(),
                AttributeBinding::FromGeometry(name) => geometry.attribute_value(name.as_ref()),
                AttributeBinding::FromMaterial(name) => material.attribute_value(name.as_ref()),
                AttributeBinding::FromEntity(name) => entity.base().attribute_value(name.as_ref()),
            };
            let Some(value) = value else {
                warn!(
                    target: "BindAttributes",
                    "no value specified for attribute {}",
                    binding.variable_name()
                );
                continue;
            };

            match self.bind_attribute_value(*location, value.as_ref()) {
                Ok(ba) => bounds.extend(ba),
                Err(err) => warn!(
                    target: "BindUniforms",
                    "failed to bind attribute value {}",
                    err
                ),
            }
        }

        Ok(bounds)
    }

    /// Binds an [`AttributeValue`] to an attribute.
    pub fn bind_attribute_value(
        &mut self,
        location: u32,
        value: &AttributeValue,
    ) -> Result<Vec<BoundAttribute>, Error> {
        let mut bounds = Vec::new();
        match value {
            AttributeValue::Buffer {
                descriptor,
                target,
                component_size,
                data_type,
                normalized,
                bytes_stride,
                bytes_offset,
            } => {
                let buffer = self.buffer_store_mut().use_buffer(&descriptor, *target)?;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                self.gl.vertex_attrib_pointer_with_i32(
                    location,
                    *component_size as i32,
                    data_type.gl_enum(),
                    *normalized,
                    *bytes_stride as i32,
                    *bytes_offset as i32,
                );
                self.gl.enable_vertex_attrib_array(location);
                self.gl.bind_buffer(target.gl_enum(), None);

                bounds.push(BoundAttribute {
                    location,
                    descriptor: descriptor.clone(),
                });
            }
            AttributeValue::InstancedBuffer {
                descriptor,
                target,
                component_size,
                data_type,
                normalized,
                component_count_per_instance,
                divisor,
            } => {
                let buffer = self.buffer_store_mut().use_buffer(&descriptor, *target)?;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                let component_size = *component_size as usize;
                // binds each instance
                for i in 0..*component_count_per_instance {
                    let offset_location = location + i as u32;
                    let stride =
                        data_type.bytes_length() * component_size * component_count_per_instance;
                    let offset = i * data_type.bytes_length() * component_size;
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

                    bounds.push(BoundAttribute {
                        location: offset_location,
                        descriptor: descriptor.clone(),
                    });
                }
                self.gl.bind_buffer(target.gl_enum(), None);
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

        Ok(bounds)
    }

    pub fn bind_attribute_value_by_variable_name(
        &mut self,
        program: &mut Program,
        variable_name: &str,
        value: &AttributeValue,
    ) -> Result<Vec<BoundAttribute>, Error> {
        let Some(location) = program.attribute_locations().get(variable_name) else {
            return Err(Error::NoSuchAttribute(variable_name.to_string()));
        };
        self.bind_attribute_value(*location, value)
    }

    /// Unbinds all attributes.
    ///
    /// If you bind buffer attributes ever,
    /// remember to unbind them by yourself or use this function.
    pub fn unbind_attributes(&mut self, bounds: Vec<BoundAttribute>) {
        for BoundAttribute {
            location,
            descriptor,
        } in bounds
        {
            self.gl.disable_vertex_attrib_array(location);
            self.buffer_store_mut().unuse_buffer(&descriptor);
        }
    }

    /// Binds uniform values from a entity, geometry and material.
    pub fn bind_uniforms(
        &mut self,
        program: &mut Program,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn StandardMaterial,
    ) -> Result<Vec<BoundUniform>, Error> {
        let mut bounds = Vec::new();
        // binds uniforms
        let uniform_bindings = material.uniform_bindings();
        for binding in uniform_bindings {
            let Some(location) = program.uniform_locations().get(binding.variable_name()) else {
                warn!(
                    target: "BindUniforms",
                    "failed to get uniform location {}",
                    binding.variable_name()
                );
                continue;
            };

            let value = match binding {
                UniformBinding::ModelMatrix
                | UniformBinding::ViewMatrix
                | UniformBinding::ProjMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ViewProjMatrix => {
                    let data = match binding {
                        UniformBinding::ModelMatrix => entity.compose_model_matrix().gl_f32(),
                        UniformBinding::NormalMatrix => entity.compose_normal_matrix().gl_f32(),
                        UniformBinding::ViewMatrix => self.camera().view_matrix().gl_f32(),
                        UniformBinding::ProjMatrix => self.camera().proj_matrix().gl_f32(),
                        UniformBinding::ViewProjMatrix => self.camera().view_proj_matrix().gl_f32(),
                        _ => unreachable!(),
                    };

                    Some(Readonly::Owned(UniformValue::Matrix4 {
                        data,
                        transpose: false,
                    }))
                }
                UniformBinding::CameraPosition => Some(Readonly::Owned(
                    UniformValue::FloatVector3(self.camera().position().gl_f32()),
                )),
                UniformBinding::RenderTime => Some(Readonly::Owned(UniformValue::Float1(
                    self.timestamp() as f32,
                ))),
                UniformBinding::CanvasSize => {
                    Some(Readonly::Owned(UniformValue::UnsignedIntegerVector2([
                        self.canvas.width(),
                        self.canvas.height(),
                    ])))
                }
                UniformBinding::DrawingBufferSize => {
                    Some(Readonly::Owned(UniformValue::IntegerVector2([
                        self.gl.drawing_buffer_width(),
                        self.gl.drawing_buffer_width(),
                    ])))
                }
                UniformBinding::FromGeometry(name) => geometry.uniform_value(name.as_ref()),
                UniformBinding::FromMaterial(name) => material.uniform_value(name.as_ref()),
                UniformBinding::FromEntity(name) => entity.base().uniform_value(name.as_ref()),
            };
            let Some(value) = value else {
                warn!(
                    target: "BindUniforms",
                    "no value specified for uniform {}",
                    binding.variable_name()
                );
                continue;
            };

            match self.bind_uniform_value(&location, value.as_ref()) {
                Ok(bound) => {
                    if let Some(bound) = bound {
                        bounds.push(bound);
                    }
                }
                Err(err) => {
                    warn!(
                        target: "BindUniforms",
                        "failed to bind uniform value {}",
                        err
                    );
                }
            }
        }

        // binds uniform blocks
        let uniform_block_bindings = material.uniform_block_bindings();
        for binding in uniform_block_bindings {
            let Some(uniform_block_index) = program
                .uniform_block_indices()
                .get(binding.block_name())
                .cloned()
            else {
                continue;
            };

            let value = match binding {
                UniformBlockBinding::FromGeometry(name) => {
                    geometry.uniform_block_value(name.as_ref())
                }
                UniformBlockBinding::FromMaterial(name) => {
                    material.uniform_block_value(name.as_ref())
                }
                UniformBlockBinding::FromEntity(name) => {
                    entity.base().uniform_block_value(name.as_ref())
                }
            };
            let Some(value) = value else {
                warn!(
                    target: "BindUniforms",
                    "no value specified for uniform block {}",
                    binding.block_name()
                );
                continue;
            };

            self.bind_uniform_block_value(program.program(), uniform_block_index, value.as_ref())?;
        }

        Ok(bounds)
    }

    /// Binds a [`UniformValue`] to a uniform.
    pub fn bind_uniform_value(
        &mut self,
        location: &WebGlUniformLocation,
        value: &UniformValue,
    ) -> Result<Option<BoundUniform>, Error> {
        let bound = match value {
            UniformValue::Bool(v) => {
                if *v {
                    self.gl.uniform1i(Some(location), 1);
                } else {
                    self.gl.uniform1i(Some(location), 0);
                };
                None
            }
            UniformValue::UnsignedInteger1(x) => {
                self.gl.uniform1ui(Some(location), *x);
                None
            }
            UniformValue::UnsignedInteger2(x, y) => {
                self.gl.uniform2ui(Some(location), *x, *y);
                None
            }
            UniformValue::UnsignedInteger3(x, y, z) => {
                self.gl.uniform3ui(Some(location), *x, *y, *z);
                None
            }
            UniformValue::UnsignedInteger4(x, y, z, w) => {
                self.gl.uniform4ui(Some(location), *x, *y, *z, *w);
                None
            }
            UniformValue::Float1(x) => {
                self.gl.uniform1f(Some(location), *x);
                None
            }
            UniformValue::Float2(x, y) => {
                self.gl.uniform2f(Some(location), *x, *y);
                None
            }
            UniformValue::Float3(x, y, z) => {
                self.gl.uniform3f(Some(location), *x, *y, *z);
                None
            }
            UniformValue::Float4(x, y, z, w) => {
                self.gl.uniform4f(Some(location), *x, *y, *z, *w);
                None
            }
            UniformValue::Integer1(x) => {
                self.gl.uniform1i(Some(location), *x);
                None
            }
            UniformValue::Integer2(x, y) => {
                self.gl.uniform2i(Some(location), *x, *y);
                None
            }
            UniformValue::Integer3(x, y, z) => {
                self.gl.uniform3i(Some(location), *x, *y, *z);
                None
            }
            UniformValue::Integer4(x, y, z, w) => {
                self.gl.uniform4i(Some(location), *x, *y, *z, *w);
                None
            }
            UniformValue::FloatVector1(data) => {
                self.gl.uniform1fv_with_f32_array(Some(location), data);
                None
            }
            UniformValue::FloatVector2(data) => {
                self.gl.uniform2fv_with_f32_array(Some(location), data);
                None
            }
            UniformValue::FloatVector3(data) => {
                self.gl.uniform3fv_with_f32_array(Some(location), data);
                None
            }
            UniformValue::FloatVector4(data) => {
                self.gl.uniform4fv_with_f32_array(Some(location), data);
                None
            }
            UniformValue::IntegerVector1(data) => {
                self.gl.uniform1iv_with_i32_array(Some(location), data);
                None
            }
            UniformValue::IntegerVector2(data) => {
                self.gl.uniform2iv_with_i32_array(Some(location), data);
                None
            }
            UniformValue::IntegerVector3(data) => {
                self.gl.uniform3iv_with_i32_array(Some(location), data);
                None
            }
            UniformValue::IntegerVector4(data) => {
                self.gl.uniform4iv_with_i32_array(Some(location), data);
                None
            }
            UniformValue::UnsignedIntegerVector1(data) => {
                self.gl.uniform1uiv_with_u32_array(Some(location), data);
                None
            }
            UniformValue::UnsignedIntegerVector2(data) => {
                self.gl.uniform2uiv_with_u32_array(Some(location), data);
                None
            }
            UniformValue::UnsignedIntegerVector3(data) => {
                self.gl.uniform3uiv_with_u32_array(Some(location), data);
                None
            }
            UniformValue::UnsignedIntegerVector4(data) => {
                self.gl.uniform4uiv_with_u32_array(Some(location), data);
                None
            }
            UniformValue::Matrix2 { data, transpose } => {
                self.gl
                    .uniform_matrix2fv_with_f32_array(Some(location), *transpose, data);
                None
            }
            UniformValue::Matrix3 { data, transpose } => {
                self.gl
                    .uniform_matrix3fv_with_f32_array(Some(location), *transpose, data);
                None
            }
            UniformValue::Matrix4 { data, transpose } => {
                self.gl
                    .uniform_matrix4fv_with_f32_array(Some(location), *transpose, data);
                None
            }
            UniformValue::Texture2D { .. }
            | UniformValue::Texture2DArray { .. }
            | UniformValue::Texture3D { .. }
            | UniformValue::TextureCubeMap { .. } => {
                let (kind, unit) = match value {
                    UniformValue::Texture2D { descriptor, unit } => {
                        self.texture_store_mut().bind_texture(descriptor, *unit)?;
                        (TextureKind::Texture2D(descriptor.clone()), *unit)
                    }
                    UniformValue::Texture2DArray { descriptor, unit } => {
                        self.texture_store_mut().bind_texture(descriptor, *unit)?;
                        (TextureKind::Texture2DArray(descriptor.clone()), *unit)
                    }
                    UniformValue::Texture3D { descriptor, unit } => {
                        self.texture_store_mut().bind_texture(descriptor, *unit)?;
                        (TextureKind::Texture3D(descriptor.clone()), *unit)
                    }
                    UniformValue::TextureCubeMap { descriptor, unit } => {
                        self.texture_store_mut().bind_texture(descriptor, *unit)?;
                        (TextureKind::TextureCubeMap(descriptor.clone()), *unit)
                    }
                    // UniformValue::TextureCubeMap {
                    //     descriptor,
                    //     unit,
                    // } => (
                    //     TextureKind::TextureCubeMap(descriptor.clone()),
                    //     WebGl2RenderingContext::TEXTURE_CUBE_MAP,
                    //     self.texture_store_mut().use_texture(&descriptor, *unit)?,
                    //     unit,
                    // ),
                    // UniformValue::TextureCubeMapCompressed {
                    //     descriptor,
                    //     unit,
                    // } => (
                    //     TextureKind::TextureCubeMapCompressed(descriptor.clone()),
                    //     WebGl2RenderingContext::TEXTURE_CUBE_MAP,
                    //     self.texture_store_mut().use_texture(&descriptor, *unit)?,
                    //     unit,
                    // ),
                    _ => unreachable!(),
                };

                self.gl.uniform1i(Some(location), unit.unit_index() as i32);

                Some(BoundUniform { unit, kind })
            }
        };

        Ok(bound)
    }

    /// Unbinds all uniforms.
    ///
    /// If you bind buffer uniforms ever,
    /// remember to unbind them by yourself or use this function.
    pub fn unbind_uniforms(&mut self, bounds: Vec<BoundUniform>) -> Result<(), Error> {
        for BoundUniform {
            unit,
            kind: texture,
        } in bounds
        {
            match texture {
                TextureKind::Texture2D(descriptor) => {
                    self.texture_store_mut().unbind_texture(&descriptor, unit)?;
                }
                TextureKind::Texture2DArray(descriptor) => {
                    self.texture_store_mut().unbind_texture(&descriptor, unit)?;
                }
                TextureKind::Texture3D(descriptor) => {
                    self.texture_store_mut().unbind_texture(&descriptor, unit)?;
                }
                TextureKind::TextureCubeMap(descriptor) => {
                    self.texture_store_mut().unbind_texture(&descriptor, unit)?;
                }
            }
        }

        Ok(())
    }

    /// Binds a [`UniformValue`] to a uniform by variable name.
    pub fn bind_uniform_value_by_variable_name(
        &mut self,
        program: &mut Program,
        variable_name: &str,
        value: &UniformValue,
    ) -> Result<Option<BoundUniform>, Error> {
        let Some(location) = program.uniform_locations().get(variable_name) else {
            return Err(Error::NoSuchUniform(variable_name.to_string()));
        };
        self.bind_uniform_value(&location, value)
    }

    /// Binds a [`UniformBlockValue`] to a uniform block.
    pub fn bind_uniform_block_value(
        &mut self,
        program: &WebGlProgram,
        uniform_block_index: u32,
        value: &UniformBlockValue,
    ) -> Result<(), Error> {
        let binding = match value {
            UniformBlockValue::BufferBase {
                descriptor,
                binding,
            } => {
                self.buffer_store_mut()
                    .bind_uniform_buffer_object(&descriptor, *binding, None)?;
                binding
            }
            UniformBlockValue::BufferRange {
                descriptor,
                binding,
                offset,
                size,
            } => {
                self.buffer_store_mut().bind_uniform_buffer_object(
                    &descriptor,
                    *binding,
                    Some((*offset, *size)),
                )?;
                binding
            }
        };

        self.gl
            .uniform_block_binding(program, uniform_block_index, *binding);
        Ok(())
    }

    /// Binds a [`UniformBlockValue`] to a uniform block by a block name.
    pub fn bind_uniform_block_value_by_block_name(
        &mut self,
        program: &mut Program,
        uniform_block_name: &str,
        value: &UniformBlockValue,
    ) -> Result<(), Error> {
        let Some(uniform_block_index) = program
            .uniform_block_indices()
            .get(uniform_block_name)
            .cloned()
        else {
            return Err(Error::NoSuchUniform(uniform_block_name.to_string()));
        };
        self.bind_uniform_block_value(program.program(), uniform_block_index, value)
    }

    pub fn draw(&mut self, draw: &Draw) -> Result<(), Error> {
        match draw {
            Draw::Arrays { mode, first, count } => {
                self.gl.draw_arrays(mode.gl_enum(), *first, *count)
            }
            Draw::Elements {
                mode,
                count,
                offset,
                indices,
                indices_data_type,
            } => {
                let buffer = self
                    .buffer_store_mut()
                    .use_buffer(&indices, BufferTarget::ELEMENT_ARRAY_BUFFER)?;

                self.gl
                    .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
                self.gl.draw_elements_with_i32(
                    mode.gl_enum(),
                    *count,
                    indices_data_type.gl_enum(),
                    *offset,
                );
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
                self.buffer_store_mut().unuse_buffer(&indices);
            }
        }

        Ok(())
    }

    pub fn create_framebuffer<P>(
        &self,
        size_policy: SizePolicy,
        providers: P,
        renderbuffer_samples: Option<i32>,
    ) -> Framebuffer
    where
        P: IntoIterator<Item = (FramebufferAttachment, AttachmentProvider)>,
    {
        Framebuffer::new(
            self.gl.clone(),
            size_policy,
            providers,
            renderbuffer_samples,
        )
    }

    pub fn create_framebuffer_with_builder(&self, builder: FramebufferBuilder) -> Framebuffer {
        builder.build(self.gl.clone())
    }

    /// Reads pixels from current binding framebuffer.
    pub fn read_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        dst_data: &Object,
        dst_offset: u32,
    ) -> Result<(), Error> {
        self.gl
            .read_pixels_with_array_buffer_view_and_dst_offset(
                x, y, width, height, format, type_, dst_data, dst_offset,
            )
            .or_else(|err| Err(Error::ReadPixelsFailure(err.as_string())))?;
        Ok(())
    }

    /// Applies computation using current binding framebuffer and program.
    pub fn do_computation<'a, I>(&self, textures: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (&'a WebGlTexture, TextureUnit)>,
    {
        let sampler = self.capabilities().computation_sampler()?;
        let mut texture_units = Vec::new();
        for (texture, texture_unit) in textures {
            self.gl.active_texture(texture_unit.gl_enum());
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            self.gl
                .bind_sampler(texture_unit.unit_index() as u32, Some(&sampler));
            texture_units.push(texture_unit);
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);

        for texture_unit in texture_units {
            self.gl.active_texture(texture_unit.gl_enum());
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            self.gl.bind_sampler(texture_unit.unit_index() as u32, None);
        }

        Ok(())
    }

    /// Blits between read [`Framebuffer`] and draw [`Framebuffer`].
    pub fn blit_framebuffers(
        &self,
        read_framebuffer: &mut Framebuffer,
        draw_framebuffer: &mut Framebuffer,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error> {
        draw_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        let dst_width = draw_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let dst_height = draw_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        read_framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        let src_width = read_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_height = read_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        self.gl.blit_framebuffer(
            0,
            0,
            src_width,
            src_height,
            0,
            0,
            dst_width,
            dst_height,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        read_framebuffer.unbind();
        draw_framebuffer.unbind();

        Ok(())
    }

    /// Blits between read [`Framebuffer`] and draw [`Framebuffer`].
    pub fn blit_framebuffers_with_buffers<I>(
        &self,
        read_framebuffer: &mut Framebuffer,
        read_buffer: OperableBuffer,
        draw_framebuffer: &mut Framebuffer,
        draw_buffers: I,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = OperableBuffer>,
    {
        draw_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_framebuffer.set_draw_buffers(draw_buffers)?;
        read_framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        read_framebuffer.set_read_buffer(read_buffer)?;
        let dst_width = draw_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let dst_height = draw_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_width = read_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_height = read_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        self.gl.blit_framebuffer(
            0,
            0,
            src_width,
            src_height,
            0,
            0,
            dst_width,
            dst_height,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        draw_framebuffer.unbind();
        read_framebuffer.unbind();

        Ok(())
    }

    /// Blits between read [`WebGlFramebuffer`](WebGlFramebuffer) and draw [`WebGlFramebuffer`](WebGlFramebuffer).
    pub fn blit_framebuffers_native<I1, I2>(
        &self,
        read_framebuffer: &WebGlFramebuffer,
        read_buffer: OperableBuffer,
        draw_framebuffer: &WebGlFramebuffer,
        draw_buffers: I1,
        reset_draw_buffers: I2,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error>
    where
        I1: IntoIterator<Item = OperableBuffer>,
        I2: IntoIterator<Item = OperableBuffer>,
    {
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::DRAW_FRAMEBUFFER,
            Some(draw_framebuffer),
        );
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            Some(read_framebuffer),
        );

        let draw_buffers = Array::from_iter(
            draw_buffers
                .into_iter()
                .map(|v| JsValue::from_f64(v.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        self.gl.read_buffer(read_buffer.gl_enum());

        self.gl.blit_framebuffer(
            src_x0,
            src_y0,
            src_x1,
            src_y1,
            dst_x0,
            dst_y0,
            dst_x1,
            dst_y1,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        let draw_buffers = Array::from_iter(
            reset_draw_buffers
                .into_iter()
                .map(|v| JsValue::from_f64(v.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);

        self.gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        self.gl.read_buffer(WebGl2RenderingContext::BACK);

        Ok(())
    }
}
