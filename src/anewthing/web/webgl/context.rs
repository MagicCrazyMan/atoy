use web_sys::{WebGl2RenderingContext, WebGlProgram};

use crate::anewthing::channel::Channel;

use super::{
    buffer::WebGlBufferManager, error::Error, program::{WebGlProgramItem, WebGlProgramManager, WebGlShaderSource}
};

pub struct Context {
    gl: WebGl2RenderingContext,
    channel: Channel,
    program_manager: WebGlProgramManager,
    buffer_manager: WebGlBufferManager,

    using_program: Option<WebGlProgram>,
     // bound_ubos: HashMap<usize, WebGlBuffer>,
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
        let program = self.program_manager.get_or_compile_program(vertex, fragment)?;
        self.gl.use_program(Some(program.gl_program()));
        self.using_program = Some(program.gl_program().clone());
        Ok(program)
    }

    // pub fn
}
