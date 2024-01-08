use std::any::Any;

use crate::viewer::Viewer;

pub trait Controller {
    fn on_add(&mut self, viewer: &mut Viewer);

    fn on_remove(&mut self, viewer: &mut Viewer);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
