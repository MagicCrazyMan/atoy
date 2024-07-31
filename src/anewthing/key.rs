use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Uuid(Uuid),
    Usize(usize),
    String(String),
    Str(&'static str),
}
