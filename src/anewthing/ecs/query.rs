use std::{any::Any, marker::PhantomData};

use hashbrown::HashMap;

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, SharedComponentKey},
};

/// Query types.
pub enum QueryType {
    /// Archetype does not match.
    NotMatched,
    /// Archetype contains specific component at specif chunk index.
    With((ComponentKey, usize)),
    /// Archetype contains specific shared component.
    WithShared(SharedComponentKey),
    /// Archetype contains no specific component.
    Without,
}

/// A query operator returns a [`QueryType`].
pub trait QueryOp {
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized;
}

/// A simple query operator MUST returns neither
/// [`QueryType::With`] or [`QueryType::WithShared`].
pub trait SimpleQueryOp<C>: QueryOp {}

/// Queries components with a specific component type.
pub struct With<C>(PhantomData<C>);

impl<C> QueryOp for With<C>
where
    C: Component + 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.component_index::<C>() {
            Some(position) => QueryType::With((ComponentKey::new::<C>(), position)),
            None => QueryType::NotMatched,
        }
    }
}

impl<C> SimpleQueryOp<C> for With<C> where C: Component + 'static {}

/// Queries components without a specific component type.
pub struct Without<C>(PhantomData<C>);

impl<C> QueryOp for Without<C>
where
    C: Component + 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.has_component::<C>() {
            true => QueryType::NotMatched,
            false => QueryType::Without,
        }
    }
}

/// Queries components with or without a specific component type.
/// Do not query entities with only [`Maybe`] operators, which is meaningless.
pub struct Maybe<C>(PhantomData<C>);

impl<C> QueryOp for Maybe<C>
where
    C: Component + 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.component_index::<C>() {
            Some(position) => QueryType::With((ComponentKey::new::<C>(), position)),
            None => QueryType::Without,
        }
    }
}

/// Queries shared components with a specific component type.
pub struct WithShared<C, T>(PhantomData<C>, PhantomData<T>)
where
    C: Component + 'static,
    T: 'static;

impl<C, T> QueryOp for WithShared<C, T>
where
    C: Component + 'static,
    T: 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.has_shared_component::<C, T>() {
            true => QueryType::WithShared(SharedComponentKey::new::<C, T>()),
            false => QueryType::NotMatched,
        }
    }
}

impl<C, T> SimpleQueryOp<C> for WithShared<C, T>
where
    C: Component + 'static,
    T: 'static,
{
}

/// Queries shared components without a specific component type.
pub struct WithoutShared<C, T>(PhantomData<C>, PhantomData<T>);

impl<C, T> QueryOp for WithoutShared<C, T>
where
    C: Component + 'static,
    T: 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.has_shared_component::<C, T>() {
            true => QueryType::NotMatched,
            false => QueryType::Without,
        }
    }
}

/// Queries shared components with or without a specific component type.
pub struct MaybeShared<C, T>(PhantomData<C>, PhantomData<T>)
where
    C: Component + 'static,
    T: 'static;

impl<C, T> QueryOp for MaybeShared<C, T>
where
    C: Component + 'static,
    T: 'static,
{
    fn query(archetype: &Archetype) -> QueryType
    where
        Self: Sized,
    {
        match archetype.has_shared_component::<C, T>() {
            true => QueryType::WithShared(SharedComponentKey::new::<C, T>()),
            false => QueryType::Without,
        }
    }
}

/// A simple query queries entities using [`SimpleQueryOp`] only.
pub trait QuerySimple<'a, A> {
    fn query_simple(
        archetype: &Archetype,
        components: &'a mut [Box<dyn Any>],
        shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
    ) -> Option<A>
    where
        Self: Sized;
}

/// Implements simple query for all simple query operators.
impl<'a, A, S> QuerySimple<'a, &'a mut A> for S
where
    A: Component + 'static,
    S: SimpleQueryOp<A> + 'static,
{
    fn query_simple(
        archetype: &Archetype,
        components: &'a mut [Box<dyn Any>],
        shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
    ) -> Option<&'a mut A>
    where
        Self: Sized,
    {
        match S::query(archetype) {
            QueryType::NotMatched => None,
            QueryType::With((_, position)) => Some(components[position].downcast_mut().unwrap()),
            QueryType::WithShared(key) => Some(
                shared_components
                    .get_mut(&key)
                    .unwrap()
                    .downcast_mut()
                    .unwrap(),
            ),
            _ => unreachable!(),
        }
    }
}

/// A macro rule implements simple query for tuple simple query operators.
macro_rules! query_simple {
    (
        $(
            ($c: tt, $q: tt),
        )+
    ) => {
        impl<'a, $($c, $q,)+> QuerySimple<'a, ($(&'a mut $c,)+)> for ($($q,)+)
        where
            $(
                $c: Component + 'static,
                $q: SimpleQueryOp<$c> + 'static,
            )+
        {
            fn query_simple(
                archetype: &Archetype,
                components: &'a mut [Box<dyn Any>],
                shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
            ) -> Option<($(&'a mut $c,)+)>
            where
                Self: Sized,
            {
                Some((
                    $(
                        unsafe {
                            match $q::query(archetype) {
                                QueryType::NotMatched => return None,
                                QueryType::With((_, position)) => &mut *(components[position].downcast_mut::<$c>().unwrap() as *mut $c),
                                QueryType::WithShared(key) =>  &mut *(shared_components.get_mut(&key).unwrap().downcast_mut::<$c>().unwrap() as *mut $c),
                                _ => unreachable!(),
                            }
                        },
                    )+
                ))
            }
        }
    };
}

// Repeats query_simple for tuple simple query operators 16 times making it supports queries maximum 16 components at once.

query_simple! {
    (A, S1),
}
query_simple! {
    (A, S1),
    (B, S2),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
    (L, S12),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
    (L, S12),
    (M, S13),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
    (L, S12),
    (M, S13),
    (N, S14),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
    (L, S12),
    (M, S13),
    (N, S14),
    (O, S15),
}
query_simple! {
    (A, S1),
    (B, S2),
    (C, S3),
    (D, S4),
    (E, S5),
    (F, S6),
    (G, S7),
    (H, S8),
    (I, S9),
    (J, S10),
    (K, S11),
    (L, S12),
    (M, S13),
    (N, S14),
    (O, S15),
    (P, S16),
}

/// Queried components returns by [`QueryComplex`].
pub struct Queried<'a> {
    components: HashMap<ComponentKey, &'a mut Box<dyn Any>>,
    shared_components: HashMap<SharedComponentKey, &'a mut Box<dyn Any>>,
}

impl<'a> Queried<'a> {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
            shared_components: HashMap::new(),
        }
    }

    /// Returns a component with a specific component type.
    pub fn component<C>(&'a mut self) -> Option<&'a mut C>
    where
        C: Component + 'static,
    {
        Some(
            self.components
                .get_mut(&ComponentKey::new::<C>())?
                .downcast_mut::<C>()
                .unwrap(),
        )
    }

    /// Returns a component with a specific component type.
    /// Panic if no such component.
    pub fn component_unchecked<C>(&'a mut self) -> &'a mut C
    where
        C: Component + 'static,
    {
        self.components
            .get_mut(&ComponentKey::new::<C>())
            .unwrap()
            .downcast_mut::<C>()
            .unwrap()
    }

    /// Returns a shared component with a specific component type.
    pub fn shared_component<C, T>(&'a mut self) -> Option<&'a mut C>
    where
        C: Component + 'static,
        T: 'static,
    {
        Some(
            self.shared_components
                .get_mut(&SharedComponentKey::new::<C, T>())?
                .downcast_mut::<C>()
                .unwrap(),
        )
    }

    /// Returns a shared component with a specific component type.
    /// Panic if no such component.
    pub fn shared_component_unchecked<C, T>(&'a mut self) -> &'a mut C
    where
        C: Component + 'static,
        T: 'static,
    {
        self.shared_components
            .get_mut(&SharedComponentKey::new::<C, T>())
            .unwrap()
            .downcast_mut::<C>()
            .unwrap()
    }
}

/// A complex query queries components using all kinds of query operators, returns [`Queried`] as result.
pub trait QueryComplex<'a> {
    fn query_complex(
        archetype: &Archetype,
        components: &'a mut [Box<dyn Any>],
        shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
    ) -> Option<Queried<'a>>
    where
        Self: Sized;
}

impl<'a, S> QueryComplex<'a> for S
where
    S: QueryOp + 'static,
{
    fn query_complex(
        archetype: &Archetype,
        components: &'a mut [Box<dyn Any>],
        shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
    ) -> Option<Queried<'a>>
    where
        Self: Sized,
    {
        let mut queried = None;

        match S::query(archetype) {
            QueryType::Without => {
                // do nothing
            }
            QueryType::With((key, position)) => {
                queried
                    .get_or_insert_with(|| Queried::new())
                    .components
                    .insert_unique_unchecked(key, &mut components[position]);
            }
            QueryType::WithShared(key) => {
                queried
                    .get_or_insert_with(|| Queried::new())
                    .shared_components
                    .insert_unique_unchecked(key, shared_components.get_mut(&key).unwrap());
            }
            QueryType::NotMatched => return None,
        };

        queried
    }
}

/// A macro rule implements complex query for tuple query operators.
macro_rules! query_complex {
    (
        $(
            $q: tt
        ),+
    ) => {
        impl<'a, $($q,)+> QueryComplex<'a> for ($($q,)+)
        where
            $(
                $q: QueryOp + 'static,
            )+
        {
            fn query_complex(
                archetype: &Archetype,
                components: &'a mut [Box<dyn Any>],
                shared_components: &'a mut HashMap<SharedComponentKey, Box<dyn Any>>,
            ) -> Option<Queried<'a>>
            where
                Self: Sized,
            {
                let mut queried = None;

                $(
                    match $q::query(archetype) {
                        QueryType::Without => {
                            // do nothing
                        }
                        QueryType::With((key, position)) => {
                            queried
                                .get_or_insert_with(|| Queried::new())
                                .components
                                .insert_unique_unchecked(key, unsafe { &mut *(&mut components[position] as *mut Box<dyn Any>) });
                        }
                        QueryType::WithShared(key) => {
                            queried
                                .get_or_insert_with(|| Queried::new())
                                .shared_components
                                .insert_unique_unchecked(key, unsafe { &mut *(shared_components.get_mut(&key).unwrap() as *mut Box<dyn Any>) });
                        }
                        QueryType::NotMatched => return None,
                    };
                )+

                queried
            }
        }
    };
}

// Repeats query_complex for tuple query operators 16 times making it supports queries maximum 16 components at once.

query_complex!(A);
query_complex!(A, B);
query_complex!(A, B, C);
query_complex!(A, B, C, D);
query_complex!(A, B, C, D, E);
query_complex!(A, B, C, D, E, F);
query_complex!(A, B, C, D, E, F, G);
query_complex!(A, B, C, D, E, F, G, H);
query_complex!(A, B, C, D, E, F, G, H, I);
query_complex!(A, B, C, D, E, F, G, H, I, J);
query_complex!(A, B, C, D, E, F, G, H, I, J, K);
query_complex!(A, B, C, D, E, F, G, H, I, J, K, L);
query_complex!(A, B, C, D, E, F, G, H, I, J, K, L, M);
query_complex!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
query_complex!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
query_complex!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
