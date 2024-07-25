use super::archetype::Archetype;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub index: usize,
    pub version: usize,
}

impl Entity {
    pub(super) fn new(index: usize, version: usize) -> Self {
        Self { index, version }
    }
}
