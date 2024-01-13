use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

use log::warn;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::{
    attribute::AttributeBinding,
    conversion::GLuint,
    error::Error,
    shader::{ShaderBuilder, ShaderType},
    uniform::{UniformBinding, UniformBlockBinding, UniformStructuralBinding},
};

/// Source providing basic data for compiling a [`WebGlProgram`].
/// Data provided by program source should never change even in different condition.
pub trait ProgramSource {
    /// Program name, should be unique,
    fn name(&self) -> Cow<'static, str>;

    /// Shader sources, at least one vertex shader and one fragment shader should be specified.
    fn sources(&self) -> Vec<ShaderSource>;

    /// Attribute binding variable name.
    fn attribute_bindings(&self) -> Vec<AttributeBinding>;

    /// Uniform variable bindings.
    fn uniform_bindings(&self) -> Vec<UniformBinding>;

    /// Uniform structural variable bindings.
    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding>;

    /// Uniform block variable bindings.
    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding>;
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

    name_attribute_locations: HashMap<String, GLuint>,
    name_uniform_locations: HashMap<String, WebGlUniformLocation>,
    name_uniform_structural_locations: HashMap<String, WebGlUniformLocation>,
    name_uniform_block_indices: HashMap<String, u32>,

    binding_attribute_locations: HashMap<AttributeBinding, GLuint>,
    binding_uniform_locations: HashMap<UniformBinding, WebGlUniformLocation>,
    binding_uniform_structural_locations:
        HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>>,
    binding_uniform_block_indices: HashMap<UniformBlockBinding, u32>,
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

    /// Returns variable names to attribute locations mapping.
    pub fn attribute_locations(&self) -> &HashMap<String, GLuint> {
        &self.name_attribute_locations
    }

    /// Returns variable names to uniform locations mapping.
    pub fn uniform_locations(&self) -> &HashMap<String, WebGlUniformLocation> {
        &self.name_uniform_locations
    }

    /// Returns variable names to uniform locations mapping.
    pub fn get_or_uniform_locations<S: Into<String>>(
        &mut self,
        variable_name: S,
    ) -> Option<WebGlUniformLocation> {
        let variable_name = variable_name.into();
        match self.name_uniform_locations.entry(variable_name.clone()) {
            Entry::Occupied(v) => Some(v.get().clone()),
            Entry::Vacant(v) => {
                let location = self.gl.get_uniform_location(&self.program, &variable_name);
                match location {
                    None => {
                        warn!(
                            target: "CompileProgram",
                            "failed to get uniform location {}", variable_name
                        );
                        None
                    }
                    Some(location) => {
                        let location = v.insert(location);
                        Some(location.clone())
                    }
                }
            },
        }
    }

    /// Returns variable names to uniform struct field locations mapping.
    pub fn uniform_structural_locations(&self) -> &HashMap<String, WebGlUniformLocation> {
        &self.name_uniform_structural_locations
    }

    /// Returns blovk names to uniform block indices mapping.
    pub fn uniform_block_indices(&self) -> &HashMap<String, u32> {
        &self.name_uniform_block_indices
    }

    /// Returns [`AttributeBinding`] to attribute locations mapping.
    pub fn binding_attribute_locations(&self) -> &HashMap<AttributeBinding, GLuint> {
        &self.binding_attribute_locations
    }

    /// Returns [`UniformBinding`] to uniform locations mapping.
    pub fn binding_uniform_locations(&self) -> &HashMap<UniformBinding, WebGlUniformLocation> {
        &self.binding_uniform_locations
    }

    /// Returns [`UniformStructuralBinding`] to uniform struct field locations mapping.
    pub fn binding_uniform_structural_locations(
        &self,
    ) -> &HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>> {
        &self.binding_uniform_structural_locations
    }

    /// Returns [`UniformBlockBinding`] to uniform block indices mapping.
    pub fn binding_uniform_block_indices(&self) -> &HashMap<UniformBlockBinding, u32> {
        &self.binding_uniform_block_indices
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

        if let Some(using_program) = self.using_program.as_ref() {
            unsafe {
                let using_program = &mut **using_program;
                if using_program.name() == name {
                    return Ok(using_program);
                }
            }
        }

        unsafe {
            let program = match self.store.entry(name.to_string()) {
                Entry::Occupied(mut occupied) => Ok(&mut **occupied.get_mut()),
                Entry::Vacant(vacant) => {
                    let program =
                        compile_program(&self.gl, source, vertex_defines, fragment_defines)?;
                    let program: *mut Program = Box::leak(Box::new(program));
                    let program = vacant.insert(program);
                    Ok(&mut **program)
                }
            }?;
            self.gl.use_program(Some(&(*program).program));
            self.using_program = Some(program);
            Ok(&mut *program)
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
    let (name_attribute_locations, binding_attribute_locations) =
        collect_attribute_locations(gl, &program, source.attribute_bindings().as_slice());
    let (name_uniform_locations, binding_uniform_locations) =
        collect_uniform_locations(gl, &program, source.uniform_bindings().as_slice());
    let (name_uniform_block_indices, binding_uniform_block_indices) =
        collect_uniform_block_indices(gl, &program, source.uniform_block_bindings().as_slice());
    let (name_uniform_structural_locations, binding_uniform_structural_locations) =
        collect_uniform_structural_locations(
            gl,
            &program,
            source.uniform_structural_bindings().as_slice(),
        );
    Ok(Program {
        name: source.name().to_string(),

        name_attribute_locations,
        name_uniform_locations,
        name_uniform_structural_locations,
        name_uniform_block_indices,

        binding_attribute_locations,
        binding_uniform_locations,
        binding_uniform_structural_locations,
        binding_uniform_block_indices,

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

fn collect_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[AttributeBinding],
) -> (HashMap<String, GLuint>, HashMap<AttributeBinding, GLuint>) {
    let mut name_locations = HashMap::with_capacity(bindings.len());
    let mut binding_locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
            warn!(
                target: "CompileProgram",
                "failed to get attribute location {}", variable_name
            );
        } else {
            name_locations.insert(binding.variable_name().to_string(), location as GLuint);
            binding_locations.insert(binding.clone(), location as GLuint);
        }
    });

    (name_locations, binding_locations)
}

fn collect_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBinding],
) -> (
    HashMap<String, WebGlUniformLocation>,
    HashMap<UniformBinding, WebGlUniformLocation>,
) {
    let mut name_locations = HashMap::with_capacity(bindings.len());
    let mut binding_locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_uniform_location(program, variable_name);
        match location {
            None => {
                warn!(
                    target: "CompileProgram",
                    "failed to get uniform location {}", variable_name
                );
            }
            Some(location) => {
                name_locations.insert(binding.variable_name().to_string(), location.clone());
                binding_locations.insert(binding.clone(), location);
            }
        }
    });

    (name_locations, binding_locations)
}

fn collect_uniform_structural_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformStructuralBinding],
) -> (
    HashMap<String, WebGlUniformLocation>,
    HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>>,
) {
    let mut name_locations = HashMap::with_capacity(bindings.len());
    let mut binding_locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let array_len = binding.array_len();
        let fields = binding.fields();
        let mut field_locations = HashMap::with_capacity(fields.len());
        fields.iter().for_each(|field| {
            match array_len {
                Some(len) => {
                    for index in 0..len {
                        let complete_field_name = format!("{}[{}].{}", variable_name, index, field);
                        let location = gl.get_uniform_location(program, &complete_field_name);
                        match location {
                            None => {
                                warn!(
                                    target: "CompileProgram",
                                    "failed to get uniform location {}", complete_field_name
                                );
                            }
                            Some(location) => {
                                name_locations
                                    .insert(complete_field_name.clone(), location.clone());
                                field_locations.insert(complete_field_name, location);
                            }
                        }
                    }
                }
                None => {
                    let complete_field_name = format!("{}.{}", variable_name, field);
                    let location = gl.get_uniform_location(program, &complete_field_name);
                    match location {
                        None => {
                            warn!(
                                target: "CompileProgram",
                                "failed to get uniform location {}", complete_field_name
                            );
                        }
                        Some(location) => {
                            name_locations.insert(complete_field_name.clone(), location.clone());
                            field_locations.insert(complete_field_name, location);
                        }
                    }
                }
            };
        });
        binding_locations.insert(binding.clone(), field_locations);
    });

    (name_locations, binding_locations)
}

pub fn collect_uniform_block_indices(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBlockBinding],
) -> (HashMap<String, u32>, HashMap<UniformBlockBinding, u32>) {
    let mut name_indices = HashMap::with_capacity(bindings.len());
    let mut binding_indices = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.block_name();
        let index = gl.get_uniform_block_index(program, variable_name);
        name_indices.insert(binding.block_name().to_string(), index);
        binding_indices.insert(binding.clone(), index);
    });

    (name_indices, binding_indices)
}
