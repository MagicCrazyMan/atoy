use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use smallvec::SmallVec;
use uuid::Uuid;

use super::{
    channel::{MessageChannel, Sender, Unregister},
    AsAny,
};

#[derive(Debug, Clone)]
pub enum Message {
    AddComponent {
        entity_id: Uuid,
        old_component_types: BTreeSet<TypeId>,
        new_component_types: BTreeSet<TypeId>,
    },
    RemoveComponent {
        entity_id: Uuid,
        old_component_types: BTreeSet<TypeId>,
        new_component_types: BTreeSet<TypeId>,
    },
    AddEntity {
        entity_id: Uuid,
    },
    RemoveEntity {
        entity_id: Uuid,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentTypes(SmallVec<[TypeId; 3]>);

impl ComponentTypes {
    pub fn new<I>(component_types: I) -> Self
    where
        I: IntoIterator<Item = TypeId>,
    {
        let mut component_types: SmallVec<[TypeId; 3]> = component_types.into_iter().collect();
        component_types.sort();
        component_types.dedup();
        Self(component_types)
    }
}

pub trait IntoComponentTypes {
    fn into_component_types() -> ComponentTypes;
}

// impl<I> From<I> for ComponentTypes
// where
//     I: IntoIterator<Item = TypeId>,
// {
//     fn from(value: I) -> Self {
//         Self::new(value)
//     }
// }

impl<A0> IntoComponentTypes for A0
where
    A0: Component + 'static,
{
    fn into_component_types() -> ComponentTypes {
        ComponentTypes::new([TypeId::of::<A0>()])
    }
}

macro_rules! into_component_types {
    ($($ct: tt),+) => {
        impl<$($ct,)+> IntoComponentTypes for ($($ct,)+)
        where
            $(
                $ct: Component + 'static,
            )+
        {
            fn into_component_types() -> ComponentTypes {
                ComponentTypes::new([
                    $(
                        TypeId::of::<$ct>(),
                    )+
                ])
            }
        }
    };
}

macro_rules! into_component_types_4 {
    ($(($($ct: tt),+))+) => {
        $(
            into_component_types!($($ct),+);
        )+
    };
}

into_component_types_4! {
    (A0)
    (A0, A1)
    (A0, A1, A2)
    (A0, A1, A2, A3)
}

fn a() {
    struct A {
        a: usize
    }
    impl Component for A {}

    struct B;
    impl Component for B {}

    <(A, B)>::into_component_types();
}

pub trait Component {
    #[inline]
    fn component_type() -> TypeId
    where
        Self: Sized + 'static,
    {
        TypeId::of::<Self>()
    }
}

pub struct Entity {
    id: Uuid,
    sender: Sender<Message>,
    components: HashMap<TypeId, Box<dyn Any>>,

    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, BTreeSet<TypeId>>>>,
}

impl Entity {
    fn new(manager: &EntityManager) -> Self {
        Self::with_components(manager, HashMap::new())
    }

    fn with_components(manager: &EntityManager, components: HashMap<TypeId, Box<dyn Any>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: manager.sender.clone(),
            components,

            archetypes: Rc::clone(&manager.archetypes),
            entities: Rc::clone(&manager.entities),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn component_types(&self) -> BTreeSet<TypeId> {
        self.components.keys().cloned().collect()
    }

    pub fn component_len(&self) -> usize {
        self.components.len()
    }

    pub fn component<T>(&self) -> Option<&T>
    where
        T: Component + 'static,
    {
        match self.components.get(&T::component_type()) {
            Some(component) => Some(component.downcast_ref::<T>().unwrap()),
            None => None,
        }
    }

    pub fn component_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Component + 'static,
    {
        match self.components.get_mut(&T::component_type()) {
            Some(component) => Some(component.downcast_mut::<T>().unwrap()),
            None => None,
        }
    }

    pub fn component_unchecked<T>(&self) -> &T
    where
        T: Component + 'static,
    {
        self.component::<T>().unwrap()
    }

    pub fn component_mut_unchecked<T>(&mut self) -> &mut T
    where
        T: Component + 'static,
    {
        self.component_mut::<T>().unwrap()
    }

    pub fn has_component<T>(&self) -> bool
    where
        T: Component + 'static,
    {
        self.components.contains_key(&T::component_type())
    }

    pub fn add_component<T>(&mut self, component: T) -> Result<(), T>
    where
        T: Component + 'static,
    {
        if self.components.contains_key(&T::component_type()) {
            return Err(component);
        };

        let old_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();
        self.components
            .insert_unique_unchecked(T::component_type(), Box::new(component));
        let new_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();

        self.swap_archetype(&old_component_types, &new_component_types);
        self.sender.send(Message::AddComponent {
            entity_id: self.id,
            old_component_types,
            new_component_types,
        });

        Ok(())
    }

    pub fn remove_component<T>(&mut self) -> Option<T>
    where
        T: Component + 'static,
    {
        if self.components.contains_key(&T::component_type()) {
            return None;
        };

        let old_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();
        let removed = *self
            .components
            .remove(&T::component_type())
            .unwrap()
            .downcast::<T>()
            .unwrap();
        let new_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();

        self.swap_archetype(&old_component_types, &new_component_types);
        self.sender.send(Message::RemoveComponent {
            entity_id: self.id,
            old_component_types,
            new_component_types,
        });

        Some(removed)
    }

    fn swap_archetype(
        &self,
        old_component_types: &BTreeSet<TypeId>,
        new_component_types: &BTreeSet<TypeId>,
    ) {
        if old_component_types == new_component_types {
            return;
        }

        let mut archetypes = self.archetypes.borrow_mut();
        let Some(entity) = archetypes
            .get_mut(old_component_types)
            .and_then(|archetype| archetype.remove_entity(&self.id))
        else {
            return;
        };

        match archetypes.entry(new_component_types.clone()) {
            Entry::Occupied(mut o) => {
                o.get_mut().add_entity_unchecked(entity);
            }
            Entry::Vacant(v) => {
                let mut archetype = Archetype::new(self.sender.clone());
                archetype.add_entity_unchecked(entity);
                v.insert(archetype);
            }
        };
        self.entities
            .borrow_mut()
            .insert(self.id, new_component_types.clone());
    }
}

struct Archetype {
    entities: HashMap<Uuid, Entity>,
    sender: Sender<Message>,
}

impl Archetype {
    fn new(sender: Sender<Message>) -> Self {
        Self {
            entities: HashMap::new(),
            sender,
        }
    }

    fn add_entity_unchecked(&mut self, entity: Entity) -> &mut Entity {
        let id = entity.id;
        let (_, entity) = self.entities.insert_unique_unchecked(id, entity);
        self.sender.send(Message::AddEntity { entity_id: id });
        entity
    }

    fn remove_entity(&mut self, id: &Uuid) -> Option<Entity> {
        let removed = self.entities.remove(id);
        if let Some(removed) = &removed {
            self.sender.send(Message::RemoveEntity {
                entity_id: removed.id,
            });
        }
        removed
    }
}

const EMPTY_ARCHETYPE: BTreeSet<TypeId> = BTreeSet::new();

pub struct EntityManager {
    id: Uuid,

    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, BTreeSet<TypeId>>>>,

    sender: Sender<Message>,
    unregistered: Option<Unregister<Message>>,
}

impl Drop for EntityManager {
    fn drop(&mut self) {
        self.unregistered.take().map(|unregister| {
            unregister.unregister();
        });
    }
}

impl EntityManager {
    pub fn new(channel: MessageChannel) -> Self {
        Self {
            id: Uuid::new_v4(),

            archetypes: Rc::new(RefCell::new(HashMap::from([(
                EMPTY_ARCHETYPE,
                Archetype::new(channel.sender()),
            )]))),
            entities: Rc::new(RefCell::new(HashMap::new())),

            sender: channel.sender(),
            unregistered: None,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn create_empty_entity(&self) -> RefMut<'_, Entity> {
        let entity = Entity::new(self);
        self.entities
            .borrow_mut()
            .insert_unique_unchecked(entity.id, EMPTY_ARCHETYPE.clone());
        RefMut::map(self.archetypes.borrow_mut(), |archetypes| {
            archetypes
                .get_mut(&EMPTY_ARCHETYPE)
                .unwrap()
                .add_entity_unchecked(entity)
        })
    }

    pub fn create_entity<I>(&self, components: I) -> RefMut<'_, Entity>
    where
        I: IntoIterator<Item = (TypeId, Box<dyn Any>)>,
    {
        let (component_types, components) = components.into_iter().fold(
            (BTreeSet::new(), HashMap::new()),
            |(mut component_types, mut components), (component_type, component)| {
                component_types.insert(component_type);
                components.insert(component_type, component);
                (component_types, components)
            },
        );

        let entity = Entity::with_components(self, components);
        self.entities
            .borrow_mut()
            .insert_unique_unchecked(entity.id, component_types.clone());
        let entity = RefMut::map(self.archetype_or_create(component_types), |archetype| {
            archetype.add_entity_unchecked(entity)
        });
        entity
    }

    pub fn remove_entity(&mut self, entity_id: &Uuid) {
        let Some(archetype) = self.entities.borrow_mut().remove(entity_id) else {
            return;
        };
        self.archetypes
            .borrow_mut()
            .get_mut(&archetype)
            .unwrap()
            .remove_entity(entity_id);
    }

    pub fn entities(&self) -> OverallIter {
        OverallIter::new(self)
    }

    pub fn entities_mut(&self) -> OverallIterMut {
        OverallIterMut::new(self)
    }

    fn archetype_or_create(&self, component_types: BTreeSet<TypeId>) -> RefMut<'_, Archetype> {
        RefMut::map(self.archetypes.borrow_mut(), |archetypes| match archetypes
            .entry(component_types)
        {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Archetype::new(self.sender.clone())),
        })
    }
}

impl EntityManager {
    // pub fn create_entity_4<A, B, C, D>(&self, a: A, b: B, c: C, d: D) -> RefMut<'_, Entity>
    // where
    //     A: Component + 'static,
    //     B: Component + 'static,
    //     C: Component + 'static,
    //     D: Component + 'static,
    // {
    //     self.create_entity([
    //         (TypeId::of::<A>(), Box::new(a) as Box<dyn Any>),
    //         (TypeId::of::<B>(), Box::new(b) as Box<dyn Any>),
    //         (TypeId::of::<C>(), Box::new(c) as Box<dyn Any>),
    //         (TypeId::of::<D>(), Box::new(d) as Box<dyn Any>),
    //     ])
    // }

    // pub fn entities_in_archetype_4<A, B, C, D>(&self) -> Iter<'_>
    // where
    //     A: Component + 'static,
    //     B: Component + 'static,
    //     C: Component + 'static,
    //     D: Component + 'static,
    // {
    //     Iter::new(
    //         self,
    //         BTreeSet::from_iter([
    //             TypeId::of::<A>(),
    //             TypeId::of::<B>(),
    //             TypeId::of::<C>(),
    //             TypeId::of::<D>(),
    //         ]),
    //     )
    // }

    // pub fn entities_in_archetype_4_mut<A, B, C, D>(&self) -> IterMut<'_>
    // where
    //     A: Component + 'static,
    //     B: Component + 'static,
    //     C: Component + 'static,
    //     D: Component + 'static,
    // {
    //     IterMut::new(
    //         self,
    //         BTreeSet::from_iter([
    //             TypeId::of::<A>(),
    //             TypeId::of::<B>(),
    //             TypeId::of::<C>(),
    //             TypeId::of::<D>(),
    //         ]),
    //     )
    // }

    // pub fn query_4_mut<A, B, C, D>(&self) -> Query<'_, A, B, C, D>
    // where
    //     A: Component + 'static,
    //     B: Component + 'static,
    //     C: Component + 'static,
    //     D: Component + 'static,
    // {
    //     Query::<A, B, C, D>::new(self)
    // }
}

pub struct Iter<'a> {
    component_types: BTreeSet<TypeId>,
    archetype: *mut Option<Ref<'a, Archetype>>,
    entities: Option<hashbrown::hash_map::Iter<'a, Uuid, Entity>>,
}

impl<'a> Drop for Iter<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetype);
        }
    }
}

impl<'a> Iter<'a> {
    fn new(manager: &'a EntityManager, component_types: BTreeSet<TypeId>) -> Self {
        let archetype = Ref::filter_map(manager.archetypes.borrow(), |archetypes| {
            archetypes.get(&component_types)
        })
        .ok();
        let archetype = Box::into_raw(Box::new(archetype));
        let entities = unsafe {
            match (*archetype).as_ref() {
                Some(archetype) => Some(archetype.entities.iter()),
                None => None,
            }
        };

        Self {
            component_types,
            archetype,
            entities,
        }
    }

    pub fn component_types(&self) -> &BTreeSet<TypeId> {
        &self.component_types
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.as_mut()?.next().map(|(_, entity)| entity)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.entities.as_ref() {
            Some(entities) => entities.size_hint(),
            None => (0, Some(0)),
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        match self.entities.as_ref() {
            Some(entities) => entities.len(),
            None => 0,
        }
    }
}

pub struct IterMut<'a> {
    component_types: BTreeSet<TypeId>,
    archetype: *mut Option<RefMut<'a, Archetype>>,
    entities: Option<hashbrown::hash_map::IterMut<'a, Uuid, Entity>>,
}

impl<'a> Drop for IterMut<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetype);
        }
    }
}

impl<'a> IterMut<'a> {
    fn new(manager: &'a EntityManager, component_types: BTreeSet<TypeId>) -> Self {
        let archetype = RefMut::filter_map(manager.archetypes.borrow_mut(), |archetypes| {
            archetypes.get_mut(&component_types)
        })
        .ok();
        let archetype = Box::into_raw(Box::new(archetype));
        let entities = unsafe {
            match (*archetype).as_mut() {
                Some(archetype) => Some(archetype.entities.iter_mut()),
                None => None,
            }
        };

        Self {
            component_types,
            archetype,
            entities,
        }
    }

    pub fn component_types(&self) -> &BTreeSet<TypeId> {
        &self.component_types
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.as_mut()?.next().map(|(_, entity)| entity)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.entities.as_ref() {
            Some(entities) => entities.size_hint(),
            None => (0, Some(0)),
        }
    }
}

impl<'a> ExactSizeIterator for IterMut<'a> {
    fn len(&self) -> usize {
        match self.entities.as_ref() {
            Some(entities) => entities.len(),
            None => 0,
        }
    }
}

pub struct OverallIter<'a> {
    archetypes: *mut Ref<'a, HashMap<BTreeSet<TypeId>, Archetype>>,
    entities: Ref<'a, HashMap<Uuid, BTreeSet<TypeId>>>,
    entities_iter: Box<dyn Iterator<Item = &'a Entity> + 'a>,
}

impl<'a> Drop for OverallIter<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetypes);
        }
    }
}

impl<'a> OverallIter<'a> {
    fn new(manager: &'a EntityManager) -> Self {
        unsafe {
            let entities = manager.entities.borrow();
            let archetypes: *mut Ref<HashMap<BTreeSet<TypeId>, Archetype>> =
                Box::leak(Box::new(manager.archetypes.borrow()));
            let entities_iter = (*archetypes)
                .iter()
                .flat_map(|(_, archetype)| archetype.entities.iter().map(|(_, entity)| entity));
            Self {
                archetypes,
                entities,
                entities_iter: Box::new(entities_iter),
            }
        }
    }
}

impl<'a> Iterator for OverallIter<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.entities.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for OverallIter<'a> {
    fn len(&self) -> usize {
        self.entities.len()
    }
}

pub struct OverallIterMut<'a> {
    archetypes: *mut RefMut<'a, HashMap<BTreeSet<TypeId>, Archetype>>,
    entities: RefMut<'a, HashMap<Uuid, BTreeSet<TypeId>>>,
    entities_iter: Box<dyn Iterator<Item = &'a mut Entity> + 'a>,
}

impl<'a> Drop for OverallIterMut<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetypes);
        }
    }
}

impl<'a> OverallIterMut<'a> {
    fn new(manager: &'a EntityManager) -> Self {
        unsafe {
            let entities = manager.entities.borrow_mut();
            let archetypes: *mut RefMut<HashMap<BTreeSet<TypeId>, Archetype>> =
                Box::leak(Box::new(manager.archetypes.borrow_mut()));
            let entities_iter = (*archetypes)
                .iter_mut()
                .flat_map(|(_, archetype)| archetype.entities.iter_mut().map(|(_, entity)| entity));
            Self {
                archetypes,
                entities,
                entities_iter: Box::new(entities_iter),
            }
        }
    }
}

impl<'a> Iterator for OverallIterMut<'a> {
    type Item = &'a mut Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.entities.len();
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for OverallIterMut<'a> {
    fn len(&self) -> usize {
        self.entities.len()
    }
}

pub struct Query<'a, A, B, C, D> {
    _component_types: PhantomData<(A, B, C, D)>,
    iter_mut: IterMut<'a>,
}

impl<'a, A, B, C, D> Query<'a, A, B, C, D>
where
    A: Component + 'static,
    B: Component + 'static,
    C: Component + 'static,
    D: Component + 'static,
{
    fn new(manager: &'a EntityManager) -> Self {
        let component_types = BTreeSet::from_iter([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ]);

        Self {
            _component_types: PhantomData,
            iter_mut: IterMut::new(manager, component_types),
        }
    }

    pub fn component_types(&self) -> &BTreeSet<TypeId> {
        &self.iter_mut.component_types
    }
}

impl<'a, A, B, C, D> Iterator for Query<'a, A, B, C, D> {
    type Item = &'a mut Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_mut.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter_mut.size_hint()
    }
}

impl<'a, A, B, C, D> ExactSizeIterator for Query<'a, A, B, C, D> {
    fn len(&self) -> usize {
        self.iter_mut.len()
    }
}

pub trait System<A, B, C, D> {
    fn execute(&mut self, query: Query<'_, A, B, C, D>);
}

// #[test]
// fn a() {
//     struct A {}

//     impl Component for A {}

//     struct B {}

//     impl Component for B {}

//     println!("{:?}", TypeId::of::<A>());
//     println!("{:?}", A {}.component_type());
//     println!("{:?}", Box::new(A {}).component_type());
// }
