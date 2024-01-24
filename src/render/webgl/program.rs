use std::borrow::Cow;

use hashbrown::HashMap;
use log::warn;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::{
    error::Error,
    shader::{Define, ShaderBuilder},
};

/// Source providing basic data for compiling a [`WebGlProgram`].
/// Data provided by program source should never change even in different condition.
pub trait ProgramSource {
    /// Program name, should be unique,
    fn name(&self) -> Cow<'static, str>;

    /// Returns [`VertexShaderSource`]  for vertex shader.
    fn vertex_source(&self) -> VertexShaderSource;

    /// Returns [`FragmentShaderSource`]  for vertex shader.
    fn fragment_source(&self) -> FragmentShaderSource;
}

/// Vertex shader source code. 2 available types:
///
/// 1. `Builder`, build shader source code from a [`ShaderBuilder`].
/// 2. `Rsw`, developer provides a complete vertex shader source code.
#[derive(Clone)]
pub enum VertexShaderSource {
    Builder(ShaderBuilder),
    Raw(Cow<'static, str>),
}

impl VertexShaderSource {
    /// Returns vertex shader code.
    pub fn code(self) -> Cow<'static, str> {
        match self {
            VertexShaderSource::Builder(builder) => Cow::Owned(builder.build_vertex_shader()),
            VertexShaderSource::Raw(code) => code,
        }
    }
}

/// Fragment shader source code. 2 available types:
///
/// 1. `Builder`, build shader source code from a [`ShaderBuilder`].
/// 2. `Rsw`, developer provides a complete fragment shader source code.
#[derive(Clone)]
pub enum FragmentShaderSource {
    Builder(ShaderBuilder),
    Raw(Cow<'static, str>),
}

impl FragmentShaderSource {
    /// Returns fragment shader code.
    pub fn code(self) -> Cow<'static, str> {
        match self {
            FragmentShaderSource::Builder(builder) => Cow::Owned(builder.build_fragment_shader()),
            FragmentShaderSource::Raw(code) => code,
        }
    }
}

/// Compiled program item.
pub struct Program {
    name: String,
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vertex_shader: WebGlShader,
    fragment_shader: WebGlShader,

    attribute_locations: HashMap<String, u32>,
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
        &mut self,
        variable_name: &str,
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
        vertex_defines: Option<&'b [Define]>,
        fragment_defines: Option<&'b [Define]>,
    ) -> Result<&'c mut Program, Error>
    where
        S: ProgramSource + ?Sized,
    {
        let vertex_defines = vertex_defines.and_then(|defines| {
            if defines.len() == 0 {
                None
            } else {
                Some(defines)
            }
        });
        let fragment_defines = fragment_defines.and_then(|defines| {
            if defines.len() == 0 {
                None
            } else {
                Some(defines)
            }
        });

        // create shader name
        let name = match (vertex_defines.as_ref(), fragment_defines.as_ref()) {
            (None, None) => source.name(),
            (Some(defines), None) => Cow::Owned(format!(
                "{}!{}",
                source.name(),
                defines
                    .iter()
                    .map(|d| d.name().clone())
                    .collect::<Vec<_>>()
                    .join("_")
            )),
            (None, Some(defines)) => Cow::Owned(format!(
                "{}!!{}",
                source.name(),
                defines
                    .iter()
                    .map(|d| d.name().clone())
                    .collect::<Vec<_>>()
                    .join("_")
            )),
            (Some(vertex_defines), Some(fragment_defines)) => Cow::Owned(format!(
                "{}!{}!{}",
                source.name(),
                vertex_defines
                    .iter()
                    .map(|d| d.name().clone())
                    .collect::<Vec<_>>()
                    .join("_"),
                fragment_defines
                    .iter()
                    .map(|d| d.name().clone())
                    .collect::<Vec<_>>()
                    .join("_")
            )),
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

            let name = name.to_string();
            let program = compile_program(
                self.gl.clone(),
                name.clone(),
                source,
                vertex_defines,
                fragment_defines,
            )?;
            let program: *mut Program = Box::leak(Box::new(program));
            let (_, program) = self.store.insert_unique_unchecked(name, program);
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
        vertex_defines: &'b [Define],
        fragment_defines: &'b [Define],
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
                self.gl.use_program(None);
            }

            delete_program(&self.gl, *removed);
        }
    }
}

impl Drop for ProgramStore {
    fn drop(&mut self) {
        self.using_program.take();

        let gl = self.gl.clone();
        gl.use_program(None);
        self.store.drain().for_each(|(_, program)| unsafe {
            let program = Box::from_raw(program);
            delete_program(&gl, *program);
        });
    }
}

/// Compiles a [`WebGlProgram`] from a [`ProgramSource`].
pub fn compile_program<S>(
    gl: WebGl2RenderingContext,
    name: String,
    source: &S,
    vertex_defines: Option<&[Define]>,
    fragment_defines: Option<&[Define]>,
) -> Result<Program, Error>
where
    S: ProgramSource + ?Sized,
{
    let mut vertex_source = source.vertex_source();
    if let Some(vertex_defines) = vertex_defines {
        if let VertexShaderSource::Builder(builder) = &mut vertex_source {
            builder.defines_mut().extend_from_slice(vertex_defines);
        } else {
            warn!("vertex defines for VertexShaderSource::Raw is not supported");
        }
    }
    let vertex_shader = compile_shader(&gl, true, vertex_source.code().as_ref())?;

    let mut fragment_source = source.fragment_source();
    if let Some(fragment_defines) = fragment_defines {
        if let FragmentShaderSource::Builder(builder) = &mut fragment_source {
            builder.defines_mut().extend_from_slice(fragment_defines);
        } else {
            warn!("fragment defines for FragmentShaderSource::Raw is not supported");
        }
    }
    let fragment_shader = compile_shader(&gl, false, fragment_source.code().as_ref())?;

    let program = create_program(&gl, &vertex_shader, &fragment_shader)?;
    Ok(Program {
        name,

        gl,
        program,
        vertex_shader,
        fragment_shader,

        attribute_locations: HashMap::new(),
        uniform_locations: HashMap::new(),
        uniform_block_indices: HashMap::new(),
    })
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
    is_vertex_shader: bool,
    code: &str,
) -> Result<WebGlShader, Error> {
    let shader = gl
        .create_shader(if is_vertex_shader {
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
        Err(Error::CompileProgramFailure(err))
    } else {
        Ok(program)
    }
}
