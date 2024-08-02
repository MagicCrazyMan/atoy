use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use hashbrown::{HashMap, HashSet};

use crate::anewthing::key::Key;

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, SharedComponentKey},
};

pub enum QueryType {
    With(usize),
    Without,
    Maybe(Option<usize>),
    WithShared(SharedComponentKey),
    WithoutShared,
    MaybeShared(Option<SharedComponentKey>),
}

pub trait QueryOp {
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized;
}

/// Queries entities and components with a specific component.
pub struct With<C>(PhantomData<C>);

/// Queries entities and components without a specific component.
pub struct Without<C>(PhantomData<C>)
where
    C: Component + 'static;

/// Queries entities and components with or without a specific component.
/// Do not query entities with only [`Maybe`] operators, which is meaningless.
pub struct Maybe<C>(PhantomData<C>)
where
    C: Component + 'static;

pub struct WithShared<C>(PhantomData<C>, Key)
where
    C: Component + 'static;

pub struct WithoutShared<C>(PhantomData<C>, Key)
where
    C: Component + 'static;

pub struct MaybeShared<C>(PhantomData<C>, Key)
where
    C: Component + 'static;

/// Simple query. only [`With`] query operator is available.
pub trait QuerySimple<'a, A> {
    fn query(archetype: &Archetype, components: &'a mut [Box<dyn Any>]) -> Option<A>
    where
        Self: Sized;
}

impl<'a, A> QuerySimple<'a, &'a mut A> for With<A>
where
    A: Component + 'static,
{
    fn query(archetype: &Archetype, components: &'a mut [Box<dyn Any>]) -> Option<&'a mut A>
    where
        Self: Sized,
    {
        let position = archetype.component_index::<A>()?;
        Some(components[position].downcast_mut().unwrap())
    }
}

macro_rules! simple_query {
    ($($comp: tt),+) => {
        impl<'a, $($comp,)+> QuerySimple<'a, ($(&'a mut $comp,)+)> for ($(With<$comp>,)+)
        where
            $(
                $comp: Component + 'static,
            )+
        {
            fn query(archetype: &Archetype, components: &'a mut [Box<dyn Any>]) -> Option<($(&'a mut $comp,)+)>
            where
                Self: Sized,
            {
                Some((
                    $(
                        unsafe { &mut *(components[archetype.component_index::<$comp>()?].downcast_mut().unwrap() as *mut $comp) },
                    )+
                ))
            }
        }
    };
}

simple_query!(A);
simple_query!(A, B);
simple_query!(A, B, C);
simple_query!(A, B, C, D);
simple_query!(A, B, C, D, E);
simple_query!(A, B, C, D, E, F);
simple_query!(A, B, C, D, E, F, G);
simple_query!(A, B, C, D, E, F, G, H);
simple_query!(A, B, C, D, E, F, G, H, I);
simple_query!(A, B, C, D, E, F, G, H, I, J);
simple_query!(A, B, C, D, E, F, G, H, I, J, K);
simple_query!(A, B, C, D, E, F, G, H, I, J, K, L);
simple_query!(A, B, C, D, E, F, G, H, I, J, K, L, M);
simple_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
simple_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
simple_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

// impl<'a, A> QuerySimple<'a, (&'a mut A,)> for (With<A>,) {
//     fn query(archetype: &Archetype, components: &'a mut [Box<dyn Any>]) -> Option<(&'a mut A,)>
//     where
//         Self: Sized,
//     {
//         todo!()
//     }
// }

// impl<'a, A, B> QuerySimple<'a, (&'a mut A, &'a mut B)> for (With<A>, With<B>)
// where
//     A: Component + 'static,
//     B: Component + 'static,
// {
//     fn query(
//         archetype: &Archetype,
//         components: &'a mut [Box<dyn Any>],
//     ) -> Option<(&'a mut A, &'a mut B)>
//     where
//         Self: Sized,
//     {
//         unsafe {
//             Some((
//                 &mut *(components[archetype.component_index::<A>()?]
//                     .downcast_mut()
//                     .unwrap() as *mut A),
//                 components[archetype.component_index::<B>()?]
//                     .downcast_mut()
//                     .unwrap(),
//             ))
//         }
//     }
// }

// impl<A> QueryComplex for A
// where
//     A: QueryOp,
// {
//     fn query(archetype: &Archetype)
//     where
//         Self: Sized,
//     {
//         A::query(archetype);
//     }
// }

// pub(super) trait QueryComplex {
//     fn query(archetype: &Archetype)
//     where
//         Self: Sized;
// }

// impl<A> QueryComplex for A
// where
//     A: QueryOp,
// {
//     fn query(archetype: &Archetype)
//     where
//         Self: Sized,
//     {
//         A::query(archetype);
//     }
// }
