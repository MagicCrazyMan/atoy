use std::any::{Any, TypeId};

use super::{archetype::Archetype, error::Error};

pub trait Component {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ComponentKey(TypeId);

impl ComponentKey {
    /// Constructs a new shared component key by a component type and a key.
    #[inline]
    pub(super) fn new<C>() -> Self
    where
        C: Component + 'static,
    {
        Self(TypeId::of::<C>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct SharedComponentKey(TypeId, TypeId);

impl SharedComponentKey {
    /// Constructs a new shared component key by a component type and a key.
    #[inline]
    pub(super) fn new<C, T>() -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(TypeId::of::<C>(), TypeId::of::<T>())
    }
}

pub struct ComponentSet(
    pub(super) Vec<(ComponentKey, Box<dyn Any>)>, // non-shared components
    pub(super) Vec<SharedComponentKey>,           // shared components with only type id
    pub(super) Vec<(SharedComponentKey, Box<dyn Any>)>, // shared components with instance as well
);

impl ComponentSet {
    pub fn new() -> Self {
        Self(Vec::new(), Vec::new(), Vec::new())
    }

    pub fn with_component<C>(component: C) -> Self
    where
        C: Component + 'static,
    {
        Self(
            vec![(ComponentKey::new::<C>(), Box::new(component))],
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn with_shared_component<C, T>() -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(
            Vec::new(),
            vec![SharedComponentKey::new::<C, T>()],
            Vec::new(),
        )
    }

    pub fn with_shared_component_instance<C, T>(shared_component: C) -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(
            Vec::new(),
            Vec::new(),
            vec![(
                SharedComponentKey::new::<C, T>(),
                Box::new(shared_component),
            )],
        )
    }

    pub fn len(&self) -> usize {
        self.0.len() + self.1.len() + self.2.len()
    }

    pub fn archetype(&self) -> Archetype {
        let mut shard_keys = self
            .1
            .iter()
            .cloned()
            .chain(self.2.iter().map(|(k, _)| k.clone()))
            .collect::<Vec<_>>();
        shard_keys.sort_by(|a, b| a.cmp(b));
        Archetype(self.0.iter().map(|(id, _)| *id).collect(), shard_keys)
    }

    pub fn add<C>(&mut self, component: C) -> Result<(), Error>
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();
        let has_component = self.0.iter().any(|(k, _)| k == &key);
        if has_component {
            return Err(Error::DuplicateComponent);
        }

        self.0.push((key, Box::new(component)));
        self.0.sort_by(|(a, _), (b, _)| a.cmp(b));

        Ok(())
    }

    pub unsafe fn add_unique_unchecked<C>(&mut self, component: C)
    where
        C: Component + 'static,
    {
        self.0.push((ComponentKey::new::<C>(), Box::new(component)));
        self.0.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    pub fn remove<C>(&mut self) -> Result<C, Error>
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();
        let Some(index) = self.0.iter().position(|(k, _)| k == &key) else {
            return Err(Error::NoSuchComponent);
        };

        let removed = *self.0.remove(index).1.downcast::<C>().unwrap();

        Ok(removed)
    }

    pub fn add_shared<C, T>(&mut self) -> Result<(), Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();
        let has_component = self
            .1
            .iter()
            .chain(self.2.iter().map(|(k, _)| k))
            .any(|k| k == &key);
        if has_component {
            return Err(Error::DuplicateComponent);
        }

        self.1.sort_by(|a, b| a.cmp(b));

        Ok(())
    }

    pub unsafe fn add_shared_unique_unchecked<C, T>(&mut self)
    where
        C: Component + 'static,
        T: 'static,
    {
        self.1.push(SharedComponentKey::new::<C, T>());
        self.1.sort_by(|a, b| a.cmp(b));
    }

    pub fn remove_shared<C, T>(&mut self) -> Result<(), Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();
        let Some(index) = self.1.iter().position(|k| k == &key) else {
            return Err(Error::NoSuchComponent);
        };

        self.1.remove(index);

        Ok(())
    }

    pub fn add_shared_instance<C, T>(&mut self, shared_component: C) -> Result<(), Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();
        let has_component = self
            .1
            .iter()
            .chain(self.2.iter().map(|(k, _)| k))
            .any(|k| k == &key);
        if has_component {
            return Err(Error::DuplicateComponent);
        }

        self.2.push((key, Box::new(shared_component)));
        self.2.sort_by(|(a, _), (b, _)| a.cmp(b));

        Ok(())
    }

    pub unsafe fn add_shared_instance_unique_unchecked<C, T>(&mut self, shared_component: C)
    where
        C: Component + 'static,
        T: 'static,
    {
        self.2.push((
            SharedComponentKey::new::<C, T>(),
            Box::new(shared_component),
        ));
        self.2.sort_by(|(a, _), (b, _)| a.cmp(b));
    }

    pub fn remove_shared_instance<C, T>(&mut self) -> Result<C, Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();
        let Some(index) = self.2.iter().position(|(k, _)| k == &key) else {
            return Err(Error::NoSuchComponent);
        };

        let removed = *self.2.remove(index).1.downcast::<C>().unwrap();

        Ok(removed)
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
