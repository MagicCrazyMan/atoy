use std::borrow::Cow;

/// Available shader types.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/// GLSL shaders builder.
///
/// Only GLSL version 300 es supported.
#[derive(Clone)]
pub struct ShaderBuilder {
    shader_type: ShaderType,
    include_header: bool,
    defines: Vec<Cow<'static, str>>,
    prepends: Vec<Cow<'static, str>>,
    appends: Vec<Cow<'static, str>>,
}

impl ShaderBuilder {
    /// Constructs a new shader builder.
    pub fn new(
        shader_type: ShaderType,
        include_header: bool,
        defines: Vec<Cow<'static, str>>,
        prepends: Vec<Cow<'static, str>>,
        appends: Vec<Cow<'static, str>>,
    ) -> Self {
        Self {
            shader_type,
            include_header,
            defines,
            prepends,
            appends,
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
