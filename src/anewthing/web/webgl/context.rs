use std::ops::{Range, RangeBounds};

use hashbrown::{hash_map::Entry, HashMap};
use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture, WebGlUniformLocation,
};

use crate::anewthing::channel::Channel;

use super::{
    attribute::WebGlAttributeValue,
    buffer::{
        WebGlBufferItem, WebGlBufferManager, WebGlBufferTarget, WebGlBufferUsage, WebGlBuffering,
    },
    capabilities::WebGlCapabilities,
    client_wait::WebGlClientWait,
    error::Error,
    framebuffer::{
        WebGlFramebufferAttachTarget, WebGlFramebufferCreateOptions, WebGlFramebufferFactory,
        WebGlFramebufferItem, WebGlFramebufferTarget,
    },
    pixel::{self, WebGlPixelDataType, WebGlPixelFormat, WebGlPixelPackStores},
    program::{WebGlProgramItem, WebGlProgramManager, WebGlShaderSource},
    texture::{
        WebGlTextureItem, WebGlTextureLayout, WebGlTextureManager, WebGlTextureUnit, WebGlTexturing,
    },
    uniform::{WebGlUniformBlockValue, WebGlUniformValue},
};

pub struct WebGlContext {
    gl: WebGl2RenderingContext,
    channel: Channel,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,
    texture_manager: WebGlTextureManager,
    framebuffer_factory: WebGlFramebufferFactory,
    capabilities: WebGlCapabilities,

    using_draw_framebuffer_item: Option<WebGlFramebufferItem>,
    using_program_item: Option<WebGlProgramItem>,
    using_ubos: HashMap<usize, (WebGlBuffer, Option<(usize, usize)>)>,
    activating_texture_unit: WebGlTextureUnit,
    using_textures: HashMap<(WebGlTextureUnit, WebGlTextureLayout), (WebGlTexture, WebGlSampler)>,
}

impl WebGlContext {
    /// Constructs a new WebGl drawing context.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            program_manager: WebGlProgramManager::new(gl.clone()),
            buffer_manager: WebGlBufferManager::new(gl.clone(), channel.clone()),
            texture_manager: WebGlTextureManager::new(gl.clone(), channel.clone()),
            framebuffer_factory: WebGlFramebufferFactory::new(gl.clone()),
            capabilities: WebGlCapabilities::new(gl.clone()),
            gl,
            channel,

            using_draw_framebuffer_item: None,
            using_program_item: None,
            using_ubos: HashMap::new(),
            activating_texture_unit: WebGlTextureUnit::Texture0,
            using_textures: HashMap::new(),
        }
    }

    /// Returns native [`WebGl2RenderingContext`].
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Returns associated message channel.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Returns [`WebGlProgramManager`].
    pub fn program_manager(&self) -> &WebGlProgramManager {
        &self.program_manager
    }

    /// Returns mutable [`WebGlProgramManager`].
    pub fn program_manager_mut(&mut self) -> &mut WebGlProgramManager {
        &mut self.program_manager
    }

    /// Returns [`WebGlBufferManager`].
    pub fn buffer_manager(&self) -> &WebGlBufferManager {
        &self.buffer_manager
    }

    /// Returns mutable [`WebGlBufferManager`].
    pub fn buffer_manager_mut(&mut self) -> &mut WebGlBufferManager {
        &mut self.buffer_manager
    }

    /// Returns [`WebGlTextureManager`].
    pub fn texture_manager(&self) -> &WebGlTextureManager {
        &self.texture_manager
    }

    /// Returns mutable [`WebGlTextureManager`].
    pub fn texture_manager_mut(&mut self) -> &mut WebGlTextureManager {
        &mut self.texture_manager
    }

    /// Returns [`WebGlCapabilities`].
    pub fn capabilities(&self) -> &WebGlCapabilities {
        &self.capabilities
    }

    // /// Creates a new [`WebGlClientWait`].
    // pub fn create_client_wait(&self, wait_timeout: Duration) -> WebGlClientWait {
    //     WebGlClientWait::new(wait_timeout)
    // }

    // /// Creates a new [`WebGlClientWait`] with retries.
    // pub fn create_client_wait_with_retries(
    //     &self,
    //     wait_timeout: Duration,
    //     retry_interval: Duration,
    //     max_retries: usize,
    // ) -> WebGlClientWait {
    //     WebGlClientWait::with_retries(wait_timeout, retry_interval, max_retries)
    // }

    // /// Creates a new [`WebGlClientWait`] with flags.
    // pub fn create_client_wait_with_flags<I>(
    //     &self,
    //     wait_timeout: Duration,
    //     flags: I,
    // ) -> WebGlClientWait
    // where
    //     I: IntoIterator<Item = WebGlClientWaitFlag>,
    // {
    //     WebGlClientWait::with_flags(wait_timeout, flags)
    // }

    // /// Creates a new [`WebGlClientWait`] with flags and retries.
    // pub fn create_client_wait_with_flags_and_retries<I>(
    //     &self,
    //     wait_timeout: Duration,
    //     retry_interval: Duration,
    //     max_retries: usize,
    //     flags: I,
    // ) -> WebGlClientWait
    // where
    //     I: IntoIterator<Item = WebGlClientWaitFlag>,
    // {
    //     WebGlClientWait::with_flags_and_retries(wait_timeout, retry_interval, max_retries, flags)
    // }

    /// Manages a [`WebGlBuffering`] and syncs its queueing [`BufferData`](super::super::super::buffering::BufferData) into WebGl context.
    pub fn sync_buffering(&mut self, buffering: &WebGlBuffering) -> Result<WebGlBufferItem, Error> {
        self.buffer_manager.sync_buffering(buffering)
    }

    /// Manages a [`WebGlTexturing`] and syncs its queueing [`TextureData`](super::super::super::texturing::TextureData) into WebGl context.
    pub fn sync_texturing(
        &mut self,
        texturing: &WebGlTexturing,
    ) -> Result<WebGlTextureItem, Error> {
        self.texture_manager.sync_texturing(
            texturing,
            &mut self.buffer_manager,
            self.activating_texture_unit,
            &self.using_textures,
            &self.capabilities,
        )
    }

    /// Creates a new framebuffer item by a [`WebGlFramebufferCreateOptions`].
    pub fn create_framebuffer(
        &self,
        options: WebGlFramebufferCreateOptions,
    ) -> Result<WebGlFramebufferItem, Error> {
        self.framebuffer_factory.create_framebuffer(
            options,
            &self.using_draw_framebuffer_item,
            self.activating_texture_unit,
            &self.using_textures,
            &self.capabilities,
        )
    }

    /// Binds a [`WebGlFramebufferItem`] to draw framebuffer and enable all color attachments.
    pub fn bind_draw_framebuffer(&mut self, item: &mut WebGlFramebufferItem) -> Result<(), Error> {
        self.bind_draw_framebuffer_with_draw_buffers(item, ..)
    }

    /// Binds a [`WebGlFramebufferItem`] to draw framebuffer
    /// with specifying the color attachments are available to written to.
    pub fn bind_draw_framebuffer_with_draw_buffers<R>(
        &mut self,
        item: &mut WebGlFramebufferItem,
        draw_buffers_range: R,
    ) -> Result<(), Error>
    where
        R: RangeBounds<usize>,
    {
        self.framebuffer_factory.update_framebuffer(
            item,
            &self.using_draw_framebuffer_item,
            self.activating_texture_unit,
            &self.using_textures,
            &self.capabilities,
        )?;
        self.gl.bind_framebuffer(
            WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
            Some(item.gl_framebuffer()),
        );

        let draw_buffers = Array::new();
        for i in 0..item.color_attachment_len() {
            if draw_buffers_range.contains(&i) {
                draw_buffers.push(&JsValue::from_f64(
                    (WebGlFramebufferAttachTarget::ColorAttachment0.to_gl_enum() + i as u32) as f64,
                ));
            } else {
                draw_buffers.push(&JsValue::from_f64(WebGl2RenderingContext::NONE as f64));
            }
        }
        self.gl.draw_buffers(&draw_buffers);

        self.using_draw_framebuffer_item = Some(item.clone());
        Ok(())
    }

    // Unbinds draw buffer
    pub fn unbind_draw_framebuffer(&mut self) {
        if let Some(_) = self.using_draw_framebuffer_item.take() {
            self.gl
                .bind_framebuffer(WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(), None);
        }
    }

    /// Compiles shader sources and then uses the compiled program.
    /// Returns the compiled [`WebGlProgramItem`] as well.
    pub fn use_program_by_shader_sources<VS, FS>(
        &mut self,
        vertex: &VS,
        fragment: &FS,
    ) -> Result<WebGlProgramItem, Error>
    where
        VS: WebGlShaderSource,
        FS: WebGlShaderSource,
    {
        let program_item = self
            .program_manager
            .get_or_compile_program(vertex, fragment)?;
        Self::use_program_inner(&self.gl, &mut self.using_program_item, &program_item);
        Ok(program_item)
    }

    /// Uses a compiled [`WebGlProgramItem`] to this WebGl context.
    fn use_program_inner(
        gl: &WebGl2RenderingContext,
        using_program: &mut Option<WebGlProgramItem>,
        program_item: &WebGlProgramItem,
    ) {
        if using_program.as_ref().map(|i| i.gl_program()) == Some(program_item.gl_program()) {
            return;
        }

        gl.use_program(Some(program_item.gl_program()));
        *using_program = Some(program_item.clone());
    }

    /// Sets a attribute by specified attribute name.
    pub fn set_attribute_value(
        &mut self,
        name: &str,
        value: WebGlAttributeValue,
    ) -> Result<(), Error> {
        let Some(using_program) = self.using_program_item.as_ref() else {
            return Err(Error::NoUsingProgram);
        };
        let Some(location) = using_program.attribute_location(name) else {
            return Err(Error::AttributeLocationNotFound(name.to_string()));
        };
        Self::set_attribute_value_inner(&self.gl, &mut self.buffer_manager, location, value)?;
        Ok(())
    }

    fn set_attribute_value_inner(
        gl: &WebGl2RenderingContext,
        buffer_manager: &mut WebGlBufferManager,
        location: u32,
        value: WebGlAttributeValue,
    ) -> Result<(), Error> {
        match value {
            WebGlAttributeValue::ArrayBuffer {
                buffering,
                component_size,
                data_type,
                normalized,
                bytes_stride,
                bytes_offset,
            } => {
                let buffer_item = buffer_manager.sync_buffering(buffering)?;
                gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(buffer_item.gl_buffer()),
                );
                gl.vertex_attrib_pointer_with_i32(
                    location,
                    component_size as i32,
                    data_type.to_gl_enum(),
                    normalized,
                    bytes_stride as i32,
                    bytes_offset as i32,
                );
                gl.enable_vertex_attrib_array(location);
                gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);
            }
            WebGlAttributeValue::InstancedBuffer {
                buffering,
                component_size,
                instance_size,
                data_type,
                normalized,
                bytes_stride,
                bytes_offset,
            } => {
                let buffer_item = buffer_manager.sync_buffering(buffering)?;
                gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(buffer_item.gl_buffer()),
                );
                gl.vertex_attrib_pointer_with_i32(
                    location,
                    component_size as i32,
                    data_type.to_gl_enum(),
                    normalized,
                    bytes_stride as i32,
                    bytes_offset as i32,
                );
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_divisor(location, instance_size as u32);
                gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);
            }
            _ => {
                match value {
                    WebGlAttributeValue::Float1(x) => gl.vertex_attrib1f(location, x),
                    WebGlAttributeValue::Float2(x, y) => gl.vertex_attrib2f(location, x, y),
                    WebGlAttributeValue::Float3(x, y, z) => gl.vertex_attrib3f(location, x, y, z),
                    WebGlAttributeValue::Float4(x, y, z, w) => {
                        gl.vertex_attrib4f(location, x, y, z, w)
                    }
                    WebGlAttributeValue::Integer4(x, y, z, w) => {
                        gl.vertex_attrib_i4i(location, x, y, z, w)
                    }
                    WebGlAttributeValue::UnsignedInteger4(x, y, z, w) => {
                        gl.vertex_attrib_i4ui(location, x, y, z, w)
                    }
                    WebGlAttributeValue::FloatVector1(v) => {
                        gl.vertex_attrib1fv_with_f32_array(location, v.data.as_slice())
                    }
                    WebGlAttributeValue::FloatVector2(v) => {
                        gl.vertex_attrib2fv_with_f32_array(location, v.data.as_slice())
                    }
                    WebGlAttributeValue::FloatVector3(v) => {
                        gl.vertex_attrib3fv_with_f32_array(location, v.data.as_slice())
                    }
                    WebGlAttributeValue::FloatVector4(v) => {
                        gl.vertex_attrib4fv_with_f32_array(location, v.data.as_slice())
                    }
                    WebGlAttributeValue::IntegerVector4(v) => {
                        gl.vertex_attrib_i4i(location, v.x, v.y, v.z, v.w)
                    }
                    WebGlAttributeValue::UnsignedIntegerVector4(v) => {
                        gl.vertex_attrib_i4ui(location, v.x, v.y, v.z, v.w)
                    }
                    _ => unreachable!(),
                };
                gl.disable_vertex_attrib_array(location);
            }
        };

        Ok(())
    }

    /// Sets a uniform value by specified uniform name.
    pub fn set_uniform_value(&mut self, name: &str, value: WebGlUniformValue) -> Result<(), Error> {
        let Some(using_program) = self.using_program_item.as_ref() else {
            return Err(Error::NoUsingProgram);
        };
        let Some(location) = using_program.uniform_location(name) else {
            return Err(Error::UniformLocationNotFound(name.to_string()));
        };
        Self::set_uniform_value_inner(
            &self.gl,
            &location,
            value,
            &mut self.activating_texture_unit,
            &mut self.using_textures,
            &mut self.texture_manager,
            &mut self.buffer_manager,
            &self.capabilities,
        )?;
        Ok(())
    }

    fn set_uniform_value_inner(
        gl: &WebGl2RenderingContext,
        location: &WebGlUniformLocation,
        value: WebGlUniformValue,
        activating_texture_unit: &mut WebGlTextureUnit,
        using_textures: &mut HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        texture_manager: &mut WebGlTextureManager,
        buffer_manager: &mut WebGlBufferManager,
        capabilities: &WebGlCapabilities,
    ) -> Result<(), Error> {
        match value {
            WebGlUniformValue::Bool(v) => gl.uniform1i(Some(location), if v { 1 } else { 0 }),
            WebGlUniformValue::Texture { texturing, unit } => {
                Self::bind_texture_inner(
                    gl,
                    texturing,
                    unit,
                    activating_texture_unit,
                    using_textures,
                    texture_manager,
                    buffer_manager,
                    capabilities,
                )?;
                gl.uniform1i(Some(location), unit.as_index());
            }
            WebGlUniformValue::Float1(x) => gl.uniform1f(Some(location), x),
            WebGlUniformValue::Float2(x, y) => gl.uniform2f(Some(location), x, y),
            WebGlUniformValue::Float3(x, y, z) => gl.uniform3f(Some(location), x, y, z),
            WebGlUniformValue::Float4(x, y, z, w) => gl.uniform4f(Some(location), x, y, z, w),
            WebGlUniformValue::UnsignedInteger1(x) => gl.uniform1ui(Some(location), x),
            WebGlUniformValue::UnsignedInteger2(x, y) => gl.uniform2ui(Some(location), x, y),
            WebGlUniformValue::UnsignedInteger3(x, y, z) => gl.uniform3ui(Some(location), x, y, z),
            WebGlUniformValue::UnsignedInteger4(x, y, z, w) => {
                gl.uniform4ui(Some(location), x, y, z, w)
            }
            WebGlUniformValue::Integer1(x) => gl.uniform1i(Some(location), x),
            WebGlUniformValue::Integer2(x, y) => gl.uniform2i(Some(location), x, y),
            WebGlUniformValue::Integer3(x, y, z) => gl.uniform3i(Some(location), x, y, z),
            WebGlUniformValue::Integer4(x, y, z, w) => gl.uniform4i(Some(location), x, y, z, w),
            WebGlUniformValue::FloatVector1(v) => {
                gl.uniform1fv_with_f32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::FloatVector2(v) => {
                gl.uniform2fv_with_f32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::FloatVector3(v) => {
                gl.uniform3fv_with_f32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::FloatVector4(v) => {
                gl.uniform4fv_with_f32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::IntegerVector1(v) => {
                gl.uniform1iv_with_i32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::IntegerVector2(v) => {
                gl.uniform2iv_with_i32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::IntegerVector3(v) => {
                gl.uniform3iv_with_i32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::IntegerVector4(v) => {
                gl.uniform4iv_with_i32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::UnsignedIntegerVector1(v) => {
                gl.uniform1uiv_with_u32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::UnsignedIntegerVector2(v) => {
                gl.uniform2uiv_with_u32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::UnsignedIntegerVector3(v) => {
                gl.uniform3uiv_with_u32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::UnsignedIntegerVector4(v) => {
                gl.uniform4uiv_with_u32_array(Some(location), v.data.as_slice())
            }
            WebGlUniformValue::Matrix2 { data, transpose } => {
                gl.uniform_matrix2fv_with_f32_array(Some(location), transpose, data.data.as_slice())
            }
            WebGlUniformValue::Matrix3 { data, transpose } => {
                gl.uniform_matrix3fv_with_f32_array(Some(location), transpose, data.data.as_slice())
            }
            WebGlUniformValue::Matrix4 { data, transpose } => {
                gl.uniform_matrix4fv_with_f32_array(Some(location), transpose, data.data.as_slice())
            }
            WebGlUniformValue::Matrix3x2 { data, transpose } => gl
                .uniform_matrix3x2fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
            WebGlUniformValue::Matrix4x2 { data, transpose } => gl
                .uniform_matrix4x2fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
            WebGlUniformValue::Matrix2x3 { data, transpose } => gl
                .uniform_matrix2x3fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
            WebGlUniformValue::Matrix4x3 { data, transpose } => gl
                .uniform_matrix4x3fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
            WebGlUniformValue::Matrix2x4 { data, transpose } => gl
                .uniform_matrix2x4fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
            WebGlUniformValue::Matrix3x4 { data, transpose } => gl
                .uniform_matrix3x4fv_with_f32_array(
                    Some(location),
                    transpose,
                    data.data.as_slice(),
                ),
        };

        Ok(())
    }

    /// Sets a uniform block value by specified uniform block name.
    pub fn set_uniform_block_value(
        &mut self,
        name: &str,
        value: WebGlUniformBlockValue,
    ) -> Result<(), Error> {
        let (buffer, mount_point, bytes_range) = match value {
            WebGlUniformBlockValue::Base {
                buffer,
                mount_point,
            } => (buffer, mount_point, None),
            WebGlUniformBlockValue::Range {
                buffer,
                mount_point,
                bytes_offset,
                bytes_length,
            } => (buffer, mount_point, Some((bytes_offset, bytes_length))),
        };

        let buffer_item = self.buffer_manager.sync_buffering(buffer)?;
        let Some(using_program) = self.using_program_item.as_ref() else {
            return Err(Error::NoUsingProgram);
        };
        let Some(location) = using_program.uniform_block_location(name) else {
            return Err(Error::UniformBlockLocationNotFound(name.to_string()));
        };

        Self::mount_uniform_buffer_object_inner(
            &self.gl,
            buffer_item.gl_buffer(),
            &mut self.using_ubos,
            mount_point,
            bytes_range,
        );

        Self::set_uniform_block_mount_point_inner(
            &self.gl,
            using_program,
            buffer_item.gl_buffer(),
            location,
            mount_point,
        );

        Ok(())
    }

    fn set_uniform_block_mount_point_inner(
        gl: &WebGl2RenderingContext,
        program_item: &WebGlProgramItem,
        gl_buffer: &WebGlBuffer,
        location: u32,
        mount_point: usize,
    ) {
        gl.bind_buffer(
            WebGlBufferTarget::UniformBuffer.to_gl_enum(),
            Some(gl_buffer),
        );
        gl.uniform_block_binding(program_item.gl_program(), location, mount_point as u32);
        gl.bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);
    }

    /// Binds and actives a texture from [`WebGlTexturing`] to specified texture unit.
    pub fn bind_texture(
        &mut self,
        texturing: &WebGlTexturing,
        unit: WebGlTextureUnit,
    ) -> Result<(), Error> {
        Self::bind_texture_inner(
            &self.gl,
            texturing,
            unit,
            &mut self.activating_texture_unit,
            &mut self.using_textures,
            &mut self.texture_manager,
            &mut self.buffer_manager,
            &self.capabilities,
        )
    }

    fn bind_texture_inner(
        gl: &WebGl2RenderingContext,
        texturing: &WebGlTexturing,
        unit: WebGlTextureUnit,
        activating_texture_unit: &mut WebGlTextureUnit,
        using_textures: &mut HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        texture_manager: &mut WebGlTextureManager,
        buffer_manager: &mut WebGlBufferManager,
        capabilities: &WebGlCapabilities,
    ) -> Result<(), Error> {
        let item = texture_manager.sync_texturing(
            texturing,
            buffer_manager,
            *activating_texture_unit,
            using_textures,
            capabilities,
        )?;
        let layout = item.layout().as_layout();

        match using_textures.entry((unit, layout)) {
            Entry::Occupied(mut e) => {
                let (t, s) = e.get_mut();
                if t != item.gl_texture() || s != item.gl_sampler() {
                    gl.active_texture(unit.to_gl_enum());
                    gl.bind_texture(layout.to_gl_enum(), Some(item.gl_texture()));
                    gl.bind_sampler(unit.as_index() as u32, Some(item.gl_sampler()));
                    e.replace_entry((item.gl_texture().clone(), item.gl_sampler().clone()));
                }
            }
            Entry::Vacant(e) => {
                gl.active_texture(unit.to_gl_enum());
                gl.bind_texture(layout.to_gl_enum(), Some(item.gl_texture()));
                gl.bind_sampler(unit.as_index() as u32, Some(item.gl_sampler()));
                e.insert((item.gl_texture().clone(), item.gl_sampler().clone()));
            }
        };
        *activating_texture_unit = unit;

        Ok(())
    }

    /// Unbinds a texture in specified texture unit.
    pub fn unbind_texture(&mut self, unit: WebGlTextureUnit, layout: WebGlTextureLayout) {
        let Some(_) = self.using_textures.remove(&(unit, layout)) else {
            return;
        };
        self.gl.active_texture(unit.to_gl_enum());
        self.gl.bind_texture(layout.to_gl_enum(), None);
        self.gl.bind_sampler(unit.as_index() as u32, None);
        self.activating_texture_unit = unit;
    }

    /// Binds a buffer to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object(&mut self, gl_buffer: &WebGlBuffer, mount_point: usize) {
        Self::mount_uniform_buffer_object_inner(
            &self.gl,
            gl_buffer,
            &mut self.using_ubos,
            mount_point,
            None,
        );
    }

    /// Binds a buffer range to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object_by_range<R>(
        &mut self,
        gl_buffer: &WebGlBuffer,
        mount_point: usize,
        src_bytes_offset: usize,
        src_bytes_length: usize,
    ) {
        Self::mount_uniform_buffer_object_inner(
            &self.gl,
            gl_buffer,
            &mut self.using_ubos,
            mount_point,
            Some((src_bytes_offset, src_bytes_length)),
        );
    }

    fn mount_uniform_buffer_object_inner(
        gl: &WebGl2RenderingContext,
        gl_buffer: &WebGlBuffer,
        using_ubos: &mut HashMap<usize, (WebGlBuffer, Option<(usize, usize)>)>,
        mount_point: usize,
        bytes_range: Option<(usize, usize)>,
    ) {
        if let Some((bound_gl_buffer, bound_bytes_range)) = using_ubos.get(&mount_point) {
            if bound_gl_buffer == gl_buffer && bound_bytes_range == &bytes_range {
                return;
            }
        }

        match bytes_range {
            Some((offset, length)) => gl.bind_buffer_range_with_i32_and_i32(
                WebGlBufferTarget::UniformBuffer.to_gl_enum(),
                mount_point as u32,
                Some(gl_buffer),
                offset as i32,
                length as i32,
            ),
            None => gl.bind_buffer_base(
                WebGlBufferTarget::UniformBuffer.to_gl_enum(),
                mount_point as u32,
                Some(gl_buffer),
            ),
        };
        gl.bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);

        using_ubos.insert(mount_point, (gl_buffer.clone(), bytes_range));
    }

    /// Reads buffer data into an [`Uint8Array`].
    pub fn read_buffer(&mut self, buffering: &WebGlBuffering) -> Result<Uint8Array, Error> {
        self.read_buffer_by_range(buffering, ..)
    }

    /// Reads buffer data into an [`Uint8Array`] with byte range.
    pub fn read_buffer_by_range<R>(
        &mut self,
        buffering: &WebGlBuffering,
        bytes_range: R,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let item = self.buffer_manager.sync_buffering(buffering)?;
        let bytes_range = item.normalize_bytes_range(bytes_range);
        Self::read_buffer_inner(&self.gl, item.gl_buffer(), bytes_range)
    }

    /// Reads buffer data into an [`Uint8Array`] asynchronously.
    pub async fn read_buffer_with_client_wait(
        &mut self,
        buffering: &WebGlBuffering,
        client_wait: &WebGlClientWait,
    ) -> Result<Uint8Array, Error> {
        self.read_buffer_by_range_with_client_wait(buffering, .., client_wait)
            .await
    }

    /// Reads buffer data into an [`Uint8Array`] with byte range asynchronously.
    pub async fn read_buffer_by_range_with_client_wait<R>(
        &mut self,
        buffering: &WebGlBuffering,
        bytes_range: R,
        client_wait: &WebGlClientWait,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffering(buffering)?;
        let tmp_gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.gl.bind_buffer(
            WebGlBufferTarget::CopyReadBuffer.to_gl_enum(),
            Some(buffer_item.gl_buffer()),
        );
        self.gl.bind_buffer(
            WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
            Some(&tmp_gl_buffer),
        );
        self.gl.buffer_data_with_i32(
            WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
            buffer_item.bytes_length() as i32,
            WebGlBufferUsage::StreamRead.to_gl_enum(),
        );
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            WebGlBufferTarget::CopyReadBuffer.to_gl_enum(),
            WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
            0,
            0,
            buffer_item.bytes_length() as i32,
        );
        self.gl
            .bind_buffer(WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(), None);
        self.gl
            .bind_buffer(WebGlBufferTarget::CopyReadBuffer.to_gl_enum(), None);

        client_wait.client_wait(&self.gl).await?;

        let bytes_range = buffer_item.normalize_bytes_range(bytes_range);
        let data = Self::read_buffer_inner(&self.gl, &tmp_gl_buffer, bytes_range)?;

        self.gl.delete_buffer(Some(&tmp_gl_buffer));

        Ok(data)
    }

    fn read_buffer_inner(
        gl: &WebGl2RenderingContext,
        gl_buffer: &WebGlBuffer,
        bytes_range: Range<usize>,
    ) -> Result<Uint8Array, Error> {
        let bytes_length = bytes_range.len();
        let dst = Uint8Array::new_with_length(bytes_length as u32);

        gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), Some(gl_buffer));
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view_and_dst_offset_and_length(
            WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
            bytes_range.start as i32,
            dst.as_ref(),
            0,
            bytes_length as u32,
        );
        gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);

        Ok(dst)
    }

    /// Reads pixels from framebuffer into a pixel buffer object and returns a native [`WebGlBuffer`].
    /// Calls [`read_buffer`](WebGlContext::read_buffer) or [`read_buffer_by_range`](WebGlContext::read_buffer_by_range)
    /// to get pixels back to client.
    ///
    /// - `framebuffer`: Reads pixels from back framebuffer if framebuffer is `None`.
    /// When providing a framebuffer, a read buffer index should be specified as well.
    /// - `x` and `y`: Uses `0` as default if not specified.
    /// - `width` and `height`: Uses framebuffer width and height if not specified.
    /// - `dst_bytes_offset`: applies no offset if not specified.
    pub fn read_pixels_pbo(
        &mut self,
        framebuffer: Option<(&WebGlFramebufferItem, usize)>,
        pixel_format: WebGlPixelFormat,
        pixel_data_type: WebGlPixelDataType,
        pixel_pack_stores: WebGlPixelPackStores,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_bytes_offset: Option<usize>,
    ) -> Result<WebGlBuffering, Error> {
        let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        match framebuffer {
            Some((framebuffer, read_buffer_index)) => {
                self.gl.bind_framebuffer(
                    WebGlFramebufferTarget::ReadFramebuffer.to_gl_enum(),
                    Some(framebuffer.gl_framebuffer()),
                );
                self.gl.read_buffer(
                    WebGlFramebufferAttachTarget::ColorAttachment0.to_gl_enum()
                        + read_buffer_index as u32,
                );
            }
            None => {
                self.gl.read_buffer(WebGl2RenderingContext::BACK);
            }
        };

        let x = x.unwrap_or(0);
        let y = y.unwrap_or(0);
        let width = match (framebuffer, width) {
            (None, None) => self.gl.drawing_buffer_width() as usize,
            (Some((framebuffer, _)), None) => framebuffer.current_width(),
            (_, Some(width)) => width,
        };
        let height = match (framebuffer, height) {
            (None, None) => self.gl.drawing_buffer_height() as usize,
            (Some((framebuffer, _)), None) => framebuffer.current_height(),
            (_, Some(height)) => height,
        };
        let dst_bytes_offset = dst_bytes_offset.unwrap_or(0);

        let bytes_length = pixel::bytes_length_of(
            pixel_format,
            pixel_data_type,
            pixel_pack_stores,
            width,
            height,
        );

        self.gl.bind_buffer(
            WebGlBufferTarget::PixelPackBuffer.to_gl_enum(),
            Some(&gl_buffer),
        );
        self.gl.buffer_data_with_i32(
            WebGlBufferTarget::PixelPackBuffer.to_gl_enum(),
            bytes_length as i32,
            WebGlBufferUsage::StaticRead.to_gl_enum(),
        );
        pixel_pack_stores.set_pixel_store(&self.gl);
        self.gl
            .read_pixels_with_i32(
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                pixel_format.to_gl_enum(),
                pixel_data_type.to_gl_enum(),
                dst_bytes_offset as i32,
            )
            .unwrap(); // no DomException thrown
        WebGlPixelPackStores::default().set_pixel_store(&self.gl);
        self.gl
            .bind_buffer(WebGlBufferTarget::PixelPackBuffer.to_gl_enum(), None);

        if framebuffer.is_some() {
            self.gl
                .bind_framebuffer(WebGlFramebufferTarget::ReadFramebuffer.to_gl_enum(), None);
        }

        let (buffering, _) = self
            .buffer_manager
            .wrap_gl_buffer(gl_buffer, Some(WebGlBufferUsage::StaticRead))?;
        Ok(buffering)
    }
}
