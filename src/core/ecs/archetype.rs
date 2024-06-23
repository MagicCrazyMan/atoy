use std::any::TypeId;

use smallvec::SmallVec;

use super::component::Component;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(SmallVec<[TypeId; 3]>);

impl Archetype {
    pub(super) fn new<I>(component_types: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        let mut component_types: SmallVec<[TypeId; 3]> = component_types.into_iter().collect();
        component_types.sort();
        component_types.dedup();
        Self(component_types)
    }

    pub(super) fn new_unchecked<I>(component_types: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        Self(component_types.into_iter().collect())
    }

    pub(super) fn from_vec_unchecked(component_types: SmallVec<[TypeId; 3]>) -> Self {
        Self(component_types)
    }
}

pub trait ToArchetype {
    fn to_archetype(&self) -> Archetype;
}

pub trait AsArchetype {
    fn as_archetype() -> Archetype;
}

impl<A0> AsArchetype for A0
where
    A0: Component + 'static,
{
    fn as_archetype() -> Archetype {
        Archetype::new([TypeId::of::<A0>()])
    }
}

macro_rules! as_archetype {
    ($($ct: tt),+) => {
        impl<$($ct,)+> AsArchetype for ($($ct,)+)
        where
            $(
                $ct: Component + 'static,
            )+
        {
            fn as_archetype() -> Archetype {
                Archetype::new([
                    $(
                        TypeId::of::<$ct>(),
                    )+
                ])
            }
        }
    };
}

as_archetype!(A0);
as_archetype!(A0, A1);
as_archetype!(A0, A1, A2);
as_archetype!(A0, A1, A2, A3);
