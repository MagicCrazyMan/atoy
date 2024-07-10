// use std::{cell::Ref, rc::Rc};

// use hashbrown::HashMap;
// use uuid::Uuid;

// use crate::core::Rrc;

// use super::{archetype::Archetype, entity::Entity, manager::EntityManager};

// pub struct Iter<'a> {
//     entities: *mut Ref<'a, HashMap<Uuid, Rrc<Entity>>>,
//     entities_iter: hashbrown::hash_map::Iter<'a, Uuid, Rrc<Entity>>,
// }

// impl<'a> Drop for Iter<'a> {
//     fn drop(&mut self) {
//         unsafe {
//             let _ = Box::from_raw(self.entities);
//         }
//     }
// }

// impl<'a> Iter<'a> {
//     pub(super) fn new(manager: &'a EntityManager) -> Self {
//         unsafe {
//             let entities = manager.entities.borrow();
//             let entities: *mut Ref<HashMap<Uuid, Rrc<Entity>>> = Box::leak(Box::new(entities));
//             let entities_iter = (*entities).iter();

//             Self {
//                 entities,
//                 entities_iter,
//             }
//         }
//     }
// }

// impl<'a> Iterator for Iter<'a> {
//     type Item = Rrc<Entity>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.entities_iter
//             .next()
//             .map(|(_, entity)| Rc::clone(entity))
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.entities_iter.size_hint()
//     }
// }

// impl<'a> ExactSizeIterator for Iter<'a> {
//     fn len(&self) -> usize {
//         self.entities_iter.len()
//     }
// }

// pub struct ArchetypeIter<'a> {
//     archetype: Archetype,
//     entities: *mut Ref<'a, HashMap<Uuid, Rrc<Entity>>>,
//     entities_iter: hashbrown::hash_map::Iter<'a, Uuid, Rrc<Entity>>,
// }

// impl<'a> Drop for ArchetypeIter<'a> {
//     fn drop(&mut self) {
//         unsafe {
//             let _ = Box::from_raw(self.entities);
//         }
//     }
// }

// impl<'a> ArchetypeIter<'a> {
//     pub(super) fn new(manager: &'a EntityManager, archetype: Archetype) -> Option<Self> {
//         unsafe {
//             let chunk =
//                 Ref::filter_map(manager.archetypes.borrow(), |archetypes| archetypes.get(&archetype)).ok()?;
//             let entities = Box::into_raw(Box::new(chunk));
//             let entities_iter = (*entities).iter();

//             Some(Self {
//                 archetype,
//                 entities,
//                 entities_iter,
//             })
//         }
//     }

//     pub fn archetype(&self) -> &Archetype {
//         &self.archetype
//     }
// }

// impl<'a> Iterator for ArchetypeIter<'a> {
//     type Item = Rrc<Entity>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.entities_iter.next().map(|(_, entity)| entity.clone())
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.entities_iter.size_hint()
//     }
// }

// impl<'a> ExactSizeIterator for ArchetypeIter<'a> {
//     fn len(&self) -> usize {
//         self.entities_iter.len()
//     }
// }
