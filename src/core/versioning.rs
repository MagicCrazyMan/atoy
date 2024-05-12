pub trait Versioning {
    /// Returns current version of this struct
    fn version(&self) -> usize;

    /// Sets to specific version.
    fn set_version(&mut self, version: usize);

    /// Increases to next version and returns the new version.
    fn next_version(&mut self) -> usize {
        self.set_version(self.version().wrapping_add(1));
        self.version()
    }
}
