use std::{ops::RangeBounds, time::Duration};

use hashbrown::HashMap;
use js_sys::Uint8Array;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram};

use crate::anewthing::{buffer::Buffer, channel::Channel};

use super::{
    buffer::{WebGlBufferData, WebGlBufferItem, WebGlBufferManager, WebGlBufferTarget}, capabilities::WebGlCapabilities, client_wait::{WebGlClientWait, WebGlClientWaitFlag}, error::Error, program::{WebGlProgramItem, WebGlProgramManager, WebGlShaderSource}
};

pub struct Context {
    gl: WebGl2RenderingContext,
    channel: Channel,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,
    capabilities: WebGlCapabilities,

    using_program: Option<WebGlProgram>,
    using_ubos: HashMap<usize, WebGlBuffer>,
}

impl Context {
    /// Constructs a new WebGl drawing context.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            program_manager: WebGlProgramManager::new(gl.clone()),
            buffer_manager: WebGlBufferManager::new(gl.clone(), channel.clone()),
            capabilities: WebGlCapabilities::new(gl.clone()),
            gl,
            channel,

            using_program: None,
            using_ubos: HashMap::new(),
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
    
    /// Returns [`WebGlCapabilities`].
    pub fn capabilities(&self) -> &WebGlCapabilities {
        &self.capabilities
    }

    /// Creates a new [`WebGlClientWait`].
    pub fn create_client_wait(&self, wait_timeout: Duration) -> WebGlClientWait {
        WebGlClientWait::new(self.gl.clone(), wait_timeout)
    }

    /// Creates a new [`WebGlClientWait`] with retries.
    pub fn create_client_wait_with_retries(
        &self,
        wait_timeout: Duration,
        retry_interval: Duration,
        max_retries: usize,
    ) -> WebGlClientWait {
        WebGlClientWait::with_retries(self.gl.clone(), wait_timeout, retry_interval, max_retries)
    }

    /// Creates a new [`WebGlClientWait`] with flags.
    pub fn create_client_wait_with_flags<I>(
        &self,
        wait_timeout: Duration,
        flags: I,
    ) -> WebGlClientWait
    where
        I: IntoIterator<Item = WebGlClientWaitFlag>,
    {
        WebGlClientWait::with_flags(self.gl.clone(), wait_timeout, flags)
    }

    /// Creates a new [`WebGlClientWait`] with flags and retries.
    pub fn create_client_wait_with_flags_and_retries<I>(
        &self,
        wait_timeout: Duration,
        retry_interval: Duration,
        max_retries: usize,
        flags: I,
    ) -> WebGlClientWait
    where
        I: IntoIterator<Item = WebGlClientWaitFlag>,
    {
        WebGlClientWait::with_flags_and_retries(
            self.gl.clone(),
            wait_timeout,
            retry_interval,
            max_retries,
            flags,
        )
    }

    // /// Uses a compiled [`WebGlProgramItem`] to this WebGl context.
    // pub fn use_program(&mut self, program: &WebGlProgramItem) {
    //     if self.using_program.as_ref() == Some(program.gl_program()) {
    //         return;
    //     }

    //     self.gl.use_program(Some(program.gl_program()));
    //     self.using_program = Some(program.gl_program().clone());
    // }

    /// Compiles shader sources and then uses the compiled program.
    /// Returns the compiled [`WebGlProgramItem`] as well.
    pub fn use_program_by_shader_sources<VS, FS>(
        &mut self,
        vertex: &VS,
        fragment: &FS,
    ) -> Result<&WebGlProgramItem, Error>
    where
        VS: WebGlShaderSource,
        FS: WebGlShaderSource,
    {
        let program = self
            .program_manager
            .get_or_compile_program(vertex, fragment)?;
        self.gl.use_program(Some(program.gl_program()));
        self.using_program = Some(program.gl_program().clone());
        Ok(program)
    }

    /// Binds a buffer to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object_base(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        mount_point: usize,
    ) -> Result<(), Error> {
        self.mount_uniform_buffer_object_inner(buffer, mount_point, ..)
    }

    /// Binds a buffer range to uniform buffer object mount point.
    /// Unmounting previous mounted buffer if occupied.
    pub fn mount_uniform_buffer_object_range<R>(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        mount_point: usize,
        range: R,
    ) -> Result<(), Error>
    where
        R: RangeBounds<usize>,
    {
        self.mount_uniform_buffer_object_inner(buffer, mount_point, range)
    }

    fn mount_uniform_buffer_object_inner<R>(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        mount_point: usize,
        range: R,
    ) -> Result<(), Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        let byte_range = buffer_item.normalize_byte_range(range);
        self.gl.bind_buffer_range_with_i32_and_i32(
            WebGlBufferTarget::UniformBuffer.to_gl_enum(),
            mount_point as u32,
            Some(buffer_item.gl_buffer()),
            byte_range.start as i32,
            byte_range.len() as i32,
        );
        self.gl
            .bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);
        self.using_ubos
            .insert(mount_point, buffer_item.gl_buffer().clone());

        Ok(())
    }

    /// Reads buffer data into an [`Uint8Array`].
    pub fn read_buffer(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
    ) -> Result<Uint8Array, Error> {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        self.read_buffer_inner(buffer_item, ..)
    }

    /// Reads buffer data into an [`Uint8Array`] with byte range.
    pub fn read_buffer_by_range<R>(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        range: R,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        self.read_buffer_inner(buffer_item, range)
    }

    /// Reads buffer data into an [`Uint8Array`] asynchronously.
    pub async fn read_buffer_with_client_wait(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        client_wait: WebGlClientWait,
    ) -> Result<Uint8Array, Error> {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        client_wait.client_wait().await?;
        self.read_buffer_inner(buffer_item, ..)
    }

    /// Reads buffer data into an [`Uint8Array`] with byte range asynchronously.
    pub async fn read_buffer_by_range_with_client_wait<R>(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        range: R,
        client_wait: WebGlClientWait,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;
        client_wait.client_wait().await?;
        self.read_buffer_inner(buffer_item, range)
    }

    fn read_buffer_inner<R>(
        &mut self,
        buffer_item: WebGlBufferItem,
        range: R,
    ) -> Result<Uint8Array, Error>
    where
        R: RangeBounds<usize>,
    {
        let byte_range = buffer_item.normalize_byte_range(range);
        let dst_byte_length = byte_range.len();
        let dst = Uint8Array::new_with_length(dst_byte_length as u32);

        self.gl.bind_buffer(
            WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
            Some(buffer_item.gl_buffer()),
        );
        self.gl
            .get_buffer_sub_data_with_i32_and_array_buffer_view_and_dst_offset_and_length(
                WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                byte_range.start as i32,
                dst.as_ref(),
                0,
                byte_range.end as u32,
            );
        self.gl
            .bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);

        Ok(dst)
    }
}
