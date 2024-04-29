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
