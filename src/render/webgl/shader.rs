use std::borrow::Cow;

/// GLSL shaders builder.
///
/// Only GLSL version 300 es supported.
#[derive(Clone)]
pub struct ShaderBuilder {
    include_header: bool,
    defines: Vec<Cow<'static, str>>,
    prepends: Vec<Cow<'static, str>>,
    appends: Vec<Cow<'static, str>>,
}

impl ShaderBuilder {
    /// Constructs a new shader builder.
    pub fn new(
        include_header: bool,
        defines: Vec<Cow<'static, str>>,
        prepends: Vec<Cow<'static, str>>,
        appends: Vec<Cow<'static, str>>,
    ) -> Self {
        Self {
            include_header,
            defines,
            prepends,
            appends,
        }
    }

    pub fn include_header(&self) -> bool {
        self.include_header
    }

    pub fn set_include_header(&mut self, include_header: bool) {
        self.include_header = include_header;
    }

    /// Returns defines source code.
    pub fn defines(&self) -> &[Cow<'static, str>] {
        &self.defines
    }

    pub fn defines_mut(&mut self) -> &mut Vec<Cow<'static, str>> {
        &mut self.defines
    }

    /// Returns prepends source code.
    pub fn prepends(&self) -> &[Cow<'static, str>] {
        &self.prepends
    }

    pub fn prepends_mut(&mut self) -> &mut Vec<Cow<'static, str>> {
        &mut self.prepends
    }

    /// Returns appends source code.
    pub fn appends(&self) -> &[Cow<'static, str>] {
        &self.appends
    }

    pub fn appends_mut(&mut self) -> &mut Vec<Cow<'static, str>> {
        &mut self.appends
    }

    /// Builds to vertex shader source.
    pub fn build_vertex_shader(&self) -> String {
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

    /// Builds to fragment shader source.
    pub fn build_fragment_shader(&self) -> String {
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
precision highp sampler2D;
precision highp int;
#else
precision mediump float;
precision mediump sampler2D;
precision mediump int;
#endif
";
