use proc::GlEnum;

/// Available cull face mode mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlCullFace {
    Front,
    Back,
    FrontAndBack,
}

/// Available depth compare function mode mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlDepthCompareFunction {
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

/// Available stencil compare functions mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlStencilCompareFunction {
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
pub enum WebGlStencilOperator {
    Keep,
    Zero,
    Replace,
    #[gl_enum(INCR)]
    Increment,
    #[gl_enum(INCR_WRAP)]
    IncrementWrap,
    #[gl_enum(DECR)]
    Decrement,
    #[gl_enum(DECR_WRAP)]
    DecrementWrap,
    Invert,
}

/// Available blend equations mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlBlendEquation {
    #[gl_enum(FUNC_ADD)]
    Add,
    #[gl_enum(FUNC_SUBTRACT)]
    Subtract,
    #[gl_enum(FUNC_REVERSE_SUBTRACT)]
    ReverseSubtract,
    Min,
    Max,
}

/// Available blend functions mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlBlendFunction {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    SrcAlphaSaturate,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

/// Available blend function with values mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebGlBlendFunctionWithValue {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    SrcAlphaSaturate,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor(f64, f64, f64, f64),
    OneMinusConstantColor(f64, f64, f64, f64),
    ConstantAlpha(f64),
    OneMinusConstantAlpha(f64),
}

/// Available data type of element indices mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlElementIndicesDataType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

/// Available draw modes mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlDrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}
