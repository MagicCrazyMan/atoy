pub mod app;
pub mod bounding;
pub mod camera;
pub mod channel;
pub mod clock;
pub mod command;
pub mod ecs;
pub mod engine;
pub mod entity;
pub mod frustum;
pub mod looper;
pub mod plane;
pub mod property;
pub mod resource;
pub mod scene;
pub mod transparency;
pub mod versioning;
pub mod web;

pub type Rrc<T> = std::rc::Rc<std::cell::RefCell<T>>;
pub type Wrc<T> = std::rc::Weak<std::cell::RefCell<T>>;

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
