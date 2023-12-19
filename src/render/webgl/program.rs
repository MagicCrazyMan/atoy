use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    rc::Rc,
};

use wasm_bindgen_test::console_log;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::{
    attribute::AttributeBinding,
    conversion::GLuint,
    error::Error,
    uniform::{UniformBinding, UniformBlockBinding},
};

/// Source providing basic data for compiling a [`WebGlProgram`].
/// Data provided by program source should never change even in different condition.
pub trait ProgramSource {
    /// Program name, should be unique,
    fn name(&self) -> &'static str;

    /// Shader sources, at least one vertex shader and one fragment shader should be specified.
    fn sources<'a>(&'a self) -> &[ShaderSource<'a>];

    /// Attribute binding variable name.
    fn attribute_bindings(&self) -> &[AttributeBinding];

    /// Uniform binding variable names.
    fn uniform_bindings(&self) -> &[UniformBinding];

    /// Uniform block binding variable names.
    fn uniform_block_bindings(&self) -> &[UniformBlockBinding];
}

/// Shader source codes.
#[derive(Debug, Clone)]
pub enum ShaderSource<'a> {
    Vertex(&'a str),
    Fragment(&'a str),
}

/// Compiled program item.
#[derive(Clone)]
pub struct ProgramItem {
    name: String,
    program: WebGlProgram,
    // shaders: Vec<WebGlShader>,
    attributes: Rc<HashMap<AttributeBinding, GLuint>>,
    uniform_locations: Rc<HashMap<UniformBinding, WebGlUniformLocation>>,
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
        )?),
        uniform_locations: Rc::new(collect_uniform_locations(
            gl,
            &program,
            source.uniform_bindings(),
        )?),
        uniform_block_indices: Rc::new(collect_uniform_block_indices(
            gl,
            &program,
            source.uniform_block_bindings(),
        )),
        program,
        // shaders,
    })
}

// fn delete_program(gl: &WebGl2RenderingContext, material: &ProgramItem) {
//     let ProgramItem {
//         program, shaders, ..
//     } = material;
//     gl.use_program(None);
//     shaders.into_iter().for_each(|shader| {
//         gl.delete_shader(Some(&shader));
//     });
//     gl.delete_program(Some(&program));
// }

/// Compiles [`WebGlShader`] by [`ShaderSource`].
pub fn compile_shaders(
    gl: &WebGl2RenderingContext,
    source: &ShaderSource,
) -> Result<WebGlShader, Error> {
    let (shader, code) = match source {
        ShaderSource::Vertex(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                .ok_or(Error::CreateVertexShaderFailure)?;
            (shader, code)
        }
        ShaderSource::Fragment(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                .ok_or(Error::CreateFragmentShaderFailure)?;
            (shader, code)
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
    let program = gl.create_program().ok_or(Error::CreateProgramFailure)?;

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
) -> Result<HashMap<AttributeBinding, GLuint>, Error> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
            console_log!("failed to get attribute location of {}", variable_name);
        } else {
            locations.insert(binding.clone(), location as GLuint);
        }
    });

    Ok(locations)
}

fn collect_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBinding],
) -> Result<HashMap<UniformBinding, WebGlUniformLocation>, Error> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.variable_name();
        let location = gl.get_uniform_location(program, variable_name);
        match location {
            None => {
                // should log warning
                console_log!("failed to get uniform location of {}", variable_name);
            }
            Some(location) => {
                locations.insert(binding.clone(), location);
            }
        }
    });

    Ok(locations)
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
