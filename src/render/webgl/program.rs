use std::borrow::Cow;

use hashbrown::HashMap;
use log::warn;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::{
    conversion::GLuint,
    error::Error,
    shader::{ShaderBuilder, ShaderType},
};

/// Source providing basic data for compiling a [`WebGlProgram`].
/// Data provided by program source should never change even in different condition.
pub trait ProgramSource {
    /// Program name, should be unique,
    fn name(&self) -> Cow<'static, str>;

    /// Shader sources, at least one vertex shader and one fragment shader should be specified.
    fn sources(&self) -> Vec<ShaderSource>;
}

/// Shader source codes. 3 available types:
///
/// 1. `Builder`, build shader source code from a [`ShaderBuilder`].
/// 2. `VertexRaw`, developer provides a complete vertex shader source code.
/// 3. `FragmentRaw`, developer provides a complete fragment shader source code.
#[derive(Clone)]
pub enum ShaderSource {
    Builder(ShaderBuilder),
    VertexRaw(Cow<'static, str>),
    FragmentRaw(Cow<'static, str>),
}

/// Compiled program item.
pub struct Program {
    name: String,
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    shaders: Vec<WebGlShader>,

    attribute_locations: HashMap<String, GLuint>,
    uniform_locations: HashMap<String, WebGlUniformLocation>,
    uniform_block_indices: HashMap<String, u32>,
}

impl Program {
    /// Returns program source name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the native [`WebGlProgram`].
    pub fn program(&self) -> &WebGlProgram {
        &self.program
    }

    /// Returns uniform locations from variable name.
    pub fn get_or_retrieve_attribute_locations(&mut self, variable_name: &str) -> Option<u32> {
        if let Some(location) = self.attribute_locations.get(variable_name) {
            Some(*location)
        } else {
            let location = self.gl.get_attrib_location(&self.program, variable_name);
            if location == -1 {
                None
            } else {
                let (_, location) = self
                    .attribute_locations
                    .insert_unique_unchecked(variable_name.to_string(), location as u32);
                Some(*location)
            }
        }
    }

    /// Returns uniform locations from variable name.
    pub fn get_or_retrieve_uniform_location(
        & mut self,
        variable_name: & str,
    ) -> Option<WebGlUniformLocation> {
        match self.uniform_locations.entry_ref(variable_name) {
            hashbrown::hash_map::EntryRef::Occupied(v) => Some(v.get().clone()),
            hashbrown::hash_map::EntryRef::Vacant(v) => {
                let location = self.gl.get_uniform_location(&self.program, variable_name);
                let Some(location) = location else {
                    return None;
                };
                let location = v.insert(location);
                Some(location.clone())
            }
        }
    }

    /// Returns uniform block index from variable name.
    pub fn get_or_retrieve_uniform_block_index(&mut self, block_name: &str) -> u32 {
        if let Some(index) = self.uniform_block_indices.get(block_name) {
            *index
        } else {
            let index = self.gl.get_uniform_block_index(&self.program, &block_name);
            let (_, index) = self
                .uniform_block_indices
                .insert_unique_unchecked(block_name.to_string(), index);
            *index
        }
    }
}

/// A centralized program store storing and caching compiled program item.
/// Program store caches a program by an unique name provided by [`ProgramSource::name`].
pub struct ProgramStore {
    gl: WebGl2RenderingContext,
    store: HashMap<String, *mut Program>,
    using_program: Option<*mut Program>,
}

impl ProgramStore {
    /// Constructs a new program store.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: HashMap::new(),
            using_program: None,
        }
    }

    fn use_program_inner<'a, 'b, 'c, S>(
        &'a mut self,
        source: &'b S,
        vertex_defines: Option<Vec<Cow<'static, str>>>,
        fragment_defines: Option<Vec<Cow<'static, str>>>,
    ) -> Result<&'c mut Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        let name = match fragment_defines.as_ref() {
            Some(defines) => Cow::Owned(format!("{}_{}", source.name(), defines.join("_"))),
            None => source.name(),
        };
        unsafe {
            if let Some(using_program) = self.using_program.as_ref() {
                let using_program = &mut **using_program;
                if using_program.name() == name {
                    return Ok(using_program);
                }
            }

            if let Some(program) = self.store.get(name.as_ref()) {
                let program = &mut **program;
                self.gl.use_program(Some(&program.program));
                self.using_program = Some(program);
                return Ok(program);
            }

            let program = compile_program(&self.gl, source, vertex_defines, fragment_defines)?;
            let program: *mut Program = Box::leak(Box::new(program));
            let (_, program) = self
                .store
                .insert_unique_unchecked(name.to_string(), program);
            self.gl.use_program(Some(&(**program).program));
            self.using_program = Some(*program);
            Ok(&mut **program)
        }
    }

    /// Uses a program from a program source and uses a custom name instead of program source name.
    /// Compiles program from program source if never uses before.
    pub fn use_program_with_defines<'a, 'b, 'c, S>(
        &'a mut self,
        source: &'b S,
        vertex_defines: Vec<Cow<'static, str>>,
        fragment_defines: Vec<Cow<'static, str>>,
    ) -> Result<&'c mut Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        self.use_program_inner(source, Some(vertex_defines), Some(fragment_defines))
    }

    /// Uses a program from a program source.
    /// Compiles program from program source if never uses before.
    pub fn use_program<'a, 'b, 'c, S>(&'a mut self, source: &'b S) -> Result<&'c mut Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        self.use_program_inner(source, None, None)
    }

    /// Unuses a program.
    pub fn unuse_program(&mut self) {
        self.gl.use_program(None);
        self.using_program = None;
    }

    /// Deletes a [`ProgramItem`].
    pub fn delete_program(&mut self, name: &str) {
        let Some(removed) = self.store.remove(name) else {
            return;
        };

        unsafe {
            let removed = Box::from_raw(removed);
            if self
                .using_program
                .as_ref()
                .map(|using_program| (**using_program).name() == removed.name())
                .unwrap_or(false)
            {
                self.using_program = None;
            }

            delete_program(&self.gl, *removed);
        }
    }
}

impl Drop for ProgramStore {
    fn drop(&mut self) {
        self.using_program.take();

        let gl = self.gl.clone();
        self.store.drain().for_each(|(_, program)| unsafe {
            let program = Box::from_raw(program);
            delete_program(&gl, *program);
        });
    }
}

/// Compiles a [`WebGlProgram`] from a [`ProgramSource`].
pub fn compile_program<S>(
    gl: &WebGl2RenderingContext,
    source: &S,
    vertex_defines: Option<Vec<Cow<'static, str>>>,
    fragment_defines: Option<Vec<Cow<'static, str>>>,
) -> Result<Program, Error>
where
    S: ProgramSource + ?Sized,
{
    let mut sources = source.sources();
    let mut shaders = Vec::with_capacity(sources.len());
    sources.iter_mut().try_for_each(|source| {
        shaders.push(compile_shader(
            gl,
            source,
            vertex_defines.as_ref(),
            fragment_defines.as_ref(),
        )?);
        Ok(()) as Result<(), Error>
    })?;

    let program = create_program(gl, &shaders)?;
    Ok(Program {
        name: source.name().to_string(),

        attribute_locations: HashMap::new(),
        uniform_locations: HashMap::new(),
        uniform_block_indices: HashMap::new(),

        shaders,
        program,
        gl: gl.clone(),
    })
}

fn delete_program(gl: &WebGl2RenderingContext, program: Program) {
    let Program {
        program, shaders, ..
    } = program;
    gl.use_program(None);
    shaders.iter().for_each(|shader| {
        gl.delete_shader(Some(&shader));
    });
    gl.delete_program(Some(&program));
}

/// Compiles [`WebGlShader`] by [`ShaderSource`].
pub fn compile_shader(
    gl: &WebGl2RenderingContext,
    source: &mut ShaderSource,
    vertex_defines: Option<&Vec<Cow<'static, str>>>,
    fragment_defines: Option<&Vec<Cow<'static, str>>>,
) -> Result<WebGlShader, Error> {
    let (shader, code) = match source {
        ShaderSource::Builder(builder) => match builder.shader_type() {
            ShaderType::Vertex => {
                if let Some(vertex_defines) = vertex_defines {
                    builder.defines_mut().extend_from_slice(vertex_defines);
                }

                (
                    gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                        .ok_or(Error::CreateVertexShaderFailed)?,
                    Cow::Owned(builder.build()),
                )
            }
            ShaderType::Fragment => {
                if let Some(fragment_defines) = fragment_defines {
                    builder.defines_mut().extend_from_slice(fragment_defines);
                }

                (
                    gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                        .ok_or(Error::CreateVertexShaderFailed)?,
                    Cow::Owned(builder.build()),
                )
            }
        },
        ShaderSource::VertexRaw(code) => {
            if vertex_defines.is_some() {
                warn!("vertex defines for ShaderSource::VertexRaw is not supported");
            }

            let shader = gl
                .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                .ok_or(Error::CreateVertexShaderFailed)?;
            (shader, code.clone())
        }
        ShaderSource::FragmentRaw(code) => {
            if fragment_defines.is_some() {
                warn!("fragment defines for ShaderSource::FragmentRaw is not supported");
            }

            let shader = gl
                .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                .ok_or(Error::CreateFragmentShaderFailed)?;
            (shader, code.clone())
        }
    };

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
        Err(Error::CompileShaderFailed(err))
    } else {
        Ok(shader)
    }
}

/// Creates a [`WebGlProgram`], and links compiled [`WebGlShader`] to the program.
pub fn create_program(
    gl: &WebGl2RenderingContext,
    shaders: &[WebGlShader],
) -> Result<WebGlProgram, Error> {
    let program = gl.create_program().ok_or(Error::CreateProgramFailed)?;

    // attaches shader to program
    for shader in shaders {
        gl.attach_shader(&program, shader);
    }
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
        Err(Error::CompileProgramFailed(err))
    } else {
        Ok(program)
    }
}
