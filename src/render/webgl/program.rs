use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
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
    fn name(&self) -> &'static str;

    /// Shader sources, at least one vertex shader and one fragment shader should be specified.
    fn sources(&self) -> Vec<ShaderSource>;

    /// Attribute binding variable name.
    fn attribute_bindings(&self) -> &[AttributeBinding];

    /// Uniform variable bindings.
    fn uniform_bindings(&self) -> &[UniformBinding];

    /// Uniform structural variable bindings.
    fn uniform_structural_bindings(&self) -> &[UniformStructuralBinding];

    /// Uniform block variable bindings.
    fn uniform_block_bindings(&self) -> &[UniformBlockBinding];
}

pub enum Shader {}

/// Shader source codes. 3 available types:
///
/// 1. `Builder`, build shader source code from a [`ShaderBuilder`].
/// 2. `VertexRaw`, developer provides a complete vertex shader source code.
/// 3. `FragmentRaw`, developer provides a complete fragment shader source code.
#[derive(Clone)]
pub enum ShaderSource {
    Builder(ShaderBuilder),
    VertexRaw(&'static str),
    FragmentRaw(&'static str),
}

/// Compiled program item.
#[derive(Clone)]
pub struct ProgramItem {
    name: String,
    program: WebGlProgram,
    shaders: Vec<WebGlShader>,
    attributes: Rc<HashMap<AttributeBinding, GLuint>>,
    uniform_locations: Rc<HashMap<UniformBinding, WebGlUniformLocation>>,
    uniform_structural_locations:
        Rc<HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>>>,
    uniform_block_indices: Rc<HashMap<UniformBlockBinding, u32>>,
}

impl ProgramItem {
    /// Returns program source name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the native [`WebGlProgram`].
    pub fn gl_program(&self) -> &WebGlProgram {
        &self.program
    }

    /// Returns attribute locations.
    pub fn attribute_locations(&self) -> &HashMap<AttributeBinding, GLuint> {
        &self.attributes
    }

    /// Returns uniform locations.
    pub fn uniform_locations(&self) -> &HashMap<UniformBinding, WebGlUniformLocation> {
        &self.uniform_locations
    }

    /// Returns uniform struct field locations.
    pub fn uniform_structural_locations(
        &self,
    ) -> &HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>> {
        &self.uniform_structural_locations
    }

    /// Returns uniform block indices.
    pub fn uniform_block_indices(&self) -> &HashMap<UniformBlockBinding, u32> {
        &self.uniform_block_indices
    }
}

/// A centralized program store storing and caching compiled program item.
/// Program store caches a program by an unique name provided by [`ProgramSource::name`].
pub struct ProgramStore {
    gl: WebGl2RenderingContext,
    store: HashMap<String, ProgramItem>,
}

impl ProgramStore {
    /// Constructs a new program store.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: HashMap::new(),
        }
    }

    /// Returns a [`ProgramItem`] by a [`ProgramSource`]. If not exist, compiles and stores it.
    pub fn use_program<S>(&mut self, source: &S) -> Result<ProgramItem, Error>
    where
        S: ProgramSource + ?Sized,
    {
        let store = &mut self.store;

        match store.entry(source.name().to_string()) {
            Entry::Occupied(occupied) => Ok(occupied.get().clone()),
            Entry::Vacant(vacant) => {
                let item = vacant.insert(compile_program(&self.gl, source)?);
                Ok(item.clone())
            }
        }
    }

    /// Deletes a [`ProgramItem`].
    pub fn delete_program(&mut self, name: &str) {
        let Some(program_item) = self.store.remove(name) else {
            return;
        };

        delete_program(&self.gl, &program_item);
    }
}

/// Compiles a [`WebGlProgram`] from a [`ProgramSource`].
pub fn compile_program<S>(gl: &WebGl2RenderingContext, source: &S) -> Result<ProgramItem, Error>
where
    S: ProgramSource + ?Sized,
{
    let mut shaders = Vec::with_capacity(source.sources().len());
    source.sources().iter().try_for_each(|source| {
        shaders.push(compile_shaders(gl, source)?);
        Ok(()) as Result<(), Error>
    })?;

    let program = create_program(gl, &shaders)?;
    Ok(ProgramItem {
        name: source.name().to_string(),
        attributes: Rc::new(collect_attribute_locations(
            gl,
            &program,
            source.attribute_bindings(),
        )),
        uniform_locations: Rc::new(collect_uniform_locations(
            gl,
            &program,
            source.uniform_bindings(),
        )),
        uniform_block_indices: Rc::new(collect_uniform_block_indices(
            gl,
            &program,
            source.uniform_block_bindings(),
        )),
        uniform_structural_locations: Rc::new(collect_uniform_structural_locations(
            gl,
            &program,
            source.uniform_structural_bindings(),
        )),
        program,
        shaders,
    })
}

fn delete_program(gl: &WebGl2RenderingContext, program_item: &ProgramItem) {
    let ProgramItem {
        program, shaders, ..
    } = program_item;
    gl.use_program(None);
    shaders.into_iter().for_each(|shader| {
        gl.delete_shader(Some(&shader));
    });
    gl.delete_program(Some(&program));
}

/// Compiles [`WebGlShader`] by [`ShaderSource`].
pub fn compile_shaders(
    gl: &WebGl2RenderingContext,
    source: &ShaderSource,
) -> Result<WebGlShader, Error> {
    let (shader, code) = match source {
        ShaderSource::Builder(builder) => match builder.shader_type() {
            ShaderType::Vertex => (
                gl.create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                    .ok_or(Error::CreateVertexShaderFailed)?,
                Cow::Owned(builder.build()),
            ),
            ShaderType::Fragment => (
                gl.create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                    .ok_or(Error::CreateVertexShaderFailed)?,
                Cow::Owned(builder.build()),
            ),
        },
        ShaderSource::VertexRaw(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                .ok_or(Error::CreateVertexShaderFailed)?;
            (shader, Cow::Borrowed(*code))
        }
        ShaderSource::FragmentRaw(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                .ok_or(Error::CreateFragmentShaderFailed)?;
            (shader, Cow::Borrowed(*code))
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
        Err(Error::CompileShaderFailure(err))
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
        Err(Error::CompileProgramFailure(err))
    } else {
        Ok(program)
    }
}

fn collect_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[AttributeBinding],
) -> HashMap<AttributeBinding, GLuint> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
            warn!(
                target: "CompileProgram",
                "failed to get attribute location of {}", variable_name
            );
        } else {
            locations.insert(binding.clone(), location as GLuint);
        }
    });

    locations
}

fn collect_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBinding],
) -> HashMap<UniformBinding, WebGlUniformLocation> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_uniform_location(program, variable_name);
        match location {
            None => {
                warn!(
                    target: "CompileProgram",
                    "failed to get uniform location of {}", variable_name
                );
            }
            Some(location) => {
                locations.insert(binding.clone(), location);
            }
        }
    });

    locations
}

fn collect_uniform_structural_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformStructuralBinding],
) -> HashMap<UniformStructuralBinding, HashMap<String, WebGlUniformLocation>> {
    let mut locations = HashMap::with_capacity(bindings.len());

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
                                    "failed to get uniform location of {}", complete_field_name
                                );
                            }
                            Some(location) => {
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
                                "failed to get uniform location of {}", complete_field_name
                            );
                        }
                        Some(location) => {
                            field_locations.insert(complete_field_name, location);
                        }
                    }
                }
            };
        });
        locations.insert(binding.clone(), field_locations);
    });

    locations
}

pub fn collect_uniform_block_indices(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBlockBinding],
) -> HashMap<UniformBlockBinding, u32> {
    let mut indices = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let index = gl.get_uniform_block_index(program, variable_name);
        indices.insert(*binding, index);
    });

    indices
}
