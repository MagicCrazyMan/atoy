use proc::GlEnum;

/// Available blit framebuffer masks mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BlitMask {
    #[gl_enum(COLOR_BUFFER_BIT)]
    ColorBufferBit,
    #[gl_enum(DEPTH_BUFFER_BIT)]
    DepthBufferBit,
    #[gl_enum(STENCIL_BUFFER_BIT)]
    StencilBufferBit,
}

/// Available blit framebuffer filters mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BlitFilter {
    #[gl_enum(NEAREST)]
    Nearest,
    #[gl_enum(LINEAR)]
    Linear,
}
