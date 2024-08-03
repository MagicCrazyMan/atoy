use std::{any::Any, marker::PhantomData};

use hashbrown::HashMap;

use super::{
    archetype::Archetype,
    component::{Component, ComponentKey, SharedComponentKey},
    manager::{ChunkItem, EntityManager, SharedComponentItem},
};

/// Query types.
#[derive(Clone, Copy)]
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
/// [`QueryType::NotMatched`], [`QueryType::With`] or [`QueryType::WithShared`].
pub trait QueryOpSimple<C>: QueryOp {}

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

impl<C> QueryOpSimple<C> for With<C> where C: Component + 'static {}

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

impl<C, T> QueryOpSimple<C> for WithShared<C, T>
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

/// A simple query returns a [`QuerySimpleIter`] using [`SimpleQueryOp`].
pub trait QuerySimple<'a, A, S> {
    fn query_simple(manager: &'a mut EntityManager) -> QuerySimpleIter<'a, A, S>
    where
        Self: Sized;
}

/// Implements simple query for all simple query operators.
impl<'a, A, S> QuerySimple<'a, A, S> for S
where
    A: Component + 'static,
    S: QueryOpSimple<A> + 'static,
{
    fn query_simple(manager: &'a mut EntityManager) -> QuerySimpleIter<'a, A, S>
    where
        Self: Sized,
    {
        QuerySimpleIter::new(manager)
    }
}

pub struct QuerySimpleIter<'a, A, S> {
    shared_components: &'a mut HashMap<SharedComponentKey, SharedComponentItem>,
    chunks: hashbrown::hash_map::IterMut<'a, Archetype, ChunkItem>,
    chunk: Option<(&'a Archetype, &'a mut ChunkItem, QueryType, usize)>,
    _k: PhantomData<(A, S)>,
}

impl<'a, A, S> QuerySimpleIter<'a, A, S> {
    fn new(manager: &'a mut EntityManager) -> Self {
        Self {
            shared_components: &mut manager.shared_components,
            chunks: manager.chunks.iter_mut(),
            chunk: None,
            _k: PhantomData,
        }
    }
}

impl<'a, A, S> Iterator for QuerySimpleIter<'a, A, S>
where
    A: Component + 'static,
    S: QueryOpSimple<A> + 'static,
{
    type Item = &'a mut A;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, chunk_item, _, index)) = self.chunk.as_mut() {
            *index += 1;

            if *index >= chunk_item.entity_keys.len() {
                self.chunk = None;
            }
        }

        if self.chunk.is_none() {
            // finds next chunk that matches the query
            while let Some((archetype, chunk_item)) = self.chunks.next() {
                let query_type = S::query(archetype);
                match &query_type {
                    QueryType::NotMatched => continue,
                    QueryType::With(_) | QueryType::WithShared(_) => {
                        self.chunk = Some((archetype, chunk_item, query_type, 0));
                    }
                    _ => unreachable!(),
                }
            }
        }

        let (archetype, chunk_item, query_type, index) = self.chunk.as_mut()?;

        let component: *mut Box<dyn Any> = match query_type {
            QueryType::With((_, position)) => {
                &mut chunk_item.components[*index * archetype.components_len() + *position]
                    .component
            }
            QueryType::WithShared(key) => {
                &mut self.shared_components.get_mut(key).unwrap().component
            }
            _ => unreachable!(),
        };

        unsafe { Some((*component).downcast_mut::<A>().unwrap()) }
    }
}

/// A macro rule implements simple query for tuple simple query operators.
macro_rules! query_simple {
    (
        $query: tt,
        $iter: tt,
        $count: tt,
        $(
            ($i: tt, $c: tt, $q: tt),
        )+
    ) => {
        pub trait $query<'a, $($c, $q,)+> {
            fn query_simple(manager: &'a mut EntityManager) -> $iter<'a, $($c, $q,)+>
            where
                Self: Sized;
        }


        impl<'a, $($c, $q,)+> $query<'a, $($c, $q,)+> for ($($q,)+)
        where
            $(
                $c: Component + 'static,
                $q: QueryOpSimple<$c> + 'static,
            )+
        {
            fn query_simple(manager: &'a mut EntityManager) -> $iter<'a, $($c, $q,)+>
            where
                Self: Sized,
            {
                $iter::new(manager)
            }
        }

        pub struct $iter<'a, $($c, $q,)+>{
            shared_components: &'a mut HashMap<SharedComponentKey, SharedComponentItem>,
            chunks: hashbrown::hash_map::IterMut<'a, Archetype, ChunkItem>,
            chunk: Option<(&'a Archetype, &'a mut ChunkItem, [QueryType; $count], usize)>,
            _k: PhantomData<&'a ($($c, $q,)+)>,
        }

        impl<'a, $($c, $q,)+> $iter<'a, $($c, $q,)+> {
            fn new(manager: &'a mut EntityManager) -> Self {
                Self {
                    shared_components: &mut manager.shared_components,
                    chunks: manager.chunks.iter_mut(),
                    chunk: None,
                    _k: PhantomData,
                }
            }
        }

        impl<'a, $($c, $q,)+> Iterator for $iter<'a, $($c, $q,)+>
        where
            $(
                $c: Component + 'static,
                $q: QueryOpSimple<$c> + 'static,
            )+
        {
            type Item = ($(&'a mut $c,)+);

            fn next(&mut self) -> Option<Self::Item> {
                if let Some((_, chunk_item, _, index)) = self.chunk.as_mut() {
                    *index += 1;

                    if *index >= chunk_item.entity_keys.len() {
                        self.chunk = None;
                    }
                }

                if self.chunk.is_none() {
                    while let Some((archetype, chunk_item)) = self.chunks.next() {
                        let mut query_types = [QueryType::NotMatched; $count];
                        $(
                            {
                                let query_type = $q::query(archetype);
                                match &query_type {
                                    QueryType::NotMatched => continue,
                                    QueryType::With(_) | QueryType::WithShared(_) => {
                                        query_types[$i] = query_type;
                                    }
                                    _ => unreachable!(),
                                };
                            }
                        )+
                        self.chunk = Some((archetype, chunk_item, query_types, 0));
                    }
                }

                let (archetype, chunk_item, query_types, index) = self.chunk.as_mut()?;

                unsafe {
                    Some((
                        $(
                            {
                                let component: *mut Box<dyn Any> = match &query_types[$i] {
                                    QueryType::With((_, position)) => {
                                        &mut chunk_item.components[*index * archetype.components_len() + *position]
                                            .component
                                    }
                                    QueryType::WithShared(key) => {
                                        &mut self.shared_components.get_mut(key).unwrap().component
                                    }
                                    _ => unreachable!(),
                                };

                                (*component).downcast_mut::<$c>().unwrap()
                            },
                        )+
                    ))
                }
            }
        }
    };
}

// Repeats query_simple for tuple simple query operators 16 times making it supports queries maximum 16 components at once.

query_simple! {
    QuerySimple1,
    QuerySimpleIter1,
    1,
    (0, A, S1),
}
query_simple! {
    QuerySimple2,
    QuerySimpleIter2,
    2,
    (0, A, S1),
    (1, B, S2),
}
query_simple! {
    QuerySimple3,
    QuerySimpleIter3,
    3,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
}
query_simple! {
    QuerySimple4,
    QuerySimpleIter4,
    4,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
}
query_simple! {
    QuerySimple5,
    QuerySimpleIter5,
    5,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
}
query_simple! {
    QuerySimple6,
    QuerySimpleIter6,
    6,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
}
query_simple! {
    QuerySimple7,
    QuerySimpleIter7,
    7,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
}
query_simple! {
    QuerySimple8,
    QuerySimpleIter8,
    8,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
}
query_simple! {
    QuerySimple9,
    QuerySimpleIter9,
    9,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
}
query_simple! {
    QuerySimple10,
    QuerySimpleIter10,
    10,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
}
query_simple! {
    QuerySimple11,
    QuerySimpleIter11,
    11,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
}
query_simple! {
    QuerySimple12,
    QuerySimpleIter12,
    12,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
    (11, L, S12),
}
query_simple! {
    QuerySimple13,
    QuerySimpleIter13,
    13,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
    (11, L, S12),
    (12, M, S13),
}
query_simple! {
    QuerySimple14,
    QuerySimpleIter14,
    14,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
    (11, L, S12),
    (12, M, S13),
    (13, N, S14),
}
query_simple! {
    QuerySimple15,
    QuerySimpleIter15,
    15,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
    (11, L, S12),
    (12, M, S13),
    (13, N, S14),
    (14, O, S15),
}
query_simple! {
    QuerySimple16,
    QuerySimpleIter16,
    16,
    (0, A, S1),
    (1, B, S2),
    (2, C, S3),
    (3, D, S4),
    (4, E, S5),
    (5, F, S6),
    (6, G, S7),
    (7, H, S8),
    (8, I, S9),
    (9, J, S10),
    (10, K, S11),
    (11, L, S12),
    (12, M, S13),
    (13, N, S14),
    (14, O, S15),
    (15, P, S16),
}

/// Queried components returns by [`QueryComplex`].
pub struct Queried<'a> {
    components: HashMap<ComponentKey, &'a mut Box<dyn Any>>,
    shared_components: HashMap<SharedComponentKey, &'a mut Box<dyn Any>>,
}

impl<'a> Queried<'a> {
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

pub struct QueryComplexIter<'a, S> {
    shared_components: &'a mut HashMap<SharedComponentKey, SharedComponentItem>,
    chunks: hashbrown::hash_map::IterMut<'a, Archetype, ChunkItem>,
    chunk: Option<(&'a Archetype, &'a mut ChunkItem, QueryType, usize)>,
    _k: PhantomData<S>,
}

impl<'a, S> QueryComplexIter<'a, S>
where
    S: QueryOp + 'static,
{
    fn new(manager: &'a mut EntityManager) -> Self {
        Self {
            shared_components: &mut manager.shared_components,
            chunks: manager.chunks.iter_mut(),
            chunk: None,
            _k: PhantomData,
        }
    }
}

impl<'a, S> Iterator for QueryComplexIter<'a, S>
    where
    S: QueryOp + 'static,
{
    type Item = Queried<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, chunk_item, _, index)) = self.chunk.as_mut() {
            *index += 1;

            if *index >= chunk_item.entity_keys.len() {
                self.chunk = None;
            }
        }

        if self.chunk.is_none() {
            // finds next chunk that matches the query
            while let Some((archetype, chunk_item)) = self.chunks.next() {
                let query_type = S::query(archetype);
                match &query_type {
                    QueryType::NotMatched => continue,
                    _ => {
                        self.chunk = Some((archetype, chunk_item, query_type, 0));
                    }
                }
            }
        }

        let (archetype, chunk_item, query_type, index) = self.chunk.as_mut()?;

        let mut components = HashMap::with_capacity(1);
        let mut shared_components = HashMap::with_capacity(1);
        match query_type {
            QueryType::With((key, position)) => {
                let component: *mut Box<dyn Any> = &mut chunk_item.components
                    [*index * archetype.components_len() + *position]
                    .component;
                unsafe {
                    components.insert_unique_unchecked(*key, &mut *component);
                }
            }
            QueryType::WithShared(key) => {
                let component: *mut Box<dyn Any> =
                    &mut self.shared_components.get_mut(key).unwrap().component;
                unsafe {
                    shared_components.insert_unique_unchecked(*key, &mut *component);
                }
            }
            QueryType::Without => {
                // does nothing
            }
            QueryType::NotMatched => unreachable!(),
        };

        Some(Queried {
            components,
            shared_components,
        })
    }
}

/// A complex query queries components using all kinds of query operators, returns [`Queried`] as result.
pub trait QueryComplex<'a, S> {
    fn query_complex(manager: &'a mut EntityManager) -> QueryComplexIter<'a, S>
    where
        Self: Sized;
}

impl<'a, S> QueryComplex<'a, S> for S
where
    S: QueryOp + 'static,
{
    fn query_complex(manager: &'a mut EntityManager) -> QueryComplexIter<'a, S>
    where
        Self: Sized,
    {
        QueryComplexIter::new(manager)
    }
}

/// A macro rule implements complex query for tuple query operators.
macro_rules! query_complex {
    (
        $query: tt,
        $iter: tt,
        $count: tt,
        $(
            ($i: tt, $q: tt),
        )+
    ) => {
        pub trait $query<'a, $($q,)+> {
            fn query_complex(manager: &'a mut EntityManager) -> $iter<'a, $($q,)+>
            where
                Self: Sized;
        }


        impl<'a, $($q,)+> $query<'a, $($q,)+> for ($($q,)+)
        where
            $(
                $q: QueryOp + 'static,
            )+
        {
            fn query_complex(manager: &'a mut EntityManager) -> $iter<'a, $($q,)+>
            where
                Self: Sized,
            {
                $iter::new(manager)
            }
        }

        pub struct $iter<'a, $($q,)+>{
            shared_components: &'a mut HashMap<SharedComponentKey, SharedComponentItem>,
            chunks: hashbrown::hash_map::IterMut<'a, Archetype, ChunkItem>,
            chunk: Option<(&'a Archetype, &'a mut ChunkItem, [QueryType; $count], usize)>,
            _k: PhantomData<($($q,)+)>,
        }

        impl<'a, $($q,)+> $iter<'a, $($q,)+> {
            fn new(manager: &'a mut EntityManager) -> Self {
                Self {
                    shared_components: &mut manager.shared_components,
                    chunks: manager.chunks.iter_mut(),
                    chunk: None,
                    _k: PhantomData,
                }
            }
        }

        impl<'a, $($q,)+> Iterator for $iter<'a, $($q,)+>
        where
            $(
                $q: QueryOp + 'static,
            )+
        {
            type Item = Queried<'a>;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some((_, chunk_item, _, index)) = self.chunk.as_mut() {
                    *index += 1;

                    if *index >= chunk_item.entity_keys.len() {
                        self.chunk = None;
                    }
                }

                if self.chunk.is_none() {
                    while let Some((archetype, chunk_item)) = self.chunks.next() {
                        let mut query_types = [QueryType::NotMatched; $count];
                        $(
                            {
                                let query_type = $q::query(archetype);
                                match &query_type {
                                    QueryType::NotMatched => continue,
                                    _ => {
                                        query_types[$i] = query_type;
                                    }
                                };
                            }
                        )+
                        self.chunk = Some((archetype, chunk_item, query_types, 0));
                    }
                }

                let (archetype, chunk_item, query_types, index) = self.chunk.as_mut()?;

                let mut components = HashMap::with_capacity(1);
                let mut shared_components = HashMap::with_capacity(1);
                $(
                    match &query_types[$i] {
                        QueryType::With((key, position)) => {
                            let component: *mut Box<dyn Any> = &mut chunk_item.components[*index * archetype.components_len() + *position].component;
                            unsafe {
                                components.insert_unique_unchecked(*key, &mut *component);
                            }
                        }
                        QueryType::WithShared(key) => {
                            let component: *mut Box<dyn Any> = &mut self.shared_components.get_mut(key).unwrap().component;
                            unsafe {
                                shared_components.insert_unique_unchecked(*key, &mut *component);
                            }
                        }
                        QueryType::Without => {
                            // does nothing
                        }
                        QueryType::NotMatched => unreachable!(),
                    };
                )+

                Some(Queried { components, shared_components, })
            }
        }
    };
}

// Repeats query_complex for tuple query operators 16 times making it supports queries maximum 16 components at once.


query_complex! {
    QueryComplex1,
    QueryComplexIter1,
    1,
    (0, S1),
}
query_complex! {
    QueryComplex2,
    QueryComplexIter2,
    2,
    (0, S1),
    (1, S2),
}
query_complex! {
    QueryComplex3,
    QueryComplexIter3,
    3,
    (0, S1),
    (1, S2),
    (2, S3),
}
query_complex! {
    QueryComplex4,
    QueryComplexIter4,
    4,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
}
query_complex! {
    QueryComplex5,
    QueryComplexIter5,
    5,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
}
query_complex! {
    QueryComplex6,
    QueryComplexIter6,
    6,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
}
query_complex! {
    QueryComplex7,
    QueryComplexIter7,
    7,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
}
query_complex! {
    QueryComplex8,
    QueryComplexIter8,
    8,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
}
query_complex! {
    QueryComplex9,
    QueryComplexIter9,
    9,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
}
query_complex! {
    QueryComplex10,
    QueryComplexIter10,
    10,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
}
query_complex! {
    QueryComplex11,
    QueryComplexIter11,
    11,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
}
query_complex! {
    QueryComplex12,
    QueryComplexIter12,
    12,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
    (11, S12),
}
query_complex! {
    QueryComplex13,
    QueryComplexIter13,
    13,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
    (11, S12),
    (12, S13),
}
query_complex! {
    QueryComplex14,
    QueryComplexIter14,
    14,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
    (11, S12),
    (12, S13),
    (13, S14),
}
query_complex! {
    QueryComplex15,
    QueryComplexIter15,
    15,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
    (11, S12),
    (12, S13),
    (13, S14),
    (14, S15),
}
query_complex! {
    QueryComplex16,
    QueryComplexIter16,
    16,
    (0, S1),
    (1, S2),
    (2, S3),
    (3, S4),
    (4, S5),
    (5, S6),
    (6, S7),
    (7, S8),
    (8, S9),
    (9, S10),
    (10, S11),
    (11, S12),
    (12, S13),
    (13, S14),
    (14, S15),
    (15, S16),
}

