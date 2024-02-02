use crate::notify::Notifier;

pub mod dds;
pub mod texture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoaderStatus {
    Unload,
    Loading,
    Loaded,
    Errored,
}

pub trait Loader<T> {
    type Error;

    fn status(&self) -> LoaderStatus;

    fn load(&mut self);

    fn loaded(&self) -> Result<T, Self::Error>;

    fn notifier(&self) -> &Notifier<LoaderStatus>;
}
