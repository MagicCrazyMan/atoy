use proc::GlEnum;

/// Available cull face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum CullFace {
    Front,
    Back,
    FrontAndBack,
}
