#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front,
    Back,
    FrontAndBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}
