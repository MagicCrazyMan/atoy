/// Available blit framebuffer masks mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlitMask {
    ColorBufferBit,
    DepthBufferBit,
    StencilBufferBit,
}

/// Available blit framebuffer filters mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlitFilter {
    Nearest,
    Linear,
}
