use std::{
    borrow::Cow,
    cell::LazyCell,
    convert::TryFrom,
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::warn;
use regex::Regex;
use web_sys::WebGl2RenderingContext;

/// Available shader types for WebGL 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderType {
    /// Vertex Shader.
    Vertex,
    /// Fragment Shader.
    Fragment,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderName {
    Custom(Cow<'static, str>),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum DirectiveKind {
    Define,
    Pragma,
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
    name: Cow<'static, str>,
    line_index: usize,
    char_index: usize,
}

#[derive(PartialEq, Eq, Hash)]
struct Define<'a> {
    name: Cow<'a, str>,
    value: Cow<'a, str>,
}

struct ShaderCache {
    gl: WebGl2RenderingContext,
    lines: Vec<String>,
    define_positions: Vec<DefinePosition>,
    variants: HashMap<Vec<Define<'static>>, Shader>,
}

impl ShaderCache {
    // fn new<S: ShaderSource>(gl: WebGl2RenderingContext, shader_source: S) -> Self {
    //     Self {
    //         gl,
    //         lines: shader_source.code().lines().collect(),
    //         define_positions: todo!(),
    //         variants: HashMap::new(),
    //     }
    // }

    // fn defines(&self) -> Vec<Define> {
    //     let mut key = Vec::with_capacity(self.define_positions.len());
    //     self.define_positions.iter().for_each(|position| {
    //         let DefinePosition {
    //             name,
    //             line_index,
    //             char_index,
    //         } = position;
    //         key.push(Define {
    //             name: name.clone(),
    //             value: Cow::Borrowed(&self.lines[*line_index][*char_index..]),
    //         });
    //     });
    //     key
    // }

    // fn defines_with_custom_defines(
    //     &self,
    //     defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    // ) -> Vec<Define> {
    //     let mut key = Vec::with_capacity(self.define_positions.len());
    //     self.define_positions.iter().for_each(|position| {
    //         let DefinePosition {
    //             name,
    //             line_index,
    //             char_index,
    //         } = position;

    //         match defines.get(name) {
    //             Some(define) => key.push(Define {
    //                 name: name.clone(),
    //                 value: define.clone(),
    //             }),
    //             None => key.push(Define {
    //                 name: name.clone(),
    //                 value: Cow::Borrowed(&self.lines[*line_index][*char_index..]),
    //             }),
    //         };
    //     });
    //     key
    // }

    // fn build_source_code(&self) -> String {
    //     self.lines.join("\n")
    // }

    // fn build_source_code_with_custom_defines(
    //     &self,
    //     defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    // ) -> String {
    //     self.lines.join("\n")
    // }

    // fn get_or_compile_shader(&self) -> &Shader {
    //     let key = self.defines();
    //     match self.variants.get(&key) {
    //         Some(shader) => shader,
    //         None => {
    //             let code = self.build_source_code();
    //             todo!()
    //         }
    //     }
    // }

    // fn get_or_compile_shader_with_custom_defines(
    //     &self,
    //     defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    // ) -> &Shader {
    //     let key = self.defines_with_custom_defines(defines);
    //     match self.variants.get(&key) {
    //         Some(shader) => shader,
    //         None => {
    //             let code = self.build_source_code_with_custom_defines(defines);
    //             todo!()
    //         }
    //     }
    // }

    // fn to_source_code_with_defines(
    //     &self,
    //     defines: HashMap<Cow<'static, str>, Cow<'static, str>>,
    // ) -> String {
    //     let mut lines = self.lines.clone();
    //     for (name, value) in defines {
    //         let Some((line_index, char_index)) = self.defines.get(&name).copied() else {
    //             continue;
    //         };
    //         let mut new_line = lines[line_index].to_string();
    //         new_line.replace_range(char_index.., &value);
    //         lines[line_index] = Cow::Owned(new_line);
    //     }

    //     todo!()
    // }
}

pub trait ShaderSource {
    /// Global unique name for this shader source.
    fn name(&self) -> &str;

    /// Returns the source code of the shader.
    fn code(&self) -> &str;

    /// Returns a custom snippet code by name.
    fn snippet(&self, name: &str) -> Option<&str>;

    /// Returns a custom define value by name.
    fn define(&self, name: &str) -> Option<&str>;
}

pub struct Shader {}

pub struct ShaderManager {
    caches: HashMap<ShaderCacheKey, ShaderCache>,
    snippets: HashMap<Cow<'static, str>, Vec<String>>,
}

impl ShaderManager {
    pub fn get_or_compile_shader<S>(&mut self, name: ShaderName, shader_source: &S) -> &Shader
    where
        S: ShaderSource,
    {
        todo!()
    }

    fn compile_shader<S>(&mut self, name: ShaderName, shader_source: &S) -> &Shader
    where
        S: ShaderSource,
    {
        let mut lines = shader_source
            .code()
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        self.prepare_pragmas(&mut lines, shader_source);

        todo!()
    }

    fn find_directives(&self, lines: &[String]) -> HashMap<DirectiveKind, Vec<usize>> {
        let mut directives = HashMap::new();
        lines.into_iter().enumerate().for_each(|(index, line)| {
            let line = line.trim_start();

            let directive;
            if line.starts_with("#define") {
                directive = DirectiveKind::Define;
            } else if line.starts_with("#pragma") {
                directive = DirectiveKind::Pragma;
            } else {
                return;
            };
            directives
                .entry(directive)
                .or_insert_with(|| Vec::new())
                .push(index);
        });
        directives
    }

    /// Injects snippet code.
    fn prepare_pragmas<S>(&self, lines: &mut Vec<String>, shader_source: &S)
    where
        S: ShaderSource,
    {
        /// Regex for extracting pragma operation from `#pragma <operation> <value>` directive.
        const PRAGMA_REGEX: LazyCell<Regex> =
            LazyCell::new(|| Regex::new(r"#pragma\s+(?P<operation>\w+)\s+(?P<value>.+)").unwrap());

        let mut injected: HashSet<Cow<'_, str>> = HashSet::new();
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

                    if injected.contains(name) {
                        warn!(target: "ShaderManager", "snippet '{}' inject more than once", name);
                        lines.remove(i);
                        // no need to accumulate line index
                    } else {
                        if let Some(snippet) = shader_source.snippet(name) {
                            injected.insert_unique_unchecked(Cow::Owned(name.to_string()));
                            lines.splice(i..=i, snippet.lines().map(|line| line.to_string()));
                            // no need to accumulate line index
                        } else if let Some((name, snippet)) = self.snippets.get_key_value(name) {
                            injected.insert_unique_unchecked(Cow::Borrowed(name));
                            lines.splice(i..=i, snippet.into_iter().map(|line| line.clone()));
                            // no need to accumulate line index
                        } else {
                            warn!(target: "ShaderManager", "snippet '{}' not found", name);
                            i += 1;
                            continue;
                        }
                    }
                }
            }
        }
    }
}
