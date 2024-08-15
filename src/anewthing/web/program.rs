use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

use hashbrown::HashMap;
use web_sys::WebGl2RenderingContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

struct ShaderTemplateKey {
    shader_type: ShaderType,
    name: Cow<'static, str>,
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
    lines: Vec<Cow<'static, str>>,
    define_positions: Vec<DefinePosition>,
    variants: HashMap<Vec<Define<'static>>, Shader>,
}

impl ShaderCache {
    fn new<S: ShaderSource>(gl: WebGl2RenderingContext, shader_source: S) -> Self {
        Self {
            gl,
            lines: shader_source.source_code().lines().collect(),
            define_positions: todo!(),
            variants: HashMap::new(),
        }
    }

    fn defines(&self) -> Vec<Define> {
        let mut key = Vec::with_capacity(self.define_positions.len());
        self.define_positions.iter().for_each(|position| {
            let DefinePosition {
                name,
                line_index,
                char_index,
            } = position;
            key.push(Define {
                name: name.clone(),
                value: Cow::Borrowed(&self.lines[*line_index][*char_index..]),
            });
        });
        key
    }

    fn defines_with_custom_defines(
        &self,
        defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Vec<Define> {
        let mut key = Vec::with_capacity(self.define_positions.len());
        self.define_positions.iter().for_each(|position| {
            let DefinePosition {
                name,
                line_index,
                char_index,
            } = position;

            match defines.get(name) {
                Some(define) => key.push(Define {
                    name: name.clone(),
                    value: define.clone(),
                }),
                None => key.push(Define {
                    name: name.clone(),
                    value: Cow::Borrowed(&self.lines[*line_index][*char_index..]),
                }),
            };
        });
        key
    }

    fn build_source_code(&self) -> String {
        self.lines.join("\n")
    }

    fn build_source_code_with_custom_defines(
        &self,
        defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> String {
        self.lines.join("\n")
    }

    fn get_or_compile_shader(&self) -> &Shader {
        let key = self.defines();
        match self.variants.get(&key) {
            Some(shader) => shader,
            None => {
                let code = self.build_source_code();
                todo!()
            }
        }
    }

    fn get_or_compile_shader_with_custom_defines(
        &self,
        defines: &HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> &Shader {
        let key = self.defines_with_custom_defines(defines);
        match self.variants.get(&key) {
            Some(shader) => shader,
            None => {
                let code = self.build_source_code_with_custom_defines(defines);
                todo!()
            }
        }
    }

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
    fn name(&self) -> Cow<'static, str>;

    /// Returns the source code of the shader.
    fn source_code(&self) -> Cow<'static, str>;

    /// Returns a custom snippet code by name.
    fn snippet(&self, name: &str) -> Option<Cow<'static, str>>;

    /// Returns a custom define value by name.
    fn define(&self, name: &str) -> Option<Cow<'static, str>>;
}

pub struct Shader {}

pub struct ShaderManager {
    templates: HashMap<ShaderTemplateKey, ShaderCache>,
}

impl ShaderManager {
    pub fn get_or_compile_shader<S: ShaderSource>(&mut self, shader_source: &S) -> &Shader {}
}
