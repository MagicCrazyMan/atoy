/// Material transparency.
#[derive(Clone, Copy, PartialEq)]
pub enum Transparency {
    Opaque,
    Transparent,
    Translucent(f64),
}

impl Transparency {
    /// Returns alpha value.
    pub fn alpha(&self) -> f64 {
        match self {
            Transparency::Opaque => 1.0,
            Transparency::Transparent => 0.0,
            Transparency::Translucent(alpha) => *alpha,
        }
    }
}

pub trait EntityTransparency {
    fn transparency(&self) -> Transparency;
}
