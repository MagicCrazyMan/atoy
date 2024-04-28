pub mod channel;
pub mod clock;
pub mod engine;
pub mod entity;
pub mod property;
pub mod scene;
pub mod viewer;
pub mod webgl;

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}