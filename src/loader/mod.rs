use crate::channel::Receiver;

pub mod dds;
pub mod texture;

/// Loader status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoaderStatus {
    Unload,
    Loading,
    Loaded,
    Errored,
}

/// A simple loader for loading target resources.
///
/// A loader is designed under notifying pattern.
/// Calling [`Loader::load`] method just start the procedure of loading resources without waiting.
/// When resources loaded, developer should notify caller by calling [`Notifier::notify`] method.
///
/// A loader should not be resettable or reload-able and it should load a target resource only once.
/// However, during the lifetime of a loader, the [`Loader::load`] method may not be called only once,
/// thus, developer should avoid reloading resources when implementing a loader
pub trait Loader<Success> {
    /// Value when loader fails to load resources.
    type Failure;

    /// Returns current [`LoaderStatus`].
    fn status(&self) -> LoaderStatus;

    /// Starts loading procedure,
    /// [`Loader`] should turn to [`LoaderStatus::Loading`] immediately.
    fn load(&mut self);

    /// Returns loaded resources.
    ///
    /// This method should only be called when loader status is [`LoaderStatus::Loaded`] already.
    fn loaded(&self) -> Result<Success, Self::Failure>;

    fn success(&self) -> Receiver<LoaderStatus>;
}
