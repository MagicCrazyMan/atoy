use super::ItemKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NoSuchExecutor(ItemKey),
    SelfReferential,
    AlreadyConnected,
    InvalidateGraph,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
