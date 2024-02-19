use rand::distributions::{Distribution, Standard};

pub mod webgl;

// pub mod solid_color;
// pub mod texture;

/// Material transparency.
#[derive(Clone, Copy, PartialEq)]
pub enum Transparency {
    Opaque,
    Transparent,
    Translucent(f32),
}

impl Distribution<Transparency> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> Transparency {
        let alpha = rng.gen::<f32>();
        if alpha == 1.0 {
            Transparency::Opaque
        } else if alpha == 0.0 {
            Transparency::Transparent
        } else {
            Transparency::Translucent(alpha)
        }
    }
}

impl Transparency {
    /// Returns alpha value.
    pub fn alpha(&self) -> f32 {
        match self {
            Transparency::Opaque => 1.0,
            Transparency::Transparent => 0.0,
            Transparency::Translucent(alpha) => *alpha,
        }
    }
}
