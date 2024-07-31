use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityKey(Uuid);

impl EntityKey {
    /// Constructs a new entity key.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
