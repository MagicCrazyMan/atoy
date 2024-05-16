use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    marker::PhantomData,
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::core::channel::Receiver;

use super::channel::{MessageChannel, Registry, Sender, Unregister};

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
    AddEntity(Uuid),
    RemoveEntity(Uuid),
}

pub trait Component {}

#[derive(Clone)]
pub struct Entity {
    id: Uuid,
    sender: Sender<Message>,
    components: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,

    workload: ArchetypesWorkload,
}

// impl Entity {
//     pub fn new<A, B, C, D>(a: A, b: B, c: C, d: D) -> Self
//     where
//         A: Component + 'static,
//         B: Component + 'static,
//         C: Component + 'static,
//         D: Component + 'static,
//     {
//         Self {
//             id: Uuid::new_v4(),
//             components: HashMap::from_iter([
//                 (TypeId::of::<A>(), Box::new(a) as Box<dyn Any>),
//                 (TypeId::of::<B>(), Box::new(b) as Box<dyn Any>),
//                 (TypeId::of::<C>(), Box::new(c) as Box<dyn Any>),
//                 (TypeId::of::<D>(), Box::new(d) as Box<dyn Any>),
//             ]),
//         }
//     }
// }

impl Entity {
    fn new(archetypes: &Archetypes) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: archetypes.sender.clone(),
            components: Rc::new(RefCell::new(HashMap::new())),

            workload: ArchetypesWorkload::new(archetypes),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn component_types(&self) -> BTreeSet<TypeId> {
        self.components.borrow().keys().cloned().collect()
    }

    pub fn component_len(&self) -> usize {
        self.components.borrow().len()
    }

    pub fn component<T>(&self) -> Option<Ref<'_, T>>
    where
        T: Component + 'static,
    {
        let component = Ref::filter_map(self.components.borrow(), |components| {
            components.get(&TypeId::of::<T>())
        });
        match component {
            Ok(component) => Some(Ref::map(component, |component| {
                component.downcast_ref::<T>().unwrap()
            })),
            Err(_) => None,
        }
    }

    pub fn component_mut<T>(&mut self) -> Option<RefMut<'_, T>>
    where
        T: Component + 'static,
    {
        let component = RefMut::filter_map(self.components.borrow_mut(), |components| {
            components.get_mut(&TypeId::of::<T>())
        });
        match component {
            Ok(component) => Some(RefMut::map(component, |component| {
                component.downcast_mut::<T>().unwrap()
            })),
            Err(_) => None,
        }
    }

    pub fn has_component<T>(&self) -> bool
    where
        T: Component + 'static,
    {
        self.components.borrow().contains_key(&TypeId::of::<T>())
    }

    pub fn add_component<T>(&mut self, component: T) -> Result<(), T>
    where
        T: Component + 'static,
    {
        let mut components = self.components.borrow_mut();
        if components.contains_key(&TypeId::of::<T>()) {
            return Err(component);
        };

        let old_component_types = components.keys().cloned().collect::<BTreeSet<_>>();
        components.insert_unique_unchecked(TypeId::of::<T>(), Box::new(component));
        let new_component_types = components.keys().cloned().collect::<BTreeSet<_>>();

        self.workload
            .swap_archetype(&self.id, &old_component_types, &new_component_types);
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
        let mut components = self.components.borrow_mut();
        if components.contains_key(&TypeId::of::<T>()) {
            return None;
        };

        let old_component_types = components.keys().cloned().collect::<BTreeSet<_>>();
        let removed = *components
            .remove(&TypeId::of::<T>())
            .unwrap()
            .downcast::<T>()
            .unwrap();
        let new_component_types = components.keys().cloned().collect::<BTreeSet<_>>();

        self.workload
            .swap_archetype(&self.id, &old_component_types, &new_component_types);
        self.sender.send(Message::RemoveComponent {
            entity_id: self.id,
            old_component_types,
            new_component_types,
        });

        Some(removed)
    }
}

pub trait System<A, B, C, D> {
    // fn event() ->

    fn query() -> Query<A, B, C, D>
    where
        Self: Sized;

    fn run(&mut self);
}

pub enum Query<A, B, C, D> {
    All(
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
        PhantomData<D>,
    ),
    Any(
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
        PhantomData<D>,
    ),
    Exclude(
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
        PhantomData<D>,
    ),
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

    fn add_entity_unchecked(&mut self, entity: Entity) {
        let id = entity.id;
        self.entities.insert_unique_unchecked(id, entity);
        self.sender.send(Message::AddEntity(id));
    }

    fn remove_entity(&mut self, id: &Uuid) -> Option<Entity> {
        let removed = self.entities.remove(id);
        if let Some(removed) = &removed {
            self.sender.send(Message::RemoveEntity(removed.id));
        }
        removed
    }
}

// impl Archetype {
//     fn new<A, B, C, D>(sender: Sender<Message>) -> Self
//     where
//         A: Component + 'static,
//         B: Component + 'static,
//         C: Component + 'static,
//         D: Component + 'static,
//     {
//         // channel.registry().register(receiver)
//         Self {
//             component_types: HashSet::from_iter([
//                 TypeId::of::<A>(),
//                 TypeId::of::<B>(),
//                 TypeId::of::<C>(),
//                 TypeId::of::<D>(),
//             ]),
//             entities: HashMap::new(),
//         }
//     }
// }

pub struct Archetypes {
    id: Uuid,

    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, Entity>>>,

    registry: Registry<Message>,
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

            archetypes: Rc::new(RefCell::new(HashMap::new())),
            entities: Rc::new(RefCell::new(HashMap::new())),

            registry: channel.registry(),
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

    fn create_archetype<I>(&mut self, component_types: I) -> bool
    where
        I: IntoIterator<Item = TypeId>,
    {
        let component_types = component_types.into_iter().collect::<BTreeSet<_>>();
        match self.archetypes.borrow_mut().entry(component_types) {
            Entry::Occupied(_) => false,
            Entry::Vacant(v) => {
                v.insert(Archetype::new(self.sender.clone()));
                true
            }
        }
    }
}

impl Archetypes {
    pub fn entities(&self) -> Ref<'_, HashMap<Uuid, Entity>> {
        self.entities.borrow()
    }

    pub fn entities_mut(&self) -> RefMut<'_, HashMap<Uuid, Entity>> {
        self.entities.borrow_mut()
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

    pub fn create_archetype_4<A, B, C, D>(&mut self) -> bool
    where
        A: Component + 'static,
        B: Component + 'static,
        C: Component + 'static,
        D: Component + 'static,
    {
        self.create_archetype([
            TypeId::of::<A>(),
            TypeId::of::<B>(),
            TypeId::of::<C>(),
            TypeId::of::<D>(),
        ])
    }
}

#[derive(Clone)]
struct ArchetypesWorkload {
    archetypes: Rc<RefCell<HashMap<BTreeSet<TypeId>, Archetype>>>,
    entities: Rc<RefCell<HashMap<Uuid, Entity>>>,

    sender: Sender<Message>,
}

impl ArchetypesWorkload {
    fn new(archetypes: &Archetypes) -> Self {
        Self {
            archetypes: Rc::clone(&archetypes.archetypes),
            entities: Rc::clone(&archetypes.entities),

            sender: archetypes.sender.clone(),
        }
    }

    fn swap_archetype(
        &self,
        entity_id: &Uuid,
        old_component_types: &BTreeSet<TypeId>,
        new_component_types: &BTreeSet<TypeId>,
    ) {
        if old_component_types == new_component_types {
            return;
        }

        let mut archetypes = self.archetypes.borrow_mut();
        let Some(entity) = archetypes
            .get_mut(old_component_types)
            .and_then(|archetype| archetype.remove_entity(entity_id))
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
    }
}

// impl Receiver<Message> for ArchetypesWorkload {
//     fn receive(&self, message: &Message) {
//         match message {
//             Message::AddComponent {
//                 entity_id,
//                 old_component_types,
//                 new_component_types,
//             }
//             | Message::RemoveComponent {
//                 entity_id,
//                 old_component_types,
//                 new_component_types,
//             } => self.swap_archetype(entity_id, old_component_types, new_component_types),
//             Message::AddEntity(_) => todo!(),
//             Message::RemoveEntity(_) => todo!(),
//         }
//     }
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
