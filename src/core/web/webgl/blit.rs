use proc::GlEnum;

/// Available blit framebuffer masks mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BlitMask {
    ColorBufferBit,
    DepthBufferBit,
    StencilBufferBit,
}

/// Available blit framebuffer filters mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BlitFilter {
    Nearest,
    Linear,
}
