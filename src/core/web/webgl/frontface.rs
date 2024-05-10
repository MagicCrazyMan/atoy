use proc::GlEnum;

/// Available front face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum FrontFace {
    #[gl_enum(CW)]
    Clockwise,
    #[gl_enum(CCW)]
    CounterClockwise,
}
