use std::any::TypeId;

use smallvec::SmallVec;

use super::{component::Component, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(pub(super) SmallVec<[TypeId; 3]>);

impl Archetype {
    // pub(super) fn new<I>(type_ids: I) -> Result<Self, Error>
    // where
    //     I: IntoIterator<Item = TypeId>,
    // {
    //     let type_ids: SmallVec<[TypeId; 3]> = type_ids.into_iter().collect();
    //     let mut post_processed = type_ids.clone();
    //     post_processed.sort();
    //     post_processed.dedup();

    //     if type_ids.len() != post_processed.len() {
    //         return Err(Error::DuplicateComponent);
    //     }

    //     for (i, j) in type_ids.iter().zip(post_processed.iter()) {
    //         if i != j {
    //             return Err(Error::DuplicateComponent);
    //         }
    //     }

    //     Ok(Self(type_ids))
    // }

    // // pub(super) fn new_unchecked<I>(component_types: I) -> Self
    // // where
    // //     I: IntoIterator<Item = TypeId>,
    // // {
    // //     Self(component_types.into_iter().collect())
    // // }

    // pub(super) fn from_components(components: &mut Vec<Box<dyn Component>>) -> Result<Self, Error> {
    //     let archetype = Self::new(components.iter().map(|component| component.type_id()))?;

    //     for (i, j) in archetype.0.iter()

    //     Self(component_types)
    // }
}

impl Archetype {
    pub fn components_per_entity(&self) -> usize {
        self.0.len()
    }

    pub fn component_id(&self, index: usize) -> Option<TypeId> {
        self.0.get(index).cloned()
    }

    pub fn add_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut new_archetype = self.0.clone();
        new_archetype.push(TypeId::of::<C>());
        new_archetype.sort();
        new_archetype.dedup();
        if new_archetype.len() == self.0.len() {
            Ok(Self(new_archetype))
        } else {
            Err(Error::DuplicateComponent)
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
