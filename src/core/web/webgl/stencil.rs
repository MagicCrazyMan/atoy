use proc::GlEnum;

/// Available stencil functions mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum StencilFunction {
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

/// Available stencil operators mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum StencilOp {
    #[gl_enum(KEEP)]
    Keep,
    #[gl_enum(ZERO)]
    Zero,
    #[gl_enum(REPLACE)]
    Replace,
    #[gl_enum(INCR)]
    Increase,
    #[gl_enum(INCR_WRAP)]
    IncreaseWrap,
    #[gl_enum(DECR)]
    Decrease,
    #[gl_enum(DECR_WRAP)]
    DecreaseWrap,
    #[gl_enum(INVERT)]
    Invert,
}
