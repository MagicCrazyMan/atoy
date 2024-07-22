use super::app::App;

/// A plugin for [`App`].
pub trait Plugin {
    fn plugin(&mut self, app: &mut App);

    fn plugout(&mut self, app: &mut App);
}
