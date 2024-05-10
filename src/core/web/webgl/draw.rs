use proc::GlEnum;

/// Available cull face types mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum DrawMode {
    #[gl_enum(POINTS)]
    Points,
    #[gl_enum(LINES)]
    Lines,
    #[gl_enum(LINE_LOOP)]
    LineLoop,
    #[gl_enum(LINE_STRIP)]
    LineStrip,
    #[gl_enum(TRIANGLES)]
    Triangles,
    #[gl_enum(TRIANGLE_STRIP)]
    TriangleStrip,
    #[gl_enum(TRIANGLE_FAN)]
    TriangleFan,
}
