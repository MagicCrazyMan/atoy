use std::{iter::FromIterator, ptr::NonNull};

use gl_matrix4rust::GLF32;
use log::warn;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture,
    WebGlUniformLocation,
};

use crate::{camera::Camera, entity::Entity, geometry::Geometry, material::StandardMaterial};

use super::{
    abilities::Abilities,
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferDescriptor, BufferStore, BufferTarget},
    conversion::ToGlEnum,
    draw::Draw,
    error::Error,
    framebuffer::{
        AttachmentProvider, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment,
        FramebufferBuilder, FramebufferTarget, OperatableBuffer, SizePolicy,
    },
    program::{Program, ProgramStore},
    texture::{TextureParameter, TextureStore, TextureUnit},
    uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
};

pub struct BoundAttribute {
    location: u32,
    descriptor: BufferDescriptor,
}

pub struct FrameState {
    timestamp: f64,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    camera: NonNull<(dyn Camera + 'static)>,

    program_store: NonNull<ProgramStore>,
    buffer_store: NonNull<BufferStore>,
    texture_store: NonNull<TextureStore>,
    abilities: NonNull<Abilities>,
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
        abilities: &mut Abilities,
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
                abilities: NonNull::new_unchecked(abilities),
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

    /// Returns the [`Abilities`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    #[inline]
    pub fn abilities(&self) -> &Abilities {
        unsafe { self.abilities.as_ref() }
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
            let Some(location) =
                program.get_or_retrieve_attribute_locations(binding.variable_name())
            else {
                warn!(
                    target: "BindAttributes",
                    "failed to get attribute location {}",
                    binding.variable_name()
                );
                continue;
            };

            let value = match binding {
                AttributeBinding::GeometryPosition => geometry.positions(),
                AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                AttributeBinding::GeometryNormal => geometry.normals(),
                AttributeBinding::FromGeometry(name) => geometry.attribute_value(name.as_ref()),
                AttributeBinding::FromMaterial(name) => material.attribute_value(name.as_ref()),
                AttributeBinding::FromEntity(name) => {
                    entity.attribute_values().get(name.as_ref()).cloned()
                }
            };
            let Some(value) = value else {
                warn!(
                    target: "BindAttributes",
                    "no value specified for attribute {}",
                    binding.variable_name()
                );
                continue;
            };

            match self.bind_attribute_value(location, value) {
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
        value: AttributeValue,
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
                let buffer = self.buffer_store_mut().use_buffer(&descriptor, target)?;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                self.gl.vertex_attrib_pointer_with_i32(
                    location,
                    component_size as i32,
                    data_type.gl_enum(),
                    normalized,
                    bytes_stride as i32,
                    bytes_offset as i32,
                );
                self.gl.enable_vertex_attrib_array(location);
                self.gl.bind_buffer(target.gl_enum(), None);

                bounds.push(BoundAttribute {
                    location,
                    descriptor,
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
                let buffer = self.buffer_store_mut().use_buffer(&descriptor, target)?;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                let component_size = component_size as usize;
                // binds each instance
                for i in 0..component_count_per_instance {
                    let offset_location = location + i as u32;
                    let stride =
                        data_type.bytes_length() * component_size * component_count_per_instance;
                    let offset = i * data_type.bytes_length() * component_size;
                    self.gl.vertex_attrib_pointer_with_i32(
                        offset_location,
                        component_size as i32,
                        data_type.gl_enum(),
                        normalized,
                        stride as i32,
                        offset as i32,
                    );
                    self.gl.enable_vertex_attrib_array(offset_location);
                    self.gl
                        .vertex_attrib_divisor(offset_location, divisor as u32);

                    bounds.push(BoundAttribute {
                        location: offset_location,
                        descriptor: descriptor.clone(),
                    });
                }
                self.gl.bind_buffer(target.gl_enum(), None);
            }
            AttributeValue::Vertex1f(x) => self.gl.vertex_attrib1f(location, x),
            AttributeValue::Vertex2f(x, y) => self.gl.vertex_attrib2f(location, x, y),
            AttributeValue::Vertex3f(x, y, z) => self.gl.vertex_attrib3f(location, x, y, z),
            AttributeValue::Vertex4f(x, y, z, w) => self.gl.vertex_attrib4f(location, x, y, z, w),
            AttributeValue::Vertex1fv(v) => self.gl.vertex_attrib1fv_with_f32_array(location, &v),
            AttributeValue::Vertex2fv(v) => self.gl.vertex_attrib2fv_with_f32_array(location, &v),
            AttributeValue::Vertex3fv(v) => self.gl.vertex_attrib3fv_with_f32_array(location, &v),
            AttributeValue::Vertex4fv(v) => self.gl.vertex_attrib4fv_with_f32_array(location, &v),
            AttributeValue::UnsignedInteger4(x, y, z, w) => {
                self.gl.vertex_attrib_i4ui(location, x, y, z, w)
            }
            AttributeValue::Integer4(x, y, z, w) => self.gl.vertex_attrib_i4i(location, x, y, z, w),
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
        value: AttributeValue,
    ) -> Result<Vec<BoundAttribute>, Error> {
        let Some(location) = program.get_or_retrieve_attribute_locations(variable_name) else {
            return Err(Error::NoSuchAttribute(variable_name.to_string()));
        };
        self.bind_attribute_value(location, value)
    }

    /// Unbinds all attributes after draw calls.
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
    ) -> Result<(), Error> {
        // binds uniforms
        let uniform_bindings = material.uniform_bindings();
        for binding in uniform_bindings {
            let Some(location) = program.get_or_retrieve_uniform_location(binding.variable_name())
            else {
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

                    Some(UniformValue::Matrix4 {
                        data,
                        transpose: false,
                    })
                }
                UniformBinding::CameraPosition => Some(UniformValue::FloatVector3(
                    self.camera().position().gl_f32(),
                )),
                UniformBinding::RenderTime => Some(UniformValue::Float1(self.timestamp() as f32)),
                UniformBinding::CanvasSize => Some(UniformValue::UnsignedIntegerVector2([
                    self.canvas.width(),
                    self.canvas.height(),
                ])),
                UniformBinding::DrawingBufferSize => Some(UniformValue::IntegerVector2([
                    self.gl.drawing_buffer_width(),
                    self.gl.drawing_buffer_width(),
                ])),
                UniformBinding::FromGeometry(name) => geometry.uniform_value(name.as_ref()),
                UniformBinding::FromMaterial(name) => material.uniform_value(name.as_ref()),
                UniformBinding::FromEntity(name) => {
                    entity.uniform_values().get(name.as_ref()).cloned()
                }
            };
            let Some(value) = value else {
                warn!(
                    target: "BindUniforms",
                    "no value specified for uniform {}",
                    binding.variable_name()
                );
                continue;
            };

            if let Err(err) = self.bind_uniform_value(&location, value) {
                warn!(
                    target: "BindUniforms",
                    "failed to bind uniform value {}",
                    err
                );
            };
        }

        // binds uniform blocks
        let uniform_block_bindings = material.uniform_block_bindings();
        for binding in uniform_block_bindings {
            let uniform_block_index =
                program.get_or_retrieve_uniform_block_index(binding.block_name());

            let value = match binding {
                UniformBlockBinding::FromGeometry(name) => {
                    geometry.uniform_block_value(name.as_ref())
                }
                UniformBlockBinding::FromMaterial(name) => {
                    material.uniform_block_value(name.as_ref())
                }
                UniformBlockBinding::FromEntity(name) => {
                    entity.uniform_blocks_values().get(name.as_ref()).cloned()
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

            self.bind_uniform_block_value(program.program(), uniform_block_index, value)?;
        }

        Ok(())
    }

    /// Binds a [`UniformValue`] to a uniform.
    pub fn bind_uniform_value(
        &mut self,
        location: &WebGlUniformLocation,
        value: UniformValue,
    ) -> Result<(), Error> {
        // let location = s
        match value {
            UniformValue::Bool(v) => {
                if v {
                    self.gl.uniform1i(Some(location), 1)
                } else {
                    self.gl.uniform1i(Some(location), 0)
                }
            }
            UniformValue::UnsignedInteger1(x) => self.gl.uniform1ui(Some(location), x),
            UniformValue::UnsignedInteger2(x, y) => self.gl.uniform2ui(Some(location), x, y),
            UniformValue::UnsignedInteger3(x, y, z) => self.gl.uniform3ui(Some(location), x, y, z),
            UniformValue::UnsignedInteger4(x, y, z, w) => {
                self.gl.uniform4ui(Some(location), x, y, z, w)
            }
            UniformValue::Float1(x) => self.gl.uniform1f(Some(location), x),
            UniformValue::Float2(x, y) => self.gl.uniform2f(Some(location), x, y),
            UniformValue::Float3(x, y, z) => self.gl.uniform3f(Some(location), x, y, z),
            UniformValue::Float4(x, y, z, w) => self.gl.uniform4f(Some(location), x, y, z, w),
            UniformValue::Integer1(x) => self.gl.uniform1i(Some(location), x),
            UniformValue::Integer2(x, y) => self.gl.uniform2i(Some(location), x, y),
            UniformValue::Integer3(x, y, z) => self.gl.uniform3i(Some(location), x, y, z),
            UniformValue::Integer4(x, y, z, w) => self.gl.uniform4i(Some(location), x, y, z, w),
            UniformValue::FloatVector1(data) => {
                self.gl.uniform1fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector2(data) => {
                self.gl.uniform2fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector3(data) => {
                self.gl.uniform3fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector4(data) => {
                self.gl.uniform4fv_with_f32_array(Some(location), &data)
            }
            UniformValue::IntegerVector1(data) => {
                self.gl.uniform1iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector2(data) => {
                self.gl.uniform2iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector3(data) => {
                self.gl.uniform3iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector4(data) => {
                self.gl.uniform4iv_with_i32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector1(data) => {
                self.gl.uniform1uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector2(data) => {
                self.gl.uniform2uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector3(data) => {
                self.gl.uniform3uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector4(data) => {
                self.gl.uniform4uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::Matrix2 { data, transpose } => {
                self.gl
                    .uniform_matrix2fv_with_f32_array(Some(location), transpose, &data)
            }
            UniformValue::Matrix3 { data, transpose } => {
                self.gl
                    .uniform_matrix3fv_with_f32_array(Some(location), transpose, &data)
            }
            UniformValue::Matrix4 { data, transpose } => {
                self.gl
                    .uniform_matrix4fv_with_f32_array(Some(location), transpose, &data)
            }
            UniformValue::Texture2D { .. } | UniformValue::Texture3D { .. } => {
                let (target, texture, unit, params) = match value {
                    UniformValue::Texture2D {
                        descriptor,
                        unit,
                        params,
                    } => (
                        WebGl2RenderingContext::TEXTURE_2D,
                        self.texture_store_mut().use_texture_2d(&descriptor, unit)?,
                        unit,
                        params,
                    ),
                    UniformValue::Texture3D {
                        descriptor,
                        unit,
                        params,
                    } => (
                        WebGl2RenderingContext::TEXTURE_3D,
                        self.texture_store_mut().use_texture_3d(&descriptor, unit)?,
                        unit,
                        params,
                    ),
                    _ => unreachable!(),
                };

                self.gl.uniform1i(Some(location), unit.unit_index() as i32);
                self.gl.active_texture(unit.gl_enum());
                self.gl.bind_texture(target, Some(&texture));
                params.iter().for_each(|param| match param {
                    TextureParameter::MAG_FILTER(v) => {
                        self.gl
                            .tex_parameteri(target, param.gl_enum(), v.gl_enum() as i32)
                    }
                    TextureParameter::MIN_FILTER(v) => {
                        self.gl
                            .tex_parameteri(target, param.gl_enum(), v.gl_enum() as i32)
                    }
                    TextureParameter::WRAP_S(v)
                    | TextureParameter::WRAP_T(v)
                    | TextureParameter::WRAP_R(v) => {
                        self.gl
                            .tex_parameteri(target, param.gl_enum(), v.gl_enum() as i32)
                    }
                    TextureParameter::COMPARE_FUNC(v) => {
                        self.gl
                            .tex_parameteri(target, param.gl_enum(), v.gl_enum() as i32)
                    }
                    TextureParameter::COMPARE_MODE(v) => {
                        self.gl
                            .tex_parameteri(target, param.gl_enum(), v.gl_enum() as i32)
                    }
                    TextureParameter::BASE_LEVEL(v) | TextureParameter::MAX_LEVEL(v) => {
                        self.gl.tex_parameteri(target, param.gl_enum(), *v)
                    }
                    TextureParameter::MAX_LOD(v) | TextureParameter::MIN_LOD(v) => {
                        self.gl.tex_parameterf(target, param.gl_enum(), *v)
                    }
                });
            }
        };
        Ok(())
    }

    /// Binds a [`UniformValue`] to a uniform by variable name.
    pub fn bind_uniform_value_by_variable_name(
        &mut self,
        program: &mut Program,
        variable_name: &str,
        value: UniformValue,
    ) -> Result<(), Error> {
        let Some(location) = program.get_or_retrieve_uniform_location(variable_name) else {
            return Err(Error::NoSuchUniform(variable_name.to_string()));
        };
        self.bind_uniform_value(&location, value)
    }

    /// Binds a [`UniformBlockValue`] to a uniform block.
    pub fn bind_uniform_block_value(
        &mut self,
        program: &WebGlProgram,
        uniform_block_index: u32,
        value: UniformBlockValue,
    ) -> Result<(), Error> {
        let binding = match value {
            UniformBlockValue::BufferBase {
                descriptor,
                binding,
            } => {
                self.buffer_store_mut()
                    .bind_uniform_buffer_object(&descriptor, binding, None)?;
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
                    binding,
                    Some((offset, size)),
                )?;
                binding
            }
        };

        self.gl
            .uniform_block_binding(program, uniform_block_index, binding);
        Ok(())
    }

    /// Binds a [`UniformBlockValue`] to a uniform block by a block name.
    pub fn bind_uniform_block_value_by_block_name(
        &mut self,
        program: &mut Program,
        uniform_block_name: &str,
        value: UniformBlockValue,
    ) -> Result<(), Error> {
        let uniform_block_index = program.get_or_retrieve_uniform_block_index(uniform_block_name);
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
                element_type,
                offset,
                indices,
            } => {
                let buffer = self
                    .buffer_store_mut()
                    .use_buffer(&indices, BufferTarget::ELEMENT_ARRAY_BUFFER)?;

                self.gl
                    .bind_buffer(BufferTarget::ELEMENT_ARRAY_BUFFER.gl_enum(), Some(&buffer));
                self.gl.draw_elements_with_i32(
                    mode.gl_enum(),
                    *count,
                    element_type.gl_enum(),
                    *offset,
                );
                self.gl
                    .bind_buffer(BufferTarget::ELEMENT_ARRAY_BUFFER.gl_enum(), None);
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
    pub fn do_computation<'a, I>(&mut self, textures: I)
    where
        I: IntoIterator<Item = (&'a WebGlTexture, TextureUnit)>,
    {
        let mut texture_units = Vec::new();
        for (texture, texture_unit) in textures {
            self.gl.active_texture(texture_unit.gl_enum());
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                0,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            texture_units.push(texture_unit);
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);

        for texture_unit in texture_units {
            self.gl.active_texture(texture_unit.gl_enum());
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::LINEAR as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::REPEAT as i32,
            );
            self.gl.tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::REPEAT as i32,
            );
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        }
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
        read_buffer: OperatableBuffer,
        draw_framebuffer: &mut Framebuffer,
        draw_buffers: I,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = OperatableBuffer>,
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
        read_buffer: OperatableBuffer,
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
        I1: IntoIterator<Item = OperatableBuffer>,
        I2: IntoIterator<Item = OperatableBuffer>,
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
