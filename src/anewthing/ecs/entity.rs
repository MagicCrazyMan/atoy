use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(Uuid);

impl Entity {
    pub(super) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
