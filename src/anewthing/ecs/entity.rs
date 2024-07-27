use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(Uuid);

impl EntityId {
    pub(super) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
