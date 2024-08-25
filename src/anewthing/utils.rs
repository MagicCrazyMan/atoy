/// Generates scoped unique id by increasing a [`usize`] id.
pub struct AccumulatingId(usize);

impl AccumulatingId {
    /// Constructs a new accumulating id generator.
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns current id and then increases it.
    pub fn next(&mut self) -> usize {
        let current = self.0;
        self.0 = self.0.wrapping_add(1);
        current
    }
}
