/// Lighting attenuation factors.
#[derive(Clone, Copy, PartialEq)]
pub struct Attenuation {
    a: f32,
    b: f32,
    c: f32,
}

impl Attenuation {
    /// Constructs a new lighting attenuation factors.
    pub fn new(a: f32, b: f32, c: f32) -> Self {
        Self { a, b, c }
    }

    /// Returns lighting attenuation factor a.
    pub fn a(&self) -> f32 {
        self.a
    }

    /// Returns lighting attenuation factor b.
    pub fn b(&self) -> f32 {
        self.b
    }

    /// Returns lighting attenuation factor c.
    pub fn c(&self) -> f32 {
        self.c
    }

    /// Sets lighting attenuation factor a.
    pub fn set_a(&mut self, a: f32) {
        self.a = a;
    }

    /// Sets lighting attenuation factor b.
    pub fn set_b(&mut self, b: f32) {
        self.b = b;
    }

    /// Sets lighting attenuation factor c.
    pub fn set_c(&mut self, c: f32) {
        self.c = c;
    }
}
