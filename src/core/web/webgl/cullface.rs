use proc::GlEnum;

/// Available cull face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum CullFace {
    #[gl_enum(FRONT)]
    Front,
    #[gl_enum(BACK)]
    Back,
    #[gl_enum(FRONT_AND_BACK)]
    FrontAndBack,
}
