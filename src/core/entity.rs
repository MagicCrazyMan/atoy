use std::any::Any;

use gl_matrix4rust::mat4::Mat4;
use proc::AsAny;

use super::{bounding::BoundingVolume, AsAny};

pub trait Component<RenderType>: AsAny {
    fn property(&self, name: &str) -> Option<&dyn Any>;
}

pub trait Entity<RenderType>: EntityMatrices + EntityBoundingVolume + AsAny {
    // fn components(&self) -> &[&dyn Component<RenderType>];

    // fn components_mut(&self) -> &[&mut dyn Component<RenderType>];
}

pub trait EntityMatrices {
    fn local_matrix(&self) -> &Mat4<f64>;

    fn model_matrix(&self) -> &Mat4<f64>;

    fn set_model_matrix(&mut self, mat: Mat4<f64>);
}

pub trait EntityBoundingVolume {
    fn bounding_volume(&self) -> &BoundingVolume;
}

pub struct Collection<RenderType> {
    entities: Vec<Box<dyn Entity<RenderType>>>,
}

#[derive(AsAny)]
struct A {

}