use std::borrow::Cow;

use super::{
    attribute::AttributeBinding,
    uniform::{UniformBinding, UniformBlockBinding, UniformStructuralBinding},
};

/// Available shader types.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/// Available variable data types in GLSL version 300 es.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableDataType {
    Bool,
    Int,
    Uint,
    Float,
    BoolVec2,
    BoolVec3,
    BoolVec4,
    IntVec2,
    IntVec3,
    IntVec4,
    UintVec2,
    UintVec3,
    UintVec4,
    FloatVec2,
    FloatVec3,
    FloatVec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
    Sampler2DArray,
    Sampler2DArrayShadow,
    Sampler2DShadow,
    Sampler3D,
    SamplerCube,
    SamplerCubeShadow,
    Struct(&'static str),
}

impl VariableDataType {
    fn build(&self) -> &'static str {
        match self {
            VariableDataType::Bool => "bool",
            VariableDataType::Int => "int",
            VariableDataType::Uint => "uint",
            VariableDataType::Float => "float",
            VariableDataType::BoolVec2 => "bvec2",
            VariableDataType::BoolVec3 => "bvec3",
            VariableDataType::BoolVec4 => "bvec4",
            VariableDataType::IntVec2 => "ivec2",
            VariableDataType::IntVec3 => "ivec3",
            VariableDataType::IntVec4 => "ivec4",
            VariableDataType::UintVec2 => "uvec2",
            VariableDataType::UintVec3 => "uvec3",
            VariableDataType::UintVec4 => "uvec4",
            VariableDataType::FloatVec2 => "vec2",
            VariableDataType::FloatVec3 => "vec3",
            VariableDataType::FloatVec4 => "vec4",
            VariableDataType::Mat2 => "mat2",
            VariableDataType::Mat3 => "mat3",
            VariableDataType::Mat4 => "mat4",
            VariableDataType::Sampler2D => "sampler2D",
            VariableDataType::Sampler2DArray => "sampler2DArray",
            VariableDataType::Sampler2DArrayShadow => "sampler2DArrayShadow",
            VariableDataType::Sampler2DShadow => "sampler2DShadow",
            VariableDataType::Sampler3D => "sampler3D",
            VariableDataType::SamplerCube => "samplerCube",
            VariableDataType::SamplerCubeShadow => "samplerCubeShadow",
            VariableDataType::Struct(name) => name,
        }
    }
}

/// Input and output variable types in GLSL version 300 es.
#[derive(Clone)]
pub enum Variable {
    /// Input variable.
    In {
        name: String,
        data_type: VariableDataType,
        array_len: Option<usize>,
    },
    /// Output variable.
    Out {
        name: String,
        data_type: VariableDataType,
        array_len: Option<usize>,
    },
    /// Input uniform variable.
    Uniform {
        name: String,
        data_type: VariableDataType,
        array_len: Option<usize>,
    },
    /// Structure definition.
    Structure {
        name: String,
        uniforms: Vec<(String, VariableDataType, Option<usize>)>,
    },
    /// Input uniform block variable.
    UniformBlock {
        name: String,
        uniforms: Vec<(String, VariableDataType, Option<usize>)>,
    },
    /// Constructs from [`AttributeBinding`], always to be `in`.
    FromAttributeBinding(AttributeBinding),
    /// Constructs from [`UniformBinding`], always to be `uniform`.
    FromUniformBinding(UniformBinding),
    /// Constructs from [`UniformBlockBinding`], always to be `uniform`.
    FromUniformBlockBinding(UniformBlockBinding),
    /// Constructs from [`UniformStructuralBinding`], always to be `uniform`.
    FromUniformStructuralBinding(UniformStructuralBinding),
}

impl Variable {
    /// Constructs a new input variable.
    pub fn new_in<S>(name: S, data_type: VariableDataType) -> Self
    where
        S: Into<String>,
    {
        Self::In {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array input variable.
    pub fn new_in_array<S>(name: S, data_type: VariableDataType, array_len: usize) -> Self
    where
        S: Into<String>,
    {
        Self::In {
            name: name.into(),
            data_type,
            array_len: Some(array_len),
        }
    }

    /// Constructs a new output variable.
    pub fn new_out<S>(name: S, data_type: VariableDataType) -> Self
    where
        S: Into<String>,
    {
        Self::Out {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array output variable.
    pub fn new_out_array<S>(name: S, data_type: VariableDataType, array_len: usize) -> Self
    where
        S: Into<String>,
    {
        Self::Out {
            name: name.into(),
            data_type,
            array_len: Some(array_len),
        }
    }

    /// Constructs a new uniform variable.
    pub fn new_uniform<S>(name: S, data_type: VariableDataType) -> Self
    where
        S: Into<String>,
    {
        Self::Uniform {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array uniform variable.
    pub fn new_uniform_array<S>(name: S, data_type: VariableDataType, array_len: usize) -> Self
    where
        S: Into<String>,
    {
        Self::Uniform {
            name: name.into(),
            data_type,
            array_len: Some(array_len),
        }
    }

    /// Constructs a new uniform block variable.
    pub fn new_uniform_block<
        S: Into<String>,
        U: IntoIterator<Item = (String, VariableDataType, Option<usize>)>,
    >(
        name: S,
        uniforms: U,
    ) -> Self {
        Self::UniformBlock {
            name: name.into(),
            uniforms: uniforms.into_iter().collect(),
        }
    }

    /// Constructs a new `in` variable from NON-CUSTOM [`AttributeBinding`].
    pub fn from_attribute_binding(binding: AttributeBinding) -> Self {
        Self::FromAttributeBinding(binding)
    }

    /// Constructs a new `uniform` variable from NON-CUSTOM [`UniformBinding`].
    pub fn from_uniform_binding(binding: UniformBinding) -> Self {
        Self::FromUniformBinding(binding)
    }

    /// Constructs a new `uniform` variable from NON-CUSTOM [`UniformBlockBinding`].
    pub fn from_uniform_block_binding(binding: UniformBlockBinding) -> Self {
        Self::FromUniformBlockBinding(binding)
    }

    /// Constructs a new `uniform` variable from NON-CUSTOM [`UniformStructuralBinding`].
    pub fn from_uniform_structural_binding(binding: UniformStructuralBinding) -> Self {
        Self::FromUniformStructuralBinding(binding)
    }

    fn build(&self) -> String {
        match self {
            Variable::In {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("in {} {}[{}];", data_type.build(), name, len),
                None => format!("in {} {};", data_type.build(), name),
            },
            Variable::Out {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("out {} {}[{}];", data_type.build(), name, len),
                None => format!("out {} {};", data_type.build(), name),
            },
            Variable::Uniform {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("uniform {} {}[{}];", data_type.build(), name, len),
                None => format!("uniform {} {};", data_type.build(), name),
            },
            Variable::UniformBlock { name, uniforms } => {
                let mut block = String::new();
                block.push_str(&format!("layout(std) uniform {name} {{"));
                uniforms
                    .iter()
                    .for_each(|(name, data_type, array)| match array {
                        Some(len) => {
                            block.push_str(&format!("    {} {}[{}]", data_type.build(), name, len))
                        }
                        None => block.push_str(&format!("    {} {}", data_type.build(), name)),
                    });
                block.push_str("};");

                block
            }
            Variable::Structure { name, uniforms } => {
                let mut block = String::new();
                block.push_str(&format!("struct {name} {{"));
                uniforms
                    .iter()
                    .for_each(|(name, data_type, array)| match array {
                        Some(len) => {
                            block.push_str(&format!("    {} {}[{}]", data_type.build(), name, len))
                        }
                        None => block.push_str(&format!("    {} {}", data_type.build(), name)),
                    });
                block.push_str("};");

                block
            }
            Variable::FromAttributeBinding(binding) => format!(
                "in {} {};",
                binding
                    .data_type()
                    .expect("custom attribute binding is not supported")
                    .build(),
                binding.variable_name(),
            ),
            Variable::FromUniformBinding(binding) => format!(
                "uniform {} {};",
                binding
                    .data_type()
                    .expect("custom uniform binding is not supported")
                    .build(),
                binding.variable_name(),
            ),
            Variable::FromUniformBlockBinding(_) => String::new(), // uniform block binding does nothing
            Variable::FromUniformStructuralBinding(binding) => format!(
                "uniform {} {};",
                binding
                    .data_type()
                    .expect("custom uniform binding is not supported")
                    .build(),
                binding.variable_name(),
            ),
        }
    }
}

/// `#define HDR` GLSL arguments.
pub const DEFINED_HDR: Cow<'static, str> = Cow::Borrowed("HDR");

/// GLSL shaders builder.
///
/// Only GLSL version 300 es supported.
#[derive(Clone)]
pub struct ShaderBuilder {
    shader_type: ShaderType,
    include_header: bool,
    defines: Vec<Cow<'static, str>>,
    variables: Vec<Variable>,
    prepends: Vec<Cow<'static, str>>,
    appends: Vec<Cow<'static, str>>,
}

impl ShaderBuilder {
    /// Constructs a new shader builder.
    pub fn new<D, P, I, A>(
        shader_type: ShaderType,
        include_header: bool,
        defines: D,
        prepends: P,
        variables: I,
        appends: A,
    ) -> Self
    where
        D: IntoIterator<Item = Cow<'static, str>>,
        P: IntoIterator<Item = Cow<'static, str>>,
        I: IntoIterator<Item = Variable>,
        A: IntoIterator<Item = Cow<'static, str>>,
    {
        Self {
            shader_type,
            include_header,
            defines: defines.into_iter().collect(),
            prepends: prepends.into_iter().collect(),
            variables: variables.into_iter().collect(),
            appends: appends.into_iter().collect(),
        }
    }

    /// Builds complete shader source.
    pub fn build(&self) -> String {
        match self.shader_type {
            ShaderType::Vertex => self.build_vertex_shader(),
            ShaderType::Fragment => self.build_fragment_shader(),
        }
    }

    /// Returns shader type.
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    /// Returns defines source code.
    pub fn defines(&self) -> &[Cow<'static, str>] {
        &self.defines
    }

    /// Returns prepends source code.
    pub fn prepends(&self) -> &[Cow<'static, str>] {
        &self.prepends
    }

    /// Returns appends source code.
    pub fn appends(&self) -> &[Cow<'static, str>] {
        &self.appends
    }

    /// Returns variables.
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    fn build_vertex_shader(&self) -> String {
        let mut source = String::new();

        if self.include_header {
            source.push_str(VERTEX_SHADER_PREPEND);
        }
        self.defines.iter().for_each(|define| {
            source.push_str("#define ");
            source.push_str(define);
            source.push_str("\n");
        });
        self.prepends.iter().for_each(|prepend| {
            source.push_str(prepend);
            source.push_str("\n");
        });
        self.variables.iter().for_each(|variable| {
            source.push_str(&variable.build());
            source.push_str("\n");
        });
        self.appends.iter().for_each(|append| {
            source.push_str(append);
            source.push_str("\n");
        });

        source
    }

    fn build_fragment_shader(&self) -> String {
        let mut source = String::new();

        if self.include_header {
            source.push_str(FRAGMENT_SHADER_PREPEND);
        }
        self.defines.iter().for_each(|define| {
            source.push_str("#define ");
            source.push_str(define);
            source.push_str("\n");
        });
        self.prepends.iter().for_each(|prepend| {
            source.push_str(prepend);
            source.push_str("\n");
        });
        self.variables.iter().for_each(|variable| {
            source.push_str(&variable.build());
            source.push_str("\n");
        });
        self.appends.iter().for_each(|append| {
            source.push_str(append);
            source.push_str("\n");
        });

        source
    }
}

const VERTEX_SHADER_PREPEND: &'static str = "#version 300 es
";

const FRAGMENT_SHADER_PREPEND: &'static str = "#version 300 es
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
#else
precision mediump float;
precision mediump int;
#endif
";

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::render::webgl::{attribute::AttributeBinding, uniform::UniformBinding};

    use super::{ShaderBuilder, ShaderType, Variable};

    #[test]
    fn test_vertex_builder() {
        let builder = ShaderBuilder::new(
            ShaderType::Vertex,
            true,
            [],
            [],
            [
                Variable::from_attribute_binding(AttributeBinding::GeometryPosition),
                Variable::from_uniform_binding(UniformBinding::ModelMatrix),
                Variable::from_uniform_binding(UniformBinding::ViewProjMatrix),
            ],
            [Cow::Borrowed(
            "void main() {
                gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
            }",
            )],
        );

        println!("{}", builder.build());
    }
}
