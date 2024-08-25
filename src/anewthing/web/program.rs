use std::{borrow::Cow, cell::LazyCell, hash::Hash, ops::Range};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use line_span::LineSpanExt;
use log::warn;
use regex::Regex;
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::{anewthing::web::error::Error, renderer::webgl::conversion::ToGlEnum};

/// Available shader types for WebGL 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderType {
    /// Vertex Shader.
    Vertex,
    /// Fragment Shader.
    Fragment,
}

impl ToGlEnum for ShaderType {
    fn gl_enum(&self) -> u32 {
        match self {
            ShaderType::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            ShaderType::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ShaderName {
    Custom(Cow<'static, str>),
}

pub trait ShaderSource {
    /// Global unique name for this shader source.
    fn name(&self) -> ShaderName;

    /// Returns the source code of the shader.
    fn code(&self) -> &str;

    /// Returns a custom snippet code by name.
    fn snippet(&self, name: &str) -> Option<&str>;

    /// Returns a custom define value by name.
    fn define_value(&self, name: &str) -> Option<&str>;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum PragmaOperation {
    InjectSnippet,
}

impl PragmaOperation {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "inject" => Some(PragmaOperation::InjectSnippet),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ShaderCacheKey {
    shader_type: ShaderType,
    name: ShaderName,
}

struct DefinePosition {
    line_index: usize,
    /// position in line, not in the whole code string
    name_position: Range<usize>,
    value_position: Option<Range<usize>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Define<'a> {
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
}

#[derive(Clone)]
struct Shader {
    id: Uuid,
    shader: WebGlShader,
}

struct ShaderCache {
    code: String,
    line_ranges: Vec<Range<usize>>,
    defines: Vec<DefinePosition>,
    variants: HashMap<Vec<Define<'static>>, Shader>,
}

struct ShaderSnippet {
    code: Cow<'static, str>,
    lines: Vec<Range<usize>>,
}

struct ShaderManager {
    gl: WebGl2RenderingContext,
    caches: HashMap<ShaderCacheKey, ShaderCache>,
    snippets: HashMap<Cow<'static, str>, ShaderSnippet>,
}

impl ShaderManager {
    /// Constructs a new shader manager.
    fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            caches: HashMap::new(),
            snippets: HashMap::new(),
        }
    }

    /// Returns a snippet code.
    fn snippet(&mut self, name: &str) -> Option<&str> {
        self.snippets.get(name).map(|snippet| snippet.code.as_ref())
    }

    /// Adds a new snippet code to manager. Returns the previous snippet code if occupied.
    fn add_snippet(
        &mut self,
        name: Cow<'static, str>,
        code: Cow<'static, str>,
    ) -> Option<Cow<'static, str>> {
        let lines = code
            .as_ref()
            .line_spans()
            .map(|span| span.range())
            .collect();
        self.snippets
            .insert(name, ShaderSnippet { code, lines })
            .map(|snippet| snippet.code)
    }

    /// Removes a snippet code from manager.
    fn remove_snippet(&mut self, name: &str) -> Option<Cow<'static, str>> {
        self.snippets.remove(name).map(|snippet| snippet.code)
    }

    /// Returns a compiled [`Shader`] from a [`ShaderSource`] under specified [`ShaderType`].
    ///
    /// Manager identifies shader as different variants by values of define directives in the shader code.
    /// A cached shader is returned if it has been compiled before.
    fn get_or_compile_shader<S>(
        &mut self,
        shader_type: ShaderType,
        shader_source: &S,
    ) -> Result<Shader, Error>
    where
        S: ShaderSource,
    {
        let key = ShaderCacheKey {
            shader_type,
            name: shader_source.name(),
        };
        let cache = match self.caches.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                entry.insert(Self::create_cache(&self.snippets, shader_source)?)
            }
        };
        Self::get_or_compile_variant_shader(cache, &self.gl, shader_type, shader_source)
    }

    /// Creates a shader cache from a [`ShaderSource`].
    fn create_cache<S>(
        snippets: &HashMap<Cow<'static, str>, ShaderSnippet>,
        shader_source: &S,
    ) -> Result<ShaderCache, Error>
    where
        S: ShaderSource,
    {
        let mut lines = shader_source.code().lines().collect::<Vec<_>>();
        Self::prepare_pragmas(snippets, &mut lines, shader_source)?;

        let code = lines.join("\n");
        let line_ranges = code
            .line_spans()
            .map(|line| line.range())
            .collect::<Vec<_>>();
        let defines = Self::collect_defines(&code, &line_ranges);

        let cache = ShaderCache {
            code,
            line_ranges,
            defines,
            variants: HashMap::new(),
        };

        Ok(cache)
    }

    /// Prepares pragmas.
    fn prepare_pragmas<'a, 'b: 'a, S>(
        snippets: &'a HashMap<Cow<'static, str>, ShaderSnippet>,
        lines: &mut Vec<&'a str>,
        shader_source: &'b S,
    ) -> Result<(), Error>
    where
        S: ShaderSource,
    {
        /// Regex for extracting pragma operation from `#pragma <operation> <value>` directive.
        const PRAGMA_REGEX: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(r"^\s*#pragma\s+(?P<operation>\w+)\s+(?P<value>.+)\s*$").unwrap()
        });

        let mut injecteds: HashSet<&str> = HashSet::new();
        let mut i = 0;
        while i <= lines.len() {
            let line = lines[i];

            let Some(captures) = PRAGMA_REGEX.captures(line) else {
                i += 1;
                continue;
            };
            let Some(operation) = captures
                .name("operation")
                .and_then(|matched| PragmaOperation::from_str(matched.as_str()))
            else {
                i += 1;
                continue;
            };

            match operation {
                PragmaOperation::InjectSnippet => {
                    let Some(name) = captures
                        .name("value")
                        .map(|matched| matched.as_str().trim())
                    else {
                        i += 1;
                        continue;
                    };

                    if injecteds.contains(name) {
                        warn!(target: "ShaderManager", "snippet '{}' inject more than once", name);
                        lines.remove(i);
                        // no need to accumulate line index
                    } else {
                        if let Some(snippet) = shader_source.snippet(name) {
                            lines.splice(i..=i, snippet.lines().map(|line| line));
                            // no need to accumulate line index
                        } else if let Some(snippet) = snippets.get(name) {
                            lines.splice(
                                i..=i,
                                snippet
                                    .lines
                                    .iter()
                                    .map(|line_range| &snippet.code[line_range.clone()]),
                            );
                            // no need to accumulate line index
                        } else {
                            return Err(Error::SnippetNotFound(name.to_string()));
                        }

                        injecteds.insert_unique_unchecked(name);
                    }
                }
            }
        }

        Ok(())
    }

    /// Collects define directives from lines of shader code
    fn collect_defines(code: &str, lines: &[Range<usize>]) -> Vec<DefinePosition> {
        /// Regex for extracting defines from `#define <name> [<value>]` directive. value is optional.
        const DEFINE_REGEX: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(r"^\s*#define\s+(?P<name>\w+)\s*(?P<value>.*)\s*$").unwrap()
        });

        let mut defines = Vec::new();
        lines
            .into_iter()
            .enumerate()
            .for_each(|(line_index, line_range)| {
                let line = &code[line_range.clone()];
                let Some(captures) = DEFINE_REGEX.captures(line) else {
                    return;
                };
                let Some(name_position) = captures.name("name").map(|matched| matched.range())
                else {
                    return;
                };
                let value_position = captures.name("value").and_then(|matched| {
                    let range = matched.range();
                    if line[range.clone()].is_empty() {
                        None
                    } else {
                        Some(range)
                    }
                });

                defines.push(DefinePosition {
                    line_index,
                    name_position,
                    value_position,
                });
            });

        defines
    }

    fn get_or_compile_variant_shader<S>(
        cache: &mut ShaderCache,
        gl: &WebGl2RenderingContext,
        shader_type: ShaderType,
        shader_source: &S,
    ) -> Result<Shader, Error>
    where
        S: ShaderSource,
    {
        let line_ranges = &cache.line_ranges;
        let mut replaced_defines = Vec::new();
        let defines = cache
            .defines
            .iter()
            .enumerate()
            .map(|(define_index, define_position)| {
                let DefinePosition {
                    line_index,
                    name_position,
                    value_position,
                } = define_position;
                let line_range = &line_ranges[*line_index];
                let line = &cache.code[line_range.clone()];

                let name = &line[name_position.clone()];
                let value = match shader_source.define_value(name) {
                    Some(value) => {
                        replaced_defines.push((define_index, define_position));
                        let value = value.trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value)
                        }
                    }
                    None => match value_position {
                        Some(value_position) => Some(&line[value_position.clone()]), // value is ensured to be non-empty
                        None => None,
                    },
                };

                Define {
                    name: Cow::Borrowed(name),
                    value: value.map(|value| Cow::Borrowed(value)),
                }
            })
            .collect::<Vec<_>>();

        if let Some(variant) = cache.variants.get(&defines) {
            Ok(variant.clone())
        } else {
            let code = Self::create_variant_code(cache, &defines, &replaced_defines);
            let shader = Self::compile_shader(gl, shader_type, &code)?;

            // persists string slice to String
            let defines = defines
                .into_iter()
                .map(|define| Define {
                    name: Cow::Owned(define.name.to_string()),
                    value: define.value.map(|value| Cow::Owned(value.to_string())),
                })
                .collect::<Vec<_>>();

            Ok(cache
                .variants
                .insert_unique_unchecked(
                    defines,
                    Shader {
                        id: Uuid::new_v4(),
                        shader,
                    },
                )
                .1
                .clone())
        }
    }

    fn create_variant_code<'a>(
        cache: &'a ShaderCache,
        defines: &[Define],
        replaced_defines: &[(usize, &DefinePosition)],
    ) -> Cow<'a, str> {
        if replaced_defines.is_empty() {
            return Cow::Borrowed(&cache.code);
        }

        let mut lines = cache
            .line_ranges
            .iter()
            .map(|line_range| Cow::Borrowed(&cache.code[line_range.clone()]))
            .collect::<Vec<_>>();
        replaced_defines
            .into_iter()
            .for_each(|(define_index, define_position)| {
                let DefinePosition {
                    line_index,
                    value_position,
                    ..
                } = define_position;
                let Define { value, .. } = &defines[*define_index];
                let value = match value {
                    Some(value) => value.as_ref(),
                    None => "",
                };

                let line = &lines[*line_index];
                let mut replaced_line = String::with_capacity(line.len() + value.len() + 1); // 1 for a space
                replaced_line.push_str(&line);
                match value_position {
                    Some(range) => replaced_line.replace_range(range.clone(), value),
                    None => {
                        replaced_line.push_str(" ");
                        replaced_line.push_str(value);
                    }
                };
                lines[*line_index] = Cow::Owned(replaced_line);
            });
        Cow::Owned(lines.join("\n"))
    }

    fn compile_shader(
        gl: &WebGl2RenderingContext,
        shader_type: ShaderType,
        code: &str,
    ) -> Result<WebGlShader, Error> {
        let shader = gl
            .create_shader(shader_type.gl_enum())
            .ok_or(Error::CreateShaderFailure(shader_type))?;

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
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ProgramShaderKey {
    vertex_shader_id: Uuid,
    fragment_shader_id: Uuid,
}

#[derive(Clone)]
struct Program {
    program: WebGlProgram,
    attributes: HashMap<String, u32>,
    uniforms: HashMap<String, WebGlUniformLocation>,
    uniform_blocks: HashMap<String, u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramKey(Uuid);

// impl Program {
//     /// Returns native [`WebGlProgram`].
//     fn program(&self) -> &WebGlProgram {
//         &self.program
//     }

//     /// Returns all attributes name and location key-value paris.
//     fn attributes(&self) -> &HashMap<String, u32> {
//         &self.attributes
//     }

//     /// Returns all uniforms name and [`WebGlUniformLocation`] key-value paris.
//     fn uniforms(&self) -> &HashMap<String, WebGlUniformLocation> {
//         &self.uniforms
//     }

//     /// Returns all uniform blocks name and location key-value paris.
//     fn uniform_blocks(&self) -> &HashMap<String, u32> {
//         &self.uniform_blocks
//     }

//     /// Returns attribute location by attribute name.
//     fn attribute(&self, name: &str) -> Option<u32> {
//         self.attributes.get(name).cloned()
//     }

//     /// Returns uniform location by uniform name.
//     fn uniform(&self, name: &str) -> Option<WebGlUniformLocation> {
//         self.uniforms.get(name).cloned()
//     }

//     /// Returns uniform block location by uniform block name.
//     fn uniform_block(&self, name: &str) -> Option<u32> {
//         self.uniform_blocks.get(name).cloned()
//     }
// }

/// Program manager.
///
/// Once a program compiled, it is cached and not deletable.
pub struct ProgramManager {
    gl: WebGl2RenderingContext,
    shader_manager: ShaderManager,
    caches: Vec<Program>,
    uuid_keys: HashMap<ProgramKey, usize>,
    shader_keys: HashMap<ProgramShaderKey, ProgramKey>,

    program_in_used: Option<usize>,
}

impl ProgramManager {
    /// Constructs a new program manager.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            shader_manager: ShaderManager::new(gl.clone()),
            caches: Vec::new(),
            uuid_keys: HashMap::new(),
            shader_keys: HashMap::new(),
            program_in_used: None,
            gl,
        }
    }

    /// Returns a snippet code.
    pub fn snippet(&mut self, name: &str) -> Option<&str> {
        self.shader_manager.snippet(name)
    }

    /// Adds a new snippet code to manager. Returns the previous snippet code if occupied.
    pub fn add_snippet(
        &mut self,
        name: Cow<'static, str>,
        code: Cow<'static, str>,
    ) -> Option<Cow<'static, str>> {
        self.shader_manager.add_snippet(name, code)
    }

    /// Removes a snippet code from manager.
    pub fn remove_snippet(&mut self, name: &str) -> Option<Cow<'static, str>> {
        self.shader_manager.remove_snippet(name)
    }

    /// Returns a compiled [`Program`] from a vertex shader and a fragment shader.
    ///
    /// A cached program is returned if vertex shader and fragment shader are cached.
    pub fn get_or_compile_program<VS, FS>(
        &mut self,
        vertex: &VS,
        fragment: &FS,
    ) -> Result<&ProgramKey, Error>
    where
        VS: ShaderSource,
        FS: ShaderSource,
    {
        let Shader {
            id: vertex_shader_id,
            shader: vs,
        } = self
            .shader_manager
            .get_or_compile_shader(ShaderType::Vertex, vertex)?;
        let Shader {
            id: fragment_shader_id,
            shader: fs,
        } = self
            .shader_manager
            .get_or_compile_shader(ShaderType::Fragment, fragment)?;

        let cache_key = ProgramShaderKey {
            vertex_shader_id,
            fragment_shader_id,
        };
        let uuid_key = match self.shader_keys.entry(cache_key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let program = Self::create_program(&self.gl, &vs, &fs)?;
                let attributes = Self::collects_attributes(&self.gl, &program);
                let uniforms = Self::collects_uniforms(&self.gl, &program);
                let uniform_blocks = Self::collects_uniform_blocks(&self.gl, &program);
                let program = Program {
                    program,
                    attributes,
                    uniforms,
                    uniform_blocks,
                };
                self.caches.push(program);
                let uuid_key = ProgramKey(Uuid::new_v4());
                self.uuid_keys.insert(uuid_key, self.caches.len() - 1);
                entry.insert(uuid_key)
            }
        };
        Ok(uuid_key)
    }

    /// Binds and uses a program to [`WebGl2RenderingContext`].
    pub fn use_program(&mut self, key: &ProgramKey) -> Result<(), Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        if self.program_in_used == Some(index) {
            return Ok(());
        }

        let program = &self.caches[index];
        self.gl.use_program(Some(&program.program));
        self.program_in_used = Some(index);

        Ok(())
    }

    /// Unbinds current using program from [`WebGl2RenderingContext`].
    pub fn unuse_program(&mut self) {
        if let Some(_) = self.program_in_used.take() {
            self.gl.use_program(None);
        }
    }

    /// Sets using program of [`WebGl2RenderingContext`] to [`None`] forcedly.
    pub fn unuse_program_force(&mut self) {
        self.gl.use_program(None);
        self.program_in_used = None;
    }

    // pub fn bind

    /// Returns all attributes name and location key-value pairs.
    pub fn attributes_locations(&self, key: &ProgramKey) -> Result<HashMap<String, u32>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].attributes.clone())
    }

    /// Returns all uniforms name and [`WebGlUniformLocation`] key-value pairs.
    pub fn uniforms_locations(
        &self,
        key: &ProgramKey,
    ) -> Result<HashMap<String, WebGlUniformLocation>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].uniforms.clone())
    }

    /// Returns all uniform blocks name and location key-value pairs.
    pub fn uniform_blocks_locations(
        &self,
        key: &ProgramKey,
    ) -> Result<HashMap<String, u32>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].uniform_blocks.clone())
    }

    /// Returns the attribute location of a specified attribute name.
    pub fn attribute_location(&self, key: &ProgramKey, name: &str) -> Result<Option<u32>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].attributes.get(name).cloned())
    }

    /// Returns the uniform location of a specified uniform name.
    pub fn uniform_location(
        &self,
        key: &ProgramKey,
        name: &str,
    ) -> Result<Option<WebGlUniformLocation>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].uniforms.get(name).cloned())
    }

    /// Returns the uniform block location of a specified uniform block name.
    pub fn uniform_block_location(
        &self,
        key: &ProgramKey,
        name: &str,
    ) -> Result<Option<u32>, Error> {
        let index = *self.uuid_keys.get(key).ok_or(Error::ProgramNotFound)?;
        Ok(self.caches[index].uniform_blocks.get(name).cloned())
    }

    /// Creates a [`WebGlProgram`], and links compiled [`WebGlShader`] to the program.
    fn create_program(
        gl: &WebGl2RenderingContext,
        vs: &WebGlShader,
        fs: &WebGlShader,
    ) -> Result<WebGlProgram, Error> {
        let program = gl.create_program().ok_or(Error::CreateProgramFailure)?;

        // attaches shader to program
        gl.attach_shader(&program, vs);
        gl.attach_shader(&program, fs);
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
            return Err(Error::LinkProgramFailure(err));
        }

        Ok(program)
    }

    /// Collects active attribute name and location key-value pairs.
    fn collects_attributes(
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
            let Some(name) = gl
                .get_active_attrib(&program, location)
                .map(|info| info.name())
            else {
                continue;
            };

            locations.insert(name, location);
        }

        locations
    }

    /// Collects active uniform locations and bindings.
    fn collects_uniforms(
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
            let Some(name) = gl
                .get_active_uniform(&program, index)
                .map(|info| info.name())
            else {
                continue;
            };
            // getActiveUniform counts uniforms in Uniform Block as active uniforms as well.
            // getUniformLocation maybe None for those uniforms in Uniform Block.
            let Some(location) = gl.get_uniform_location(&program, &name) else {
                continue;
            };

            locations.insert(name, location);
        }

        locations
    }

    /// Collects active uniform block indices.
    fn collects_uniform_blocks(
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
}
