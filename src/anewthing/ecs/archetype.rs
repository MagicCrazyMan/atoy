use std::any::TypeId;

use crate::anewthing::key::Key;

use super::{
    component::{Component, ComponentKey, SharedComponentKey},
    error::Error,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(
    pub(super) Vec<ComponentKey>,       // non-shared components
    pub(super) Vec<SharedComponentKey>, // shared components
);

impl Archetype {
    pub fn new() -> Self {
        Self(Vec::new(), Vec::new())
    }

    pub fn with_component<C>() -> Self
    where
        C: Component + 'static,
    {
        Self(vec![ComponentKey::new::<C>()], Vec::new())
    }

    pub fn with_shared_component<C>(key: Key) -> Self
    where
        C: Component + 'static,
    {
        Self(Vec::new(), vec![(SharedComponentKey::new::<C>(key))])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn has_component<C>(&self) -> bool
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();
        self.0.iter().any(|k| k == &key)
    }

    pub fn has_shared_component<C>(&self, key: &Key) -> bool
    where
        C: Component + 'static,
    {
        let type_id = TypeId::of::<C>();
        self.1
            .iter()
            .any(|k| k.type_id() == &type_id && k.key() == key)
    }

    pub fn add_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut components = self.0.clone();
        components.push(ComponentKey::new::<C>());
        components.sort();
        components.dedup();
        if components.len() == self.0.len() {
            Ok(Self(components, self.1.clone()))
        } else {
            Err(Error::DuplicateComponent)
        }
    }

    pub fn remove_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();

        let mut components = self.0.clone();
        components.retain(|k| k != &key);
        if components.len() == self.0.len() {
            Err(Error::NoSuchComponent)
        } else {
            Ok(Self(components, self.1.clone()))
        }
    }

    pub fn add_shared_component<C>(&self, key: Key) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut components = self.1.clone();
        components.push(SharedComponentKey::new::<C>(key));
        components.sort();
        components.dedup();
        if components.len() == self.0.len() {
            Ok(Self(self.0.clone(), components))
        } else {
            Err(Error::DuplicateComponent)
        }
    }

    pub fn remove_shared_component<C>(&self, key: &Key) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let type_id = TypeId::of::<C>();

        let mut components = self.1.clone();
        components.retain(|k| k.type_id() != &type_id || k.key() != key);
        if components.len() == self.0.len() {
            Err(Error::NoSuchComponent)
        } else {
            Ok(Self(self.0.clone(), components))
        }
    }
}

// pub trait ToArchetype {
//     fn to_archetype(&self) -> Archetype;
// }

// pub trait AsArchetype {
//     fn as_archetype() -> Archetype;
// }

// impl<A0> AsArchetype for A0
// where
//     A0: Component + 'static,
// {
//     fn as_archetype() -> Archetype {
//         Archetype::new([TypeId::of::<A0>()])
//     }
// }

// macro_rules! as_archetype {
//     ($($ct: tt),+) => {
//         impl<$($ct,)+> AsArchetype for ($($ct,)+)
//         where
//             $(
//                 $ct: Component + 'static,
//             )+
//         {
//             fn as_archetype() -> Archetype {
//                 Archetype::new([
//                     $(
//                         TypeId::of::<$ct>(),
//                     )+
//                 ])
//             }
//         }
//     };
// }

// as_archetype!(A0);
// as_archetype!(A0, A1);
// as_archetype!(A0, A1, A2);
// as_archetype!(A0, A1, A2, A3);
