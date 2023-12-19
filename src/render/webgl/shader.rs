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
    /// Input uniform block variable.
    UniformBlock {
        name: String,
        uniforms: Vec<(String, VariableDataType, Option<usize>)>,
    },
}

impl Variable {
    /// Constructs a new input variable.
    pub fn new_in<S: Into<String>>(name: S, data_type: VariableDataType) -> Self {
        Self::In {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array input variable.
    pub fn new_in_array<S: Into<String>>(
        name: S,
        data_type: VariableDataType,
        array_len: usize,
    ) -> Self {
        Self::In {
            name: name.into(),
            data_type,
            array_len: Some(array_len),
        }
    }

    /// Constructs a new output variable.
    pub fn new_out<S: Into<String>>(name: S, data_type: VariableDataType) -> Self {
        Self::Out {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array output variable.
    pub fn new_out_array<S: Into<String>>(
        name: S,
        data_type: VariableDataType,
        array_len: usize,
    ) -> Self {
        Self::Out {
            name: name.into(),
            data_type,
            array_len: Some(array_len),
        }
    }

    /// Constructs a new uniform variable.
    pub fn new_uniform<S: Into<String>>(name: S, data_type: VariableDataType) -> Self {
        Self::Uniform {
            name: name.into(),
            data_type,
            array_len: None,
        }
    }

    /// Constructs a new array uniform variable.
    pub fn new_uniform_array<S: Into<String>>(
        name: S,
        data_type: VariableDataType,
        array_len: usize,
    ) -> Self {
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

    fn build(&self) -> String {
        match self {
            Variable::In {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("in {} {}[{}]", name, data_type.build(), len),
                None => format!("in {} {}", name, data_type.build()),
            },
            Variable::Out {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("out {} {}[{}]", name, data_type.build(), len),
                None => format!("out {} {}", name, data_type.build()),
            },
            Variable::Uniform {
                name,
                data_type,
                array_len: array,
            } => match array {
                Some(len) => format!("uniform {} {}[{}]", name, data_type.build(), len),
                None => format!("uniform {} {}", name, data_type.build()),
            },
            Variable::UniformBlock { name, uniforms } => {
                let mut block = String::new();
                block.push_str(&format!("layout(std) uniform {name} {{"));
                uniforms
                    .iter()
                    .for_each(|(name, data_type, array)| match array {
                        Some(len) => {
                            block.push_str(&format!("    {} {}[{}]", name, data_type.build(), len))
                        }
                        None => block.push_str(&format!("    {} {}", name, data_type.build())),
                    });
                block.push_str("};");

                block
            }
        }
    }
}

/// GLSL shaders builder.
///
/// Only GLSL version 300 es supported.
#[derive(Clone)]
pub struct ShaderBuilder {
    shader_type: ShaderType,
    variables: Vec<Variable>,
    prepends: Vec<&'static str>,
    main_function: &'static str,
}

impl ShaderBuilder {
    /// Constructs a new shader builder.
    pub fn new<I: IntoIterator<Item = Variable>, P: IntoIterator<Item = &'static str>>(
        shader_type: ShaderType,
        variables: I,
        prepends: P,
        main_function: &'static str,
    ) -> Self {
        Self {
            shader_type,
            prepends: prepends.into_iter().collect(),
            variables: variables.into_iter().collect(),
            main_function,
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

    /// Returns prepends source code.
    pub fn prepends(&self) -> &[&'static str] {
        &self.prepends
    }

    /// Returns main function source code.
    pub fn main_function(&self) -> &'static str {
        &self.main_function
    }

    /// Returns variables.
    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    fn build_vertex_shader(&self) -> String {
        let mut source = String::new();

        source.push_str(VERTEX_SHADER_PREPEND);
        self.prepends
            .iter()
            .for_each(|prepend| source.push_str(prepend));
        self.variables
            .iter()
            .for_each(|variable| source.push_str(&variable.build()));
        source.push_str(&self.main_function);

        source
    }

    fn build_fragment_shader(&self) -> String {
        let mut source = String::new();

        source.push_str(FRAGMENT_SHADER_PREPEND);
        self.prepends
            .iter()
            .for_each(|prepend| source.push_str(prepend));
        self.variables
            .iter()
            .for_each(|variable| source.push_str(&variable.build()));
        source.push_str(&self.main_function);

        source
    }
}

const VERTEX_SHADER_PREPEND: &'static str = "#version 300 es
";

const FRAGMENT_SHADER_PREPEND: &'static str = "#version 300 es
#ifdef #GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
#else
precision mediump float;
precision mediump int;
#endif

";
