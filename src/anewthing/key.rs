use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    CustomUuid(Uuid),
    CustomUsize(usize),
    CustomString(String),
    CustomStr(&'static str),
}
