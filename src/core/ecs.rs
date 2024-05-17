use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use super::channel::{MessageChannel, Sender, Unregister};

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

pub trait Component {}

pub struct Entity {
    id: Uuid,
    sender: Sender<Message>,
    components: HashMap<TypeId, Box<dyn Any>>,

    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, BTreeSet<TypeId>>>>,
}

impl Entity {
    fn with_components(archetypes: &Archetypes, components: HashMap<TypeId, Box<dyn Any>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: archetypes.sender.clone(),
            components,

            archetypes: Rc::clone(&archetypes.archetypes),
            entities: Rc::clone(&archetypes.entities),
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
        match self.components.get(&TypeId::of::<T>()) {
            Some(component) => Some(component.downcast_ref::<T>().unwrap()),
            None => None,
        }
    }

    pub fn component_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Component + 'static,
    {
        match self.components.get_mut(&TypeId::of::<T>()) {
            Some(component) => Some(component.downcast_mut::<T>().unwrap()),
            None => None,
        }
    }

    pub fn has_component<T>(&self) -> bool
    where
        T: Component + 'static,
    {
        self.components.contains_key(&TypeId::of::<T>())
    }

    pub fn add_component<T>(&mut self, component: T) -> Result<(), T>
    where
        T: Component + 'static,
    {
        if self.components.contains_key(&TypeId::of::<T>()) {
            return Err(component);
        };

        let old_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();
        self.components
            .insert_unique_unchecked(TypeId::of::<T>(), Box::new(component));
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
        if self.components.contains_key(&TypeId::of::<T>()) {
            return None;
        };

        let old_component_types = self.components.keys().cloned().collect::<BTreeSet<_>>();
        let removed = *self
            .components
            .remove(&TypeId::of::<T>())
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

pub struct Archetypes {
    id: Uuid,

    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, BTreeSet<TypeId>>>>,

    sender: Sender<Message>,
    unregistered: Option<Unregister<Message>>,
}

impl Drop for Archetypes {
    fn drop(&mut self) {
        self.unregistered.take().map(|unregister| {
            unregister.unregister();
        });
    }
}

impl Archetypes {
    pub fn new(channel: MessageChannel) -> Self {
        Self {
            id: Uuid::new_v4(),

            archetypes: Rc::new(RefCell::new(HashMap::from([(
                BTreeSet::new(),
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

    fn entities_in_archetype<I>(&self, component_types: I) -> Option<Ref<'_, HashMap<Uuid, Entity>>>
    where
        I: IntoIterator<Item = TypeId>,
    {
        let component_types = component_types.into_iter().collect::<BTreeSet<_>>();
        let archetype = Ref::filter_map(self.archetypes.borrow(), |archetypes| {
            archetypes.get(&component_types)
        })
        .ok()?;
        let entities = Ref::map(archetype, |archetype| &archetype.entities);
        Some(entities)
    }

    fn entities_in_archetype_mut<I>(
        &self,
        component_types: I,
    ) -> Option<RefMut<'_, HashMap<Uuid, Entity>>>
    where
        I: IntoIterator<Item = TypeId>,
    {
        let component_types = component_types.into_iter().collect::<BTreeSet<_>>();
        let archetype = RefMut::filter_map(self.archetypes.borrow_mut(), |archetypes| {
            archetypes.get_mut(&component_types)
        })
        .ok()?;
        let entities = RefMut::map(archetype, |archetype| &mut archetype.entities);
        Some(entities)
    }

    fn archetype_or_create(&self, component_types: BTreeSet<TypeId>) -> RefMut<'_, Archetype> {
        RefMut::map(self.archetypes.borrow_mut(), |archetypes| match archetypes
            .entry(component_types)
        {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Archetype::new(self.sender.clone())),
        })
    }

    fn create_entity<I>(&self, components: I) -> RefMut<'_, Entity>
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

    // fn remove_entity(&mut self, entity_id: &Uuid) -> HashMap<TypeId, Box<dyn Any>> {
    //     let Some(removed) = self.entities.borrow_mut().remove(entity_id) ;
    //     todo!()
    // }
}

impl Archetypes {
    pub fn create_empty_entity(&self) -> RefMut<'_, Entity> {
        self.create_entity([])
    }

    pub fn entities(&self) -> Iter {
        Iter::new(self)
    }

    pub fn entities_mut(&self) -> IterMut {
        IterMut::new(self)
    }

    pub fn create_entity_4<A, B, C, D>(&self, a: A, b: B, c: C, d: D) -> RefMut<'_, Entity>
    where
        A: Component + 'static,
        B: Component + 'static,
        C: Component + 'static,
        D: Component + 'static,
    {
        self.create_entity([
            (TypeId::of::<A>(), Box::new(a) as Box<dyn Any>),
            (TypeId::of::<B>(), Box::new(b) as Box<dyn Any>),
            (TypeId::of::<C>(), Box::new(c) as Box<dyn Any>),
            (TypeId::of::<D>(), Box::new(d) as Box<dyn Any>),
        ])
    }

    pub fn entities_in_archetype_4<A, B, C, D>(&self) -> Option<Ref<'_, HashMap<Uuid, Entity>>>
    where
        A: Component + 'static,
        B: Component + 'static,
        C: Component + 'static,
        D: Component + 'static,
    {
        self.entities_in_archetype([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ])
    }

    pub fn entities_in_archetype_4_mut<A, B, C, D>(
        &self,
    ) -> Option<RefMut<'_, HashMap<Uuid, Entity>>>
    where
        A: Component + 'static,
        B: Component + 'static,
        C: Component + 'static,
        D: Component + 'static,
    {
        self.entities_in_archetype_mut([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ])
    }
}

pub struct Iter<'a> {
    archetypes: *mut Ref<'a, HashMap<BTreeSet<TypeId>, Archetype>>,
    entities: Box<dyn Iterator<Item = &'a Entity> + 'a>,
}

impl<'a> Drop for Iter<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetypes);
        }
    }
}

impl<'a> Iter<'a> {
    fn new(archetypes: &'a Archetypes) -> Self {
        unsafe {
            let archetypes: *mut Ref<HashMap<BTreeSet<TypeId>, Archetype>> =
                Box::leak(Box::new(archetypes.archetypes.borrow()));
            let entities = (*archetypes)
                .iter()
                .flat_map(|(_, archetype)| archetype.entities.iter().map(|(_, entity)| entity));
            Self {
                archetypes,
                entities: Box::new(entities),
            }
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next()
    }
}

pub struct IterMut<'a> {
    archetypes: *mut RefMut<'a, HashMap<BTreeSet<TypeId>, Archetype>>,
    entities: Box<dyn Iterator<Item = &'a mut Entity> + 'a>,
}

impl<'a> Drop for IterMut<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetypes);
        }
    }
}

impl<'a> IterMut<'a> {
    fn new(archetypes: &'a Archetypes) -> Self {
        unsafe {
            let archetypes: *mut RefMut<HashMap<BTreeSet<TypeId>, Archetype>> =
                Box::leak(Box::new(archetypes.archetypes.borrow_mut()));
            let entities = (*archetypes)
                .iter_mut()
                .flat_map(|(_, archetype)| archetype.entities.iter_mut().map(|(_, entity)| entity));
            Self {
                archetypes,
                entities: Box::new(entities),
            }
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next()
    }
}

pub struct Query<'a, A, B, C, D> {
    _component_types: PhantomData<(A, B, C, D)>,
    archetype: *mut Option<RefMut<'a, Archetype>>,
    entities: Option<hashbrown::hash_map::IterMut<'a, Uuid, Entity>>,
}

impl<'a, A, B, C, D> Drop for Query<'a, A, B, C, D> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.archetype);
        }
    }
}

impl<'a, A, B, C, D> Query<'a, A, B, C, D>
where
    A: Component + 'static,
    B: Component + 'static,
    C: Component + 'static,
    D: Component + 'static,
{
    fn new(archetypes: &'a Archetypes) -> Self {
        let component_types = BTreeSet::from_iter([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ]);

        let archetype = RefMut::filter_map(archetypes.archetypes.borrow_mut(), |archetypes| {
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
            _component_types: PhantomData,
            archetype,
            entities,
        }
    }
}

impl<'a, A, B, C, D> Iterator for Query<'a, A, B, C, D>
where
    A: Component + 'static,
    B: Component + 'static,
    C: Component + 'static,
    D: Component + 'static,
{
    type Item = &'a mut Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.as_mut()?.next().map(|(_, entity)| entity)
    }
}

pub trait System<A, B, C, D> {
    fn execute(&mut self, query: Query<'_, A, B, C, D>);
}

// pub trait SystemAny<A, B, C, D> {
//     fn execute(&mut self, a: Option<A>, b: Option<B>, c: Option<C>, d: Option<D>);
// }

// #[test]
// fn a() {
//     struct A {}

//     impl Component for A {}

//     struct B {}

//     impl Component for B {}

//     let mut entity = Entity {
//         id: Uuid::new_v4(),
//         components: HashMap::new(),
//         sender: todo!(),
//     };

//     entity.add_component(A {});
//     assert_eq!(1, entity.component_len());
//     assert_eq!(true, entity.has_component::<A>());
//     assert_eq!(false, entity.has_component::<B>());

//     entity.remove_component::<A>();
//     assert_eq!(0, entity.component_len());
// }
