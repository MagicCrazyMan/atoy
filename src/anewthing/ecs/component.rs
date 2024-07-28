use std::any::{Any, TypeId};

use super::{archetype::Archetype, error::Error};

pub trait Component {}

pub struct ComponentSet(pub(super) Vec<(TypeId, Box<dyn Any>)>);

impl ComponentSet {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_component<C>(component: C) -> Self
    where
        C: Component + 'static,
    {
        Self(vec![(TypeId::of::<C>(), Box::new(component))])
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn archetype(&self) -> Archetype {
        Archetype(self.0.iter().map(|(id, _)| *id).collect())
    }

    pub fn add<C>(mut self, component: C) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let type_id = TypeId::of::<C>();
        let has_component = self.0.iter().any(|(id, _)| id == &type_id);
        if has_component {
            return Err(Error::DuplicateComponent);
        }

        self.0.push((type_id, Box::new(component)));
        self.0.sort_by(|(a, _), (b, _)| a.cmp(b));

        Ok(self)
    }

    pub unsafe fn add_unique_unchecked<C>(mut self, component: C) -> Self
    where
        C: Component + 'static,
    {
        self.0.push((TypeId::of::<C>(), Box::new(component)));
        self.0.sort_by(|(a, _), (b, _)| a.cmp(b));
        self
    }

    pub unsafe fn remove_unchecked<C>(mut self) -> (Self, C)
    where
        C: Component + 'static,
    {
        let type_id = TypeId::of::<C>();
        let index = self.0.iter().position(|(id, _)| id == &type_id).unwrap();
        let removed = *self.0.remove(index).1.downcast::<C>().unwrap();
        (self, removed)
    }

    pub fn remove<C>(mut self) -> Result<(Self, C), Error>
    where
        C: Component + 'static,
    {
        let type_id = TypeId::of::<C>();
        let Some(index) = self.0.iter().position(|(id, _)| id == &type_id) else {
            return Err(Error::NoSuchComponent);
        };

        let removed = *self.0.remove(index).1.downcast::<C>().unwrap();

        Ok((self, removed))
    }
}

// #[derive(AsAny, Component)]
// pub struct Transformation {
//     translation: Vec3<f64>,
//     rotation: Quat<f64>,
//     scale: Vec3<f64>,

//     model_matrix: Mat4<f64>,
// }

// impl Transformation {
//     pub fn new() -> Self {
//         Self {
//             translation: Vec3::<f64>::new_zero(),
//             rotation: Quat::<f64>::new_identity(),
//             scale: Vec3::<f64>::new(1.0, 1.0, 1.0),

//             model_matrix: Mat4::<f64>::new_identity(),
//         }
//     }

//     pub fn with_translation_rotation_scale(
//         translation: Vec3<f64>,
//         rotation: Quat<f64>,
//         scale: Vec3<f64>,
//     ) -> Self {
//         Self {
//             model_matrix: Mat4::<f64>::from_rotation_translation_scale(
//                 &rotation,
//                 &translation,
//                 &scale,
//             ),

//             translation,
//             rotation,
//             scale,
//         }
//     }

//     pub fn translation(&self) -> &Vec3<f64> {
//         &self.translation
//     }

//     pub fn rotation(&self) -> &Quat<f64> {
//         &self.rotation
//     }

//     pub fn scale(&self) -> &Vec3<f64> {
//         &self.scale
//     }

//     pub fn set_translation(&mut self, translation: Vec3<f64>) {
//         self.translation = translation;
//         self.update_model_matrix();
//     }

//     pub fn set_rotation(&mut self, rotation: Quat<f64>) {
//         self.rotation = rotation;
//         self.update_model_matrix();
//     }

//     pub fn set_scale(&mut self, scale: Vec3<f64>) {
//         self.scale = scale;
//         self.update_model_matrix();
//     }

//     pub fn set_translation_rotation_scale(
//         &mut self,
//         translation: Vec3<f64>,
//         rotation: Quat<f64>,
//         scale: Vec3<f64>,
//     ) {
//         self.translation = translation;
//         self.rotation = rotation;
//         self.scale = scale;
//         self.update_model_matrix();
//     }

//     pub fn model_matrix(&self) -> &Mat4<f64> {
//         &self.model_matrix
//     }

//     fn update_model_matrix(&mut self) {
//         self.model_matrix = Mat4::<f64>::from_rotation_translation_scale(
//             &self.rotation,
//             &self.translation,
//             &self.scale,
//         );
//     }
// }
