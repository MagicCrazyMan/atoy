/// Available stencil functions mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Available stencil operators mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Increase,
    IncreaseWrap,
    Decrease,
    DecreaseWrap,
    Invert,
}
