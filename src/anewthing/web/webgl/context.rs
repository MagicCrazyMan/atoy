use std::ops::{Bound, RangeBounds};

use hashbrown::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram};

use crate::anewthing::{buffer::Buffer, channel::Channel};

use super::{
    buffer::{WebGlBufferData, WebGlBufferManager, WebGlBufferTarget},
    error::Error,
    program::{WebGlProgramItem, WebGlProgramManager, WebGlShaderSource},
};

pub struct Context {
    gl: WebGl2RenderingContext,
    channel: Channel,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,

    using_program: Option<WebGlProgram>,
    using_ubos: HashMap<usize, WebGlBuffer>,
}

impl Context {
    /// Constructs a new WebGl drawing context.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            program_manager: WebGlProgramManager::new(gl.clone()),
            buffer_manager: WebGlBufferManager::new(gl.clone(), channel.clone()),
            gl,
            channel,

            using_program: None,
            using_ubos: HashMap::new(),
        }
    }

    /// Returns the native [`WebGl2RenderingContext`].
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Returns the associated message channel.
    pub fn channel(&self) -> &Channel {
        &self.channel
    }

    /// Returns the [`WebGlProgramManager`].
    pub fn program_manager(&self) -> &WebGlProgramManager {
        &self.program_manager
    }

    /// Returns the mutable [`WebGlProgramManager`].
    pub fn program_manager_mut(&mut self) -> &mut WebGlProgramManager {
        &mut self.program_manager
    }

    /// Returns the [`WebGlBufferManager`].
    pub fn buffer_manager(&self) -> &WebGlBufferManager {
        &self.buffer_manager
    }

    /// Returns the mutable [`WebGlBufferManager`].
    pub fn buffer_manager_mut(&mut self) -> &mut WebGlBufferManager {
        &mut self.buffer_manager
    }

    /// Uses a compiled [`WebGlProgramItem`] to this WebGl context.
    pub fn use_program(&mut self, program: &WebGlProgramItem) {
        if self.using_program.as_ref() == Some(program.gl_program()) {
            return;
        }

        self.gl.use_program(Some(program.gl_program()));
        self.using_program = Some(program.gl_program().clone());
    }

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
        self.mount_uniform_buffer_object(buffer, mount_point, ..)
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
        self.mount_uniform_buffer_object(buffer, mount_point, range)
    }

    fn mount_uniform_buffer_object<R>(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        mount_point: usize,
        range: R,
    ) -> Result<(), Error>
    where
        R: RangeBounds<usize>,
    {
        let buffer_item = self.buffer_manager.sync_buffer(buffer)?;

        let range = match (range.start_bound(), range.end_bound()) {
            (Bound::Included(s), Bound::Included(e)) => Some(*s..*e + 1),
            (Bound::Included(s), Bound::Excluded(e)) => Some(*s..*e),
            (Bound::Included(s), Bound::Unbounded) => Some(*s..buffer.byte_length()),
            (Bound::Unbounded, Bound::Included(e)) => Some(0..*e + 1),
            (Bound::Unbounded, Bound::Excluded(e)) => Some(0..*e),
            (Bound::Unbounded, Bound::Unbounded) => None,
            (Bound::Excluded(_), _) => unreachable!(),
        };

        match range {
            Some(range) => self.gl.bind_buffer_range_with_i32_and_i32(
                WebGlBufferTarget::UniformBuffer.to_gl_enum(),
                mount_point as u32,
                Some(buffer_item.gl_buffer()),
                range.start as i32,
                range.len() as i32,
            ),
            None => self.gl.bind_buffer_base(
                WebGlBufferTarget::UniformBuffer.to_gl_enum(),
                mount_point as u32,
                Some(buffer_item.gl_buffer()),
            ),
        };
        self.gl
            .bind_buffer(WebGlBufferTarget::UniformBuffer.to_gl_enum(), None);
        self.using_ubos
            .insert(mount_point, buffer_item.gl_buffer().clone());

        Ok(())
    }
}
