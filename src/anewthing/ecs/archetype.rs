use std::any::TypeId;

use smallvec::{smallvec, SmallVec};

use super::{component::Component, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(pub(super) SmallVec<[TypeId; 2]>);

impl Archetype {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn with_component<C>() -> Self
    where
        C: Component + 'static,
    {
        Self(smallvec![TypeId::of::<C>()])
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn component_type(&self, index: usize) -> Option<TypeId> {
        self.0.get(index).cloned()
    }

    pub fn has_component<C>(&self) -> bool
    where
        C: Component + 'static,
    {
        self.0.iter().any(|id| id == &TypeId::of::<C>())
    }

    pub fn add_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut components = self.0.clone();
        components.push(TypeId::of::<C>());
        components.sort();
        components.dedup();
        if components.len() == self.0.len() {
            Ok(Self(components))
        } else {
            Err(Error::DuplicateComponent)
        }
    }

    pub fn remove_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut components = self.0.clone();
        components.retain(|type_id| type_id != &TypeId::of::<C>());
        if components.len() == self.0.len() {
            Err(Error::NoSuchComponent)
        } else {
            Ok(Self(components))
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
