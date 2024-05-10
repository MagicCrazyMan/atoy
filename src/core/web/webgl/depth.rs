use proc::GlEnum;

/// Available cull face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum DepthFunction {
    #[gl_enum(NEVER)]
    Never,
    #[gl_enum(LESS)]
    Less,
    #[gl_enum(EQUAL)]
    Equal,
    #[gl_enum(LEQUAL)]
    LessEqual,
    #[gl_enum(GREATER)]
    Greater,
    #[gl_enum(NOTEQUAL)]
    NotEqual,
    #[gl_enum(GEQUAL)]
    GreaterEqual,
    #[gl_enum(ALWAYS)]
    Always,
}
