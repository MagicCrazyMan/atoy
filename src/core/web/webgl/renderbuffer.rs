/// Available render buffer targets mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderbufferTarget {
    Renderbuffer,
}

/// Available render buffer storages mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderbufferInternalFormat {
    RGBA32I,
    RGBA32UI,
    RGBA16I,
    RGBA16UI,
    RGBA8,
    RGBA8I,
    RGBA8UI,
    SRGB8_ALPHA8,
    RGB10_A2,
    RGB10_A2UI,
    RGBA4,
    RGB5_A1,
    RGB8,
    RGB565,
    RG32I,
    RG32UI,
    RG16I,
    RG16UI,
    RG8,
    RG8I,
    RG8UI,
    R32I,
    R32UI,
    R16I,
    R16UI,
    R8,
    R8I,
    R8UI,
    DEPTH_COMPONENT32F,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT16,
    DEPTH32F_STENCIL8,
    DEPTH24_STENCIL8,
    /// Available only when extension EXT_color_buffer_float is enabled
    R16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    RG16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    RGBA16F,
    /// Available only when extension EXT_color_buffer_float is enabled
    R32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    RG32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    RGBA32F,
    /// Available only when extension EXT_color_buffer_float is enabled
    R11F_G11F_B10F,
}
