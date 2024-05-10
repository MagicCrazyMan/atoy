use proc::GlEnum;

/// Available render buffer targets mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum RenderbufferTarget {
    #[gl_enum(RENDERBUFFER)]
    Renderbuffer,
}

/// Available render buffer storages mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum RenderbufferInternalFormat {
    #[gl_enum(RGBA32I)]
    RGBA32I,
    #[gl_enum(RGBA32UI)]
    RGBA32UI,
    #[gl_enum(RGBA16I)]
    RGBA16I,
    #[gl_enum(RGBA16UI)]
    RGBA16UI,
    #[gl_enum(RGBA8)]
    RGBA8,
    #[gl_enum(RGBA8I)]
    RGBA8I,
    #[gl_enum(RGBA8UI)]
    RGBA8UI,
    #[gl_enum(SRGB8_ALPHA8)]
    SRGB8_ALPHA8,
    #[gl_enum(RGB10_A2)]
    RGB10_A2,
    #[gl_enum(RGB10_A2UI)]
    RGB10_A2UI,
    #[gl_enum(RGBA4)]
    RGBA4,
    #[gl_enum(RGB5_A1)]
    RGB5_A1,
    #[gl_enum(RGB8)]
    RGB8,
    #[gl_enum(RGB565)]
    RGB565,
    #[gl_enum(RG32I)]
    RG32I,
    #[gl_enum(RG32UI)]
    RG32UI,
    #[gl_enum(RG16I)]
    RG16I,
    #[gl_enum(RG16UI)]
    RG16UI,
    #[gl_enum(RG8)]
    RG8,
    #[gl_enum(RG8I)]
    RG8I,
    #[gl_enum(RG8UI)]
    RG8UI,
    #[gl_enum(R32I)]
    R32I,
    #[gl_enum(R32UI)]
    R32UI,
    #[gl_enum(R16I)]
    R16I,
    #[gl_enum(R16UI)]
    R16UI,
    #[gl_enum(R8)]
    R8,
    #[gl_enum(R8I)]
    R8I,
    #[gl_enum(R8UI)]
    R8UI,
    #[gl_enum(DEPTH_COMPONENT32F)]
    DEPTH_COMPONENT32F,
    #[gl_enum(DEPTH_COMPONENT24)]
    DEPTH_COMPONENT24,
    #[gl_enum(DEPTH_COMPONENT16)]
    DEPTH_COMPONENT16,
    #[gl_enum(DEPTH32F_STENCIL8)]
    DEPTH32F_STENCIL8,
    #[gl_enum(DEPTH24_STENCIL8)]
    DEPTH24_STENCIL8,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(R16F)]
    R16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(RG16F)]
    RG16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(RGBA16F)]
    RGBA16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(R32F)]
    R32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(RG32F)]
    RG32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(RGBA32F)]
    RGBA32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    #[gl_enum(R11F_G11F_B10F)]
    R11F_G11F_B10F,
}
