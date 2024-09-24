use std::ops::{Range, RangeBounds};

use hashbrown::HashMap;
use js_sys::Uint8Array;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlUniformLocation};

use crate::anewthing::channel::Channel;

use super::{
    attribute::WebGlAttributeValue,
    buffer::{
        WebGlBufferItem, WebGlBufferManager, WebGlBufferTarget, WebGlBufferUsage, WebGlBuffering,
    },
    capabilities::WebGlCapabilities,
    client_wait::WebGlClientWait,
    error::Error,
    program::{WebGlProgramItem, WebGlProgramManager, WebGlShaderSource},
    texture::{WebGlTextureItem, WebGlTextureManager, WebGlTextureUnit, WebGlTexturing},
    uniform::{WebGlUniformBlockValue, WebGlUniformValue},
};

pub struct WebGlContext {
    gl: WebGl2RenderingContext,
    channel: Channel,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,
    texture_manager: WebGlTextureManager,
    capabilities: WebGlCapabilities,

    using_program: Option<WebGlProgramItem>,
    using_uniform_buffer_objects: HashMap<usize, (WebGlBufferItem, Range<usize>)>,
    activating_texture_units: HashMap<WebGlTextureUnit, WebGlTextureItem>,
}

impl WebGlContext {
    /// Constructs a new WebGl drawing context.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            program_manager: WebGlProgramManager::new(gl.clone()),
            buffer_manager: WebGlBufferManager::new(gl.clone(), channel.clone()),
            texture_manager: WebGlTextureManager::new(gl.clone(), channel.clone()),
            capabilities: WebGlCapabilities::new(gl.clone()),
            gl,
            channel,

            using_program: None,
            using_uniform_buffer_objects: HashMap::new(),
            activating_texture_units: HashMap::new(),
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
        Self::use_program_inner(&self.gl, &mut self.using_program, &program_item);
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

    /// Manages a [`WebGlBuffering`] and syncs its queueing [`BufferData`](super::super::super::buffering::BufferData) into WebGl context.
    pub fn sync_buffer(&mut self, buffering: &WebGlBuffering) -> Result<WebGlBufferItem, Error> {
        self.buffer_manager.sync_buffer(buffering)
    }

    /// Manages a [`WebGlTexturing`] and syncs its queueing [`TextureData`](super::super::super::texturing::TextureData) into WebGl context.
    pub fn sync_texture(&mut self, texturing: &WebGlTexturing) -> Result<WebGlTextureItem, Error> {
        self.texture_manager
            .sync_texture(texturing, &mut self.buffer_manager, &self.capabilities)
    }

    /// Sets a attribute by specified attribute name.
    pub fn set_attribute_value(
        &mut self,
        name: &str,
        value: WebGlAttributeValue,
    ) -> Result<(), Error> {
        let Some(using_program) = self.using_program.as_ref() else {
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
                byte_stride,
                byte_offset,
            } => {
                let buffer_item = buffer_manager.sync_buffer(buffering)?;
                gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(buffer_item.gl_buffer()),
                );
                gl.vertex_attrib_pointer_with_i32(
                    location,
                    component_size as i32,
                    data_type.to_gl_enum(),
                    normalized,
                    byte_stride as i32,
                    byte_offset as i32,
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
                byte_stride,
                byte_offset,
            } => {
                let buffer_item = buffer_manager.sync_buffer(buffering)?;
                gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(buffer_item.gl_buffer()),
                );
                gl.vertex_attrib_pointer_with_i32(
                    location,
                    component_size as i32,
                    data_type.to_gl_enum(),
                    normalized,
                    byte_stride as i32,
                    byte_offset as i32,
                );
                gl.enable_vertex_attrib_array(location);
                gl.vertex_attrib_divisor(location, instance_size as u32);
                gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);
            }
            WebGlAttributeValue::Float1(x) => gl.vertex_attrib1f(location, x),
            WebGlAttributeValue::Float2(x, y) => gl.vertex_attrib2f(location, x, y),
            WebGlAttributeValue::Float3(x, y, z) => gl.vertex_attrib3f(location, x, y, z),
            WebGlAttributeValue::Float4(x, y, z, w) => gl.vertex_attrib4f(location, x, y, z, w),
            WebGlAttributeValue::Integer4(x, y, z, w) => gl.vertex_attrib_i4i(location, x, y, z, w),
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
        };

        Ok(())
    }

    /// Sets a uniform value by specified uniform name.
    pub fn set_uniform_value(&mut self, name: &str, value: WebGlUniformValue) -> Result<(), Error> {
        let Some(using_program) = self.using_program.as_ref() else {
            return Err(Error::NoUsingProgram);
        };
        let Some(location) = using_program.uniform_location(name) else {
            return Err(Error::UniformLocationNotFound(name.to_string()));
        };
        Self::set_uniform_value_inner(
            &self.gl,
            &location,
            value,
            &mut self.activating_texture_units,
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
        activating_texture_units: &mut HashMap<WebGlTextureUnit, WebGlTextureItem>,
        texture_manager: &mut WebGlTextureManager,
        buffer_manager: &mut WebGlBufferManager,
        capabilities: &WebGlCapabilities,
    ) -> Result<(), Error> {
        match value {
            WebGlUniformValue::Bool(v) => gl.uniform1i(Some(location), if v { 1 } else { 0 }),
            WebGlUniformValue::Texture { texturing, unit } => {
                let item = texture_manager.sync_texture(texturing, buffer_manager, capabilities)?;
                gl.bind_texture(item.layout().to_gl_enum(), Some(item.gl_texture()));
                gl.uniform1i(Some(location), unit.as_index());
                gl.active_texture(unit.to_gl_enum());
                gl.bind_texture(item.layout().to_gl_enum(), None);
                activating_texture_units.insert(unit, item);
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
        let (buffer, mount_point, byte_offset, byte_length) = match value {
            WebGlUniformBlockValue::Base {
                buffer,
                mount_point,
            } => (buffer, mount_point, 0, None),
            WebGlUniformBlockValue::Range {
                buffer,
                mount_point,
                byte_offset,
                byte_length,
            } => (buffer, mount_point, byte_offset, byte_length),
        };

        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        let Some(using_program) = self.using_program.as_ref() else {
            return Err(Error::NoUsingProgram);
        };
        let Some(location) = using_program.uniform_block_location(name) else {
            return Err(Error::UniformBlockLocationNotFound(name.to_string()));
        };

        match byte_length {
            Some(byte_length) => {
                Self::mount_uniform_buffer_object_inner(
                    &self.gl,
                    buffer_item.clone(),
                    &mut self.using_uniform_buffer_objects,
                    mount_point,
                    byte_offset..byte_length,
                );
            }
            None => {
                Self::mount_uniform_buffer_object_inner(
                    &self.gl,
                    buffer_item.clone(),
                    &mut self.using_uniform_buffer_objects,
                    mount_point,
                    byte_offset..,
                );
            }
        };

        Self::set_uniform_block_mount_point_inner(
            &self.gl,
            using_program,
            &buffer_item,
            location,
            mount_point,
        );

        Ok(())
    }

    fn set_uniform_block_mount_point_inner(
        gl: &WebGl2RenderingContext,
        program_item: &WebGlProgramItem,
        buffer_item: &WebGlBufferItem,
        location: u32,
        mount_point: usize,
    ) {
        gl.bind_buffer(
            WebGlBufferTarget::UniformBuffer.to_gl_enum(),
            Some(buffer_item.gl_buffer()),
        );
        gl.uniform_block_binding(program_item.gl_program(), location, mount_point as u32);
        gl.bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);
    }

    /// Binds a buffer to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object(
        &mut self,
        buffering: &WebGlBuffering,
        mount_point: usize,
    ) -> Result<(), Error> {
        let buffer_item = self.buffer_manager.sync_buffer(buffering)?;
        Self::mount_uniform_buffer_object_inner(
            &self.gl,
            buffer_item,
            &mut self.using_uniform_buffer_objects,
            mount_point,
            ..,
        );
        Ok(())
    }

    /// Binds a buffer range to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object_by_range<R>(
        &mut self,
        buffering: &WebGlBuffering,
        mount_point: usize,
        range: R,
    ) -> Result<(), Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffering)?;
        Self::mount_uniform_buffer_object_inner(
            &self.gl,
            buffer_item,
            &mut self.using_uniform_buffer_objects,
            mount_point,
            range,
        );
        Ok(())
    }

    fn mount_uniform_buffer_object_inner<R>(
        gl: &WebGl2RenderingContext,
        buffer_item: WebGlBufferItem,
        using_uniform_buffer_objects: &mut HashMap<usize, (WebGlBufferItem, Range<usize>)>,
        mount_point: usize,
        range: R,
    ) where
        R: RangeBounds<usize>,
    {
        let byte_range = buffer_item.normalize_byte_range(range);
        if let Some((bound_buffer_item, bound_byte_range)) =
            using_uniform_buffer_objects.get(&mount_point)
        {
            if bound_buffer_item.gl_buffer() == buffer_item.gl_buffer()
                && bound_byte_range == &byte_range
            {
                return;
            }
        }

        gl.bind_buffer_range_with_i32_and_i32(
            WebGlBufferTarget::UniformBuffer.to_gl_enum(),
            mount_point as u32,
            Some(buffer_item.gl_buffer()),
            byte_range.start as i32,
            byte_range.len() as i32,
        );
        gl.bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);
        using_uniform_buffer_objects.insert(mount_point, (buffer_item, byte_range));
    }

    /// Reads buffer data into an [`Uint8Array`].
    pub fn read_buffer(&mut self, buffering: &WebGlBuffering) -> Result<Uint8Array, Error> {
        self.read_buffer_by_range(buffering, ..)
    }

    /// Reads buffer data into an [`Uint8Array`] with byte range.
    pub fn read_buffer_by_range<R>(
        &mut self,
        buffering: &WebGlBuffering,
        byte_range: R,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffering)?;
        let byte_range = buffer_item.normalize_byte_range(byte_range);
        Self::read_buffer_inner(&self.gl, buffer_item.gl_buffer(), byte_range)
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
        byte_range: R,
        client_wait: &WebGlClientWait,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffering)?;
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
            buffer_item.byte_length() as i32,
            WebGlBufferUsage::StreamRead.to_gl_enum(),
        );
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            WebGlBufferTarget::CopyReadBuffer.to_gl_enum(),
            WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
            0,
            0,
            buffer_item.byte_length() as i32,
        );
        self.gl
            .bind_buffer(WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(), None);
        self.gl
            .bind_buffer(WebGlBufferTarget::CopyReadBuffer.to_gl_enum(), None);

        client_wait.client_wait(&self.gl).await?;

        let byte_range = buffer_item.normalize_byte_range(byte_range);
        let data = Self::read_buffer_inner(&self.gl, &tmp_gl_buffer, byte_range)?;

        self.gl.delete_buffer(Some(&tmp_gl_buffer));

        Ok(data)
    }

    fn read_buffer_inner(
        gl: &WebGl2RenderingContext,
        gl_buffer: &WebGlBuffer,
        byte_range: Range<usize>,
    ) -> Result<Uint8Array, Error> {
        let dst_byte_length = byte_range.len();
        let dst = Uint8Array::new_with_length(dst_byte_length as u32);

        gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), Some(gl_buffer));
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view_and_dst_offset_and_length(
            WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
            byte_range.start as i32,
            dst.as_ref(),
            0,
            byte_range.end as u32,
        );
        gl.bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);

        Ok(dst)
    }
}
