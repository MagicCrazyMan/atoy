use std::{borrow::Cow, cell::LazyCell, hash::Hash, ops::Range, rc::Rc};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use line_span::LineSpanExt;
use log::warn;
use proc::GlEnum;
use regex::Regex;
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::{anewthing::key::Key, renderer::webgl::conversion::ToGlEnum};

use super::error::Error;

/// Available shader types for WebGL 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, GlEnum)]
pub enum WebGlShaderType {
    /// Vertex Shader.
    #[gl_enum(VERTEX_SHADER)]
    Vertex,
    /// Fragment Shader.
    #[gl_enum(FRAGMENT_SHADER)]
    Fragment,
}

impl ToGlEnum for WebGlShaderType {
    fn gl_enum(&self) -> u32 {
        match self {
            WebGlShaderType::Vertex => WebGl2RenderingContext::VERTEX_SHADER,
            WebGlShaderType::Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum WebGlShaderKey {
    Custom(Key),
}

pub trait WebGlShaderSource {
    /// Global unique key for this shader source.
    fn key(&self) -> WebGlShaderKey;

    /// Returns the source code of the shader.
    fn code(&self) -> &str;

    /// Returns a custom snippet code by name.
    fn snippet(&self, name: &str) -> Option<&str>;

    /// Returns a custom define value by name.
    fn define_value(&self, name: &str) -> Option<&str>;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum WebGlPragmaOperation {
    InjectSnippet,
}

impl WebGlPragmaOperation {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "inject" => Some(WebGlPragmaOperation::InjectSnippet),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct WebGlShaderCacheKey {
    shader_type: WebGlShaderType,
    name: WebGlShaderKey,
}

struct GLSLDefinePosition {
    line_index: usize,
    /// position in line, not in the whole code string
    name_position: Range<usize>,
    value_position: Option<Range<usize>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct GLSLDefine<'a> {
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
}

#[derive(Clone)]
struct WebGlShaderItem {
    id: Uuid,
    shader: WebGlShader,
}

struct WebGlShaderCacheItem {
    code: String,
    line_ranges: Vec<Range<usize>>,
    defines: Vec<GLSLDefinePosition>,
    variants: HashMap<Vec<GLSLDefine<'static>>, WebGlShaderItem>,
}

struct GLSLShaderSnippet {
    code: Cow<'static, str>,
    lines: Vec<Range<usize>>,
}

struct WebGlShaderManager {
    gl: WebGl2RenderingContext,
    caches: HashMap<WebGlShaderCacheKey, WebGlShaderCacheItem>,
    snippets: HashMap<Cow<'static, str>, GLSLShaderSnippet>,
}

impl WebGlShaderManager {
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
            .insert(name, GLSLShaderSnippet { code, lines })
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
        shader_type: WebGlShaderType,
        shader_source: &S,
    ) -> Result<WebGlShaderItem, Error>
    where
        S: WebGlShaderSource,
    {
        let key = WebGlShaderCacheKey {
            shader_type,
            name: shader_source.key(),
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
        snippets: &HashMap<Cow<'static, str>, GLSLShaderSnippet>,
        shader_source: &S,
    ) -> Result<WebGlShaderCacheItem, Error>
    where
        S: WebGlShaderSource,
    {
        let mut lines = shader_source.code().lines().collect::<Vec<_>>();
        Self::prepare_pragmas(snippets, &mut lines, shader_source)?;

        let code = lines.join("\n");
        let line_ranges = code
            .line_spans()
            .map(|line| line.range())
            .collect::<Vec<_>>();
        let defines = Self::collect_defines(&code, &line_ranges);

        let cache = WebGlShaderCacheItem {
            code,
            line_ranges,
            defines,
            variants: HashMap::new(),
        };

        Ok(cache)
    }

    /// Prepares pragmas.
    fn prepare_pragmas<'a, 'b: 'a, S>(
        snippets: &'a HashMap<Cow<'static, str>, GLSLShaderSnippet>,
        lines: &mut Vec<&'a str>,
        shader_source: &'b S,
    ) -> Result<(), Error>
    where
        S: WebGlShaderSource,
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
                .and_then(|matched| WebGlPragmaOperation::from_str(matched.as_str()))
            else {
                i += 1;
                continue;
            };

            match operation {
                WebGlPragmaOperation::InjectSnippet => {
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
    fn collect_defines(code: &str, lines: &[Range<usize>]) -> Vec<GLSLDefinePosition> {
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

                defines.push(GLSLDefinePosition {
                    line_index,
                    name_position,
                    value_position,
                });
            });

        defines
    }

    fn get_or_compile_variant_shader<S>(
        cache: &mut WebGlShaderCacheItem,
        gl: &WebGl2RenderingContext,
        shader_type: WebGlShaderType,
        shader_source: &S,
    ) -> Result<WebGlShaderItem, Error>
    where
        S: WebGlShaderSource,
    {
        let line_ranges = &cache.line_ranges;
        let mut replaced_defines = Vec::new();
        let defines = cache
            .defines
            .iter()
            .enumerate()
            .map(|(define_index, define_position)| {
                let GLSLDefinePosition {
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

                GLSLDefine {
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
                .map(|define| GLSLDefine {
                    name: Cow::Owned(define.name.to_string()),
                    value: define.value.map(|value| Cow::Owned(value.to_string())),
                })
                .collect::<Vec<_>>();

            Ok(cache
                .variants
                .insert_unique_unchecked(
                    defines,
                    WebGlShaderItem {
                        id: Uuid::new_v4(),
                        shader,
                    },
                )
                .1
                .clone())
        }
    }

    fn create_variant_code<'a>(
        cache: &'a WebGlShaderCacheItem,
        defines: &[GLSLDefine],
        replaced_defines: &[(usize, &GLSLDefinePosition)],
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
                let GLSLDefinePosition {
                    line_index,
                    value_position,
                    ..
                } = define_position;
                let GLSLDefine { value, .. } = &defines[*define_index];
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
        shader_type: WebGlShaderType,
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
struct WebGlProgramKey {
    vertex_shader_id: Uuid,
    fragment_shader_id: Uuid,
}

#[derive(Clone)]
pub struct WebGlProgramItem {
    gl_program: WebGlProgram,
    attributes: Rc<HashMap<String, u32>>,
    uniforms: Rc<HashMap<String, WebGlUniformLocation>>,
    uniform_blocks: Rc<HashMap<String, u32>>,
}

impl WebGlProgramItem {
    /// Returns [`WebGlProgram`].
    pub fn gl_program(&self) -> &WebGlProgram {
        &self.gl_program
    }

    /// Returns all attributes name and location key-value pairs.
    pub fn attributes_locations(&self) -> &HashMap<String, u32> {
        &self.attributes
    }

    /// Returns all uniforms name and [`WebGlUniformLocation`] key-value pairs.
    pub fn uniforms_locations(&self) -> &HashMap<String, WebGlUniformLocation> {
        &self.uniforms
    }

    /// Returns all uniform blocks name and location key-value pairs.
    pub fn uniform_blocks_locations(&self) -> &HashMap<String, u32> {
        &self.uniform_blocks
    }

    /// Returns the attribute location of a specified attribute name.
    pub fn attribute_location(&self, name: &str) -> Option<u32> {
        self.attributes.get(name).copied()
    }

    /// Returns the uniform location of a specified uniform name.
    pub fn uniform_location(&self, name: &str) -> Option<&WebGlUniformLocation> {
        self.uniforms.get(name)
    }

    /// Returns the uniform block location of a specified uniform block name.
    pub fn uniform_block_location(&self, name: &str) -> Option<u32> {
        self.uniform_blocks.get(name).copied()
    }
}

/// Program manager.
///
/// Once a program compiled, it is cached and not deletable.
pub struct WebGlProgramManager {
    gl: WebGl2RenderingContext,
    shader_manager: WebGlShaderManager,
    programs: HashMap<WebGlProgramKey, WebGlProgramItem>,
}

impl WebGlProgramManager {
    /// Constructs a new program manager.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            shader_manager: WebGlShaderManager::new(gl.clone()),
            programs: HashMap::new(),
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

    /// Returns a compiled [`WebGlProgramItem`] from a vertex shader and a fragment shader.
    ///
    /// A cached program is returned if vertex shader and fragment shader are cached.
    pub fn get_or_compile_program<VS, FS>(
        &mut self,
        vertex: &VS,
        fragment: &FS,
    ) -> Result<WebGlProgramItem, Error>
    where
        VS: WebGlShaderSource,
        FS: WebGlShaderSource,
    {
        let WebGlShaderItem {
            id: vertex_shader_id,
            shader: vs,
        } = self
            .shader_manager
            .get_or_compile_shader(WebGlShaderType::Vertex, vertex)?;
        let WebGlShaderItem {
            id: fragment_shader_id,
            shader: fs,
        } = self
            .shader_manager
            .get_or_compile_shader(WebGlShaderType::Fragment, fragment)?;

        let cache_key = WebGlProgramKey {
            vertex_shader_id,
            fragment_shader_id,
        };
        let program = match self.programs.entry(cache_key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let gl_program = Self::create_program(&self.gl, &vs, &fs)?;
                let attributes = Self::collects_attributes(&self.gl, &gl_program);
                let uniforms = Self::collects_uniforms(&self.gl, &gl_program);
                let uniform_blocks = Self::collects_uniform_blocks(&self.gl, &gl_program);
                let program = WebGlProgramItem {
                    gl_program,
                    attributes: Rc::new(attributes),
                    uniforms: Rc::new(uniforms),
                    uniform_blocks: Rc::new(uniform_blocks),
                };
                entry.insert(program)
            }
        };
        Ok(program.clone())
    }

    // /// Unbinds current using program from [`WebGl2RenderingContext`].
    // pub fn unuse_program(&mut self) {
    //     if let Some(_) = self.program_in_use.take() {
    //         self.gl.use_program(None);
    //     }
    // }

    // /// Sets using program of [`WebGl2RenderingContext`] to [`None`] forcedly.
    // pub fn unuse_program_force(&mut self) {
    //     self.gl.use_program(None);
    //     self.program_in_use = None;
    // }

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
