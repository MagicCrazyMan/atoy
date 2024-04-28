use std::any::Any;

use super::AsAny;

pub trait Component<RenderType>: AsAny {
    fn property(&self, name: &str) -> Option<&dyn Any>;
}

pub trait Entity<RenderType>: AsAny {
    fn components(&self) -> &[&dyn Component<RenderType>];
}

pub struct Collection<RenderType> {
    entities: Vec<Box<dyn Entity<RenderType>>>
}