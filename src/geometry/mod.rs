pub mod cube;

use std::{any::Any, collections::HashMap};

use crate::material::Material;

pub trait Geometry: Any {
    fn vertices(&self) -> Option<Vec<f32>>;

    fn normals(&self) -> Option<Vec<f32>>;

    fn textures(&self) -> Option<Vec<f32>>;

    fn properties(&self) -> &HashMap<String, Box<dyn Any>>;

    fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>>;

    fn material(&self) -> Option<&dyn Material>;

    // fn set_material<M: Material + Sized + 'static>(&mut self, material: Option<M>);
}
