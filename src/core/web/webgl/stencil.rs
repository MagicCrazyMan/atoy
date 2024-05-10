use proc::GlEnum;

/// Available stencil functions mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum StencilFunction {
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

/// Available stencil operators mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    #[gl_enum(INCR)]
    Increase,
    #[gl_enum(INCR_WRAP)]
    IncreaseWrap,
    #[gl_enum(DECR)]
    Decrease,
    #[gl_enum(DECR_WRAP)]
    DecreaseWrap,
    Invert,
}
