use std::{borrow::Cow, cell::LazyCell, hash::Hash, ops::Range};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use line_span::LineSpanExt;
use log::warn;
use regex::Regex;
use web_sys::{WebGl2RenderingContext, WebGlShader};

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
pub enum ShaderName {
    Custom(Cow<'static, str>),
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
    name_position: (usize, usize),
    value_position: Option<(usize, usize)>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Define<'a> {
    name: Cow<'a, str>,
    value: Option<Cow<'a, str>>,
}

pub trait ShaderSource {
    /// Global unique name for this shader source.
    fn name(&self) -> ShaderName;

    /// Returns the source code of the shader.
    fn code(&self) -> &str;

    /// Returns a custom snippet code by name.
    fn snippet(&self, name: &str) -> Option<&str>;

    /// Returns a custom define value by name.
    fn define(&self, name: &str) -> Option<&str>;
}

struct ShaderCache {
    lines: Vec<String>,
    defines: Vec<DefinePosition>,
    variants: HashMap<Vec<Define<'static>>, WebGlShader>,
}

impl ShaderCache {
    fn get_or_compile_variant<S>(
        &mut self,
        gl: &WebGl2RenderingContext,
        shader_type: ShaderType,
        shader_source: &S,
    ) -> Result<&WebGlShader, Error>
    where
        S: ShaderSource,
    {
        let lines = &self.lines;
        let mut replaced_defines = Vec::new();
        let defines = self
            .defines
            .iter()
            .enumerate()
            .map(|(define_index, define_position)| {
                let DefinePosition {
                    line_index,
                    name_position,
                    value_position,
                } = define_position;
                let line = &lines[*line_index];
                let name = &line[name_position.0..name_position.1];
                let value = match shader_source.define(name) {
                    Some(value) => {
                        replaced_defines.push((define_index, define_position));
                        Some(value.trim())
                    }
                    None => match value_position {
                        Some(value) => Some(&line[value.0..value.1]),
                        None => None,
                    },
                };

                Define {
                    name: Cow::Borrowed(name),
                    value: value.map(|value| Cow::Borrowed(value)),
                }
            })
            .collect::<Vec<_>>();

        if let Some(variant) = self.variants.get(&defines) {
            unsafe {
                let variant: *const WebGlShader = variant;
                Ok(&*variant)
            }
        } else {
            let code = self.build_code(&defines, &replaced_defines);
            let shader = self.compile_shader(gl, shader_type, &code)?;
            // persists string slice to String
            let defines = defines
                .into_iter()
                .map(|define| Define {
                    name: Cow::Owned(define.name.to_string()),
                    value: define.value.map(|value| Cow::Owned(value.to_string())),
                })
                .collect::<Vec<_>>();

            Ok(self.variants.insert_unique_unchecked(defines, shader).1)
        }
    }

    fn build_code(
        &self,
        defines: &[Define],
        replaced_defines: &[(usize, &DefinePosition)],
    ) -> String {
        if replaced_defines.is_empty() {
            return self.lines.join("\n");
        }

        let mut lines = self
            .lines
            .iter()
            .map(|line| Cow::Borrowed(line.as_str()))
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

                let mut replaced_line = lines[*line_index].to_string();
                replaced_line.shrink_to(replaced_line.len() + value.len() + 1); // 1 for a space
                match value_position {
                    Some((start, end)) => replaced_line.replace_range(*start..*end, value),
                    None => {
                        replaced_line.push_str(" ");
                        replaced_line.push_str(value);
                    }
                };
                lines[*line_index] = Cow::Owned(replaced_line);
            });
        lines.join("\n")
    }

    fn compile_shader(
        &self,
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

struct ShaderSnippet {
    code: Cow<'static, str>,
    lines: Vec<Range<usize>>,
}

pub struct ShaderManager {
    gl: WebGl2RenderingContext,
    caches: HashMap<ShaderCacheKey, ShaderCache>,
    snippets: HashMap<Cow<'static, str>, ShaderSnippet>,
}

impl ShaderManager {
    /// Adds a new snippet code to manager. Returns the previous snippet code if occupied.
    pub fn add_snippet(
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
    pub fn remove_snippet(&mut self, name: &str) -> Option<Cow<'static, str>> {
        self.snippets.remove(name).map(|snippet| snippet.code)
    }

    /// Returns a compiled [`WebGlShader`] from a [`ShaderSource`] under specified [`ShaderType`].
    ///
    /// Manager identifies shader as different variants by values of define directives in the shader code.
    /// A cached [`WebGlShader`] is returned if it has been compiled before.
    pub fn get_or_compile_shader<S>(
        &mut self,
        shader_type: ShaderType,
        shader_source: &S,
    ) -> Result<&WebGlShader, Error>
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
        cache.get_or_compile_variant(&self.gl, shader_type, shader_source)
    }

    /// Creates a shader cache from a [`ShaderSource`].
    fn create_cache<S>(
        snippets: &HashMap<Cow<'static, str>, ShaderSnippet>,
        shader_source: &S,
    ) -> Result<ShaderCache, Error>
    where
        S: ShaderSource,
    {
        let mut lines = shader_source
            .code()
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        Self::prepare_pragmas(snippets, &mut lines, shader_source)?;
        let defines = Self::collect_defines(&lines);
        let cache = ShaderCache {
            lines,
            defines,
            variants: HashMap::new(),
        };

        Ok(cache)
    }

    /// Prepares pragmas.
    fn prepare_pragmas<S>(
        snippets: &HashMap<Cow<'static, str>, ShaderSnippet>,
        lines: &mut Vec<String>,
        shader_source: &S,
    ) -> Result<(), Error>
    where
        S: ShaderSource,
    {
        /// Regex for extracting pragma operation from `#pragma <operation> <value>` directive.
        const PRAGMA_REGEX: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(r"^\s*#pragma\s+(?P<operation>\w+)\s+(?P<value>.+)\s*$").unwrap()
        });

        let mut injecteds: HashSet<Cow<'_, str>> = HashSet::new();
        let mut i = 0;
        while i <= lines.len() {
            let line = &mut lines[i];

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
                            injecteds.insert_unique_unchecked(Cow::Owned(name.to_string()));
                            lines.splice(i..=i, snippet.lines().map(|line| line.to_string()));
                            // no need to accumulate line index
                        } else if let Some((name, snippet)) = snippets.get_key_value(name) {
                            injecteds.insert_unique_unchecked(Cow::Borrowed(name));
                            lines.splice(
                                i..=i,
                                snippet
                                    .lines
                                    .iter()
                                    .map(|line| snippet.code[line.to_owned()].to_string()),
                            );
                            // no need to accumulate line index
                        } else {
                            return Err(Error::SnippetNotFound(name.to_string()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Collects define directives from lines of shader code
    fn collect_defines(lines: &[String]) -> Vec<DefinePosition> {
        /// Regex for extracting defines from `#define <name> [<value>]` directive. value is optional.
        const DEFINE_REGEX: LazyCell<Regex> = LazyCell::new(|| {
            Regex::new(r"^\s*#define\s+(?P<name>\w+)\s*(?P<value>.*)\s*$").unwrap()
        });

        let mut defines = Vec::new();
        lines.into_iter().enumerate().for_each(|(index, line)| {
            let Some(captures) = DEFINE_REGEX.captures(line) else {
                return;
            };
            let Some(name) = captures
                .name("name")
                .map(|matched| (matched.start(), matched.end()))
            else {
                return;
            };
            let value = captures
                .name("name")
                .map(|matched| (matched.start(), matched.end()));

            defines.push(DefinePosition {
                line_index: index,
                name_position: name,
                value_position: value,
            });
        });

        defines
    }
}

#[test]
fn regex() {
    const REGEX: LazyCell<Regex> =
        LazyCell::new(|| Regex::new(r"^\s*#define\s+(?P<name>\w+)\s*(?P<value>.*)\s*$").unwrap());

    let captures = REGEX.captures("#define light 1").unwrap();
    assert_eq!("light", captures.name("name").unwrap().as_str());
    assert_eq!("1", captures.name("value").unwrap().as_str());
    let captures = REGEX.captures("     #define    light    0").unwrap();
    assert_eq!("light", captures.name("name").unwrap().as_str());
    assert_eq!("0", captures.name("value").unwrap().as_str());
    let captures = REGEX.captures("#define light     ").unwrap();
    assert_eq!("light", captures.name("name").unwrap().as_str());
    assert_eq!("", captures.name("value").unwrap().as_str());
    println!("{:?}", captures.name("value").unwrap());
}
