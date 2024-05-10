use proc::GlEnum;

/// Available cull face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum DepthFunction {
    Never,
    Less,
    Equal,
    #[gl_enum(LEQUAL)]
    LessEqual,
    Greater,
    #[gl_enum(NOTEQUAL)]
    NotEqual,
    #[gl_enum(GEQUAL)]
    GreaterEqual,
    Always,
}
