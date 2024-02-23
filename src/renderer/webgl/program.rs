use std::{borrow::Cow, iter::FromIterator};

use hashbrown::{HashMap, HashSet};
use log::warn;
use regex::Regex;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use super::error::Error;

/// Custom derivative prefix for injecting code snippet when creating program using [`ProgramStore`].
pub const GLSL_REPLACEMENT_DERIVATIVE: &'static str = "#include";
/// Replacement derivative name for injecting [`ShaderProvider::vertex_defines`] and
/// [`ShaderProvider::fragment_defines`] when creating program using [`ProgramStore`].
pub const GLSL_REPLACEMENT_DEFINES: &'static str = "Defines";
/// Regular expression for matching replacement macro `#include <snippet_name>;`.
const GLSL_REPLACEMENT_DERIVATIVE_REGEX: &'static str = "^\\s*#include\\s+(.+)\\s*$";

/// GLSL `#define` macro definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Define<'a> {
    /// Define macro with value, build to `#define <name> <value>`.
    WithValue(Cow<'a, str>, Cow<'a, str>),
    /// Define macro without value, build to `#define <name>`.
    WithoutValue(Cow<'a, str>),
}

impl<'a> Define<'a> {
    /// Returns name of define macro.
    pub fn name(&self) -> &str {
        match self {
            Define::WithValue(name, _) | Define::WithoutValue(name) => &name,
        }
    }

    /// Returns value of define macro.
    pub fn value(&self) -> Option<&str> {
        match self {
            Define::WithValue(_, value) => Some(&value),
            Define::WithoutValue(_) => None,
        }
    }

    /// Builds to GLSL define macro derivative.
    pub fn build(&self) -> String {
        match self {
            Define::WithValue(name, value) => format!("#define {} {}", name, value),
            Define::WithoutValue(name) => format!("#define {}", name),
        }
    }
}

/// Source providing basic data for compiling a [`WebGlProgram`].
pub trait ShaderProvider {
    /// Global unique name for the program source.
    fn name(&self) -> Cow<'_, str>;

    /// Returns source code of vertex shader.
    fn vertex_source(&self) -> Cow<'_, str>;

    /// Returns source code of fragment shader.
    fn fragment_source(&self) -> Cow<'_, str>;

    /// Returns universal defines macros for both vertex and fragment shaders.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of vertex shader to make this work.
    fn universal_defines(&self) -> &[Define<'_>];

    /// Returns defines macros for vertex shader.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of vertex shader to make this work.
    fn vertex_defines(&self) -> &[Define<'_>];

    /// Returns defines macros for fragment shader.
    /// [`GLSL_REPLACEMENT_DEFINES`] should be placed once and only once in source code of fragment shader to make this work.
    fn fragment_defines(&self) -> &[Define<'_>];

    /// Returns self-associated GLSL code snippet by name.
    fn snippet(&self, name: &str) -> Option<Cow<'_, str>>;
}

/// Compiled program item.
pub struct Program {
    name: String,
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

    /// Returns attribute locations.
    pub fn attribute_locations(&self) -> &HashMap<String, u32> {
        &self.attribute_locations
    }

    /// Returns uniform locations by a variable name.
    pub fn uniform_locations(&self) -> &HashMap<String, WebGlUniformLocation> {
        &self.uniform_locations
    }

    /// Returns uniform block index by a uniform block name.
    pub fn uniform_block_indices(&mut self) -> &HashMap<String, u32> {
        &self.uniform_block_indices
    }
}

/// A centralized program store for storing and caching compiled [`ShaderProvider`].
pub struct ProgramStore {
    gl: WebGl2RenderingContext,
    store: HashMap<String, *mut Program>,
    using_program: Option<*mut Program>,

    replacement_regex: Regex,
    snippets: HashMap<String, String>,
}

impl ProgramStore {
    /// Constructs a new program store.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_snippets(gl, [])
    }

    /// Constructs a new program store with GLSL code snippets.
    pub fn with_snippets<'a, I: IntoIterator<Item = (Cow<'a, str>, Cow<'a, str>)>>(
        gl: WebGl2RenderingContext,
        snippet: I,
    ) -> Self {
        let snippets = snippet
            .into_iter()
            .map(|(n, s)| (n.into_owned(), s.into_owned()));

        Self {
            gl,
            store: HashMap::new(),
            using_program: None,

            replacement_regex: Regex::new(GLSL_REPLACEMENT_DERIVATIVE_REGEX).unwrap(),
            snippets: HashMap::from_iter(snippets),
        }
    }

    /// Returns GLSL code snippet by name.
    pub fn snippet(&self, name: &str) -> Option<&str> {
        match self.snippets.get(name) {
            Some(snippet) => Some(snippet.as_str()),
            None => None,
        }
    }

    /// Adds new GLSL code snippet with an unique name.
    /// Returns the old one if exists.
    pub fn add_snippet<N, S>(&mut self, name: N, snippet: S) -> Option<String>
    where
        N: Into<String>,
        S: Into<String>,
    {
        let name: String = name.into();
        let name = name.trim().to_string();
        self.snippets.insert(name, snippet.into())
    }

    /// Removes GLSL code snippet by name.
    pub fn remove_snippet(&mut self, name: &str) -> Option<String> {
        self.snippets.remove(name.trim())
    }

    /// Clears all code snippets.
    pub fn clear_snippets(&mut self) {
        self.snippets.clear();
    }

    fn replace_snippets<'a, 'b, S>(&self, provider: &'b S, is_vertex: bool) -> String
    where
        S: ShaderProvider + ?Sized,
    {
        let (code, universal_defines, defines) = match is_vertex {
            true => (
                provider.vertex_source(),
                provider.universal_defines(),
                provider.vertex_defines(),
            ),
            false => (
                provider.fragment_source(),
                provider.universal_defines(),
                provider.fragment_defines(),
            ),
        };

        // evaluated output code length
        let mut evaluated_len = code.len();
        for define in universal_defines.iter().chain(defines.iter()) {
            evaluated_len +=
                define.name().len() + define.value().map(|value| value.len()).unwrap_or(0) + 10;
        }
        let mut output = String::with_capacity(evaluated_len);

        let mut appended_snippets = HashSet::new();
        for line in code.lines() {
            let Some(matched) = self
                .replacement_regex
                .captures(line)
                .and_then(|captures| captures.get(1))
            else {
                output.push_str(line);
                if !line.ends_with("\n") {
                    output.push('\n');
                }
                continue;
            };

            let name = matched.as_str().trim();
            if appended_snippets.contains(name) {
                continue;
            }

            if name == GLSL_REPLACEMENT_DEFINES {
                for define in universal_defines.iter().chain(defines.iter()) {
                    output.push_str(&define.build());
                    output.push('\n');
                }
            } else {
                // finds snippet, finds from provider-associated first, otherwise finds from store
                let Some(snippet) = provider.snippet(name).map(|snippet| snippet).or_else(|| {
                    self.snippets
                        .get(name)
                        .map(|snippet| Cow::Borrowed(snippet.as_str()))
                }) else {
                    warn!(
                        target: "ProgramStore",
                        "code snippet with name `{}` not found",
                        name
                    );
                    continue;
                };

                output.push_str(&snippet);
                output.push('\n');
            }
            appended_snippets.insert(name);
        }
        output
    }

    fn compile<'a, 'b, S>(&'a mut self, name: String, provider: &'b S) -> Result<Program, Error>
    where
        S: ShaderProvider + ?Sized,
    {
        let vertex_code = self.replace_snippets(provider, true);
        let vertex_shader = compile_shader(&self.gl, true, &vertex_code)?;

        let fragment_code = self.replace_snippets(provider, false);
        let fragment_shader: WebGlShader = compile_shader(&self.gl, false, &fragment_code)?;

        let program = create_program(&self.gl, &vertex_shader, &fragment_shader)?;
        let attribute_locations = collects_attribute_locations(&self.gl, &program);
        let uniform_locations = collects_uniform_locations(&self.gl, &program);
        let uniform_block_indices = collects_uniform_block_index(&self.gl, &program);
        Ok(Program {
            name,

            program,
            vertex_shader,
            fragment_shader,

            attribute_locations,
            uniform_locations,
            uniform_block_indices,
        })
    }

    /// Uses a program from a program source.
    /// Program will be compiled if it is used for the first time.
    pub fn use_program<'a, 'b, 'c, S>(&'a mut self, source: &'b S) -> Result<&'c mut Program, Error>
    where
        S: ShaderProvider + ?Sized,
    {
        unsafe {
            let name = source.name();

            // checks using
            if let Some(using_program) = self.using_program.as_ref() {
                let using_program = &mut **using_program;
                if using_program.name() == name {
                    return Ok(using_program);
                }
            }

            // checks cache
            if let Some(program) = self.store.get(name.as_ref()) {
                let program = &mut **program;
                self.gl.use_program(Some(&program.program));
                self.using_program = Some(program);
                return Ok(program);
            }

            let name = name.to_string();
            let program = Box::leak(Box::new(self.compile(name.clone(), source)?));
            self.store
                .insert_unique_unchecked(name, program as *mut Program);

            self.gl.use_program(Some(program.program()));
            self.using_program = Some(program);

            Ok(program)
        }
    }

    /// Unuses a program.
    pub fn unuse_program(&mut self) {
        self.gl.use_program(None);
        self.using_program = None;
    }

    /// Deletes a cached program by unique name.
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
        let gl = self.gl.clone();

        gl.use_program(None);
        self.store.drain().for_each(|(_, program)| unsafe {
            let program = Box::from_raw(program);
            delete_program(&gl, *program);
        });
        self.using_program.take();
    }
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
    is_vertex: bool,
    code: &str,
) -> Result<WebGlShader, Error> {
    // log::info!("{}", code);
    let shader = gl
        .create_shader(if is_vertex {
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
        return Err(Error::CompileProgramFailure(err));
    }

    Ok(program)
}

/// Collects active attribute locations.
pub fn collects_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<String, u32> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_ATTRIBUTES)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);
    for location in 0..num {
        let Some(info) = gl.get_active_attrib(&program, location) else {
            continue;
        };

        locations.insert(info.name(), location);
    }

    locations
}

/// Collects active uniform locations.
pub fn collects_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<String, WebGlUniformLocation> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_UNIFORMS)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);
    for index in 0..num {
        let Some(info) = gl.get_active_uniform(&program, index) else {
            continue;
        };
        // if we have uniform block in code, getActiveUniform may return index of uniform inside uniform block,
        // while getUniformLocation can not get its location.
        let Some(location) = gl.get_uniform_location(&program, &info.name()) else {
            continue;
        };

        locations.insert(info.name(), location);
    }

    locations
}

/// Collects active uniform block indices.
pub fn collects_uniform_block_index(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> HashMap<String, u32> {
    let mut locations = HashMap::new();

    let num = gl
        .get_program_parameter(&program, WebGl2RenderingContext::ACTIVE_UNIFORM_BLOCKS)
        .as_f64()
        .map(|v| v as u32)
        .unwrap_or(0);

    for location in 0..num {
        let Some(name) = gl.get_active_uniform_block_name(&program, location) else {
            continue;
        };

        locations.insert(name, location);
    }

    locations
}
