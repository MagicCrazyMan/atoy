/// Available render buffer storages mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderbufferInternalFormat {
    R8,
    R8UI,
    R8I,
    R16UI,
    R16I,
    R32UI,
    R32I,
    RG8,
    RG8UI,
    RG8I,
    RG16UI,
    RG16I,
    RG32UI,
    RG32I,
    RGB8,
    RGBA8,
    RGB10_A2,
    RGBA8UI,
    RGBA8I,
    RGB10_A2UI,
    RGBA16UI,
    RGBA16I,
    RGBA32UI,
    RGBA32I,
    R16F,
    RG16F,
    RGBA16F,
    R32F,
    RG32F,
    RGBA32F,
    R11F_G11F_B10F,
    SRGB,
    SRGB8,
    SRGB8_ALPHA8,
    RGBA4,
    RGB565,
    RGB5_A1,
    DEPTH_COMPONENT16,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT32F,
    STENCIL_INDEX8,
    DEPTH_STENCIL,
    DEPTH24_STENCIL8,
    DEPTH32F_STENCIL8,
}
