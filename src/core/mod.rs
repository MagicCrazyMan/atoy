pub mod bounding;
pub mod channel;
pub mod clock;
pub mod engine;
pub mod entity;
pub mod frustum;
pub mod plane;
pub mod property;
pub mod scene;
pub mod viewer;
pub mod web;

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
