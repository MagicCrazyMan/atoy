// use std::{
//     any::{Any, TypeId},
//     cell::RefCell,
//     rc::Rc,
// };

// use gl_matrix4rust::mat4::Mat4;
// use hashbrown::HashMap;
// use proc::AsAny;
// use uuid::Uuid;

// use crate::core::{
//     engine::RenderEngine,
//     entity::{self, Entity, EntityBoundingVolume, EntityMatrices, EntityShared, EntitySharedWeak},
//     frustum::ViewFrustum,
//     scene::{self, Scene},
//     transparency::Transparency,
//     versioning::Versioning,
//     web::webgl::entity::WebGlEntity,
// };

// use super::Operator;

// #[derive(Debug)]
// pub struct EntityCollector {
//     use_view_frustum_culling: bool,
//     use_distance_sorting: bool,
//     use_entity_filtering: bool,
//     use_entity_transparency: bool,
//     use_cache: bool,
//     cache: Option<Cache>,
// }

// impl EntityCollector {
//     pub fn new() -> Self {
//         Self {
//             use_view_frustum_culling: true,
//             use_distance_sorting: true,
//             use_entity_filtering: true,
//             use_entity_transparency: true,
//             use_cache: true,
//             cache: None,
//         }
//     }

//     fn collect(&self, scene: &Scene) -> CollectedEntities {
//         let (list, map) = scene.entity_collection().entities().into_iter().fold(
//             (Vec::new(), HashMap::new()),
//             |(list, map), entity| {
//                 let entity = entity.borrow();

//                 if let (true, Some(entity)) = (self.use_entity_filtering, entity.as_filtering()) {
//                     if !entity.pass() {
//                         return (list, map);
//                     }
//                 };

//                 if let (true, Some(entity)) =
//                     (self.use_entity_transparency, entity.as_transparency())
//                 {
//                     if entity.transparency() == Transparency::Transparent {
//                         return (list, map);
//                     }
//                 };

//                 todo!()
//             },
//         );

//         CollectedEntities {
//             entities: list,
//             entities_map: map,
//         }
//     }
// }

// impl<RE> Operator<RE> for EntityCollector
// where
//     RE: RenderEngine,
// {
//     type Output = CollectedEntities;

//     fn execute(&mut self, scene: &mut Scene, _: &mut RE) -> Self::Output {
//         if self.use_cache {
//             if let Some(cache) = self.cache.as_ref() {
//                 if cache.update_to_date(scene) {
//                     return cache.collected.clone();
//                 }
//             }

//             let collected = self.collect(scene);
//             self.cache = Some(Cache {
//                 last_collection_id: scene.entity_collection().id().clone(),
//                 last_view_frustum: scene.camera().view_frustum().clone(),
//                 collected: collected.clone(),
//                 last_collection_version: todo!(),
//             });
//             collected
//         } else {
//             self.cache.take();
//             self.collect(scene)
//         }
//     }
// }

pub trait EntityFiltering {
    fn pass(&self) -> bool;
}

// #[derive(Debug, Clone)]
// pub struct CollectedEntity {
//     last_version: usize,
//     entity: EntitySharedWeak,
//     model_matrix: Rc<Mat4<f64>>,
// }

// impl CollectedEntity {
//     pub fn entity(&self) -> &EntitySharedWeak {
//         &self.entity
//     }

//     pub fn model_matrix(&self) -> &Mat4<f64> {
//         &self.model_matrix
//     }
// }

// #[derive(Debug, Clone)]
// pub struct CollectedEntities {
//     entities: Vec<CollectedEntity>,
//     entities_map: HashMap<Uuid, CollectedEntity>,
// }

// impl CollectedEntities {
//     pub fn entities(&self) -> &Vec<CollectedEntity> {
//         &self.entities
//     }

//     pub fn entities_map(&self) -> &HashMap<Uuid, CollectedEntity> {
//         &self.entities_map
//     }
// }

// #[derive(Debug)]
// struct Cache {
//     last_collection_id: Uuid,
//     last_collection_version: usize,
//     last_view_frustum: ViewFrustum,

//     collected: CollectedEntities,
// }

// impl Cache {
//     fn update_to_date(&self, scene: &Scene) -> bool {
//         &self.last_collection_id == scene.entity_collection().id()
//             && self.last_collection_version == scene.entity_collection().version()
//             && &self.last_view_frustum == scene.camera().view_frustum()
//     }
// }

// // #[test]
// // fn a() {
// //     #[derive(Debug, AsAny)]
// //     struct A {}

// //     impl EntityMatrices for A {
// //         fn local_matrix(&self) -> &gl_matrix4rust::mat4::Mat4<f64> {
// //             const TEST: gl_matrix4rust::mat4::Mat4<f64> =
// //                 gl_matrix4rust::mat4::Mat4::<f64>::new_identity();
// //             &TEST
// //         }

// //         fn model_matrix(&self) -> &gl_matrix4rust::mat4::Mat4<f64> {
// //             todo!()
// //         }

// //         fn set_model_matrix(&mut self, mat: gl_matrix4rust::mat4::Mat4<f64>) {
// //             todo!()
// //         }
// //     }

// //     impl EntityBoundingVolume for A {
// //         fn bounding_volume(&self) -> &crate::core::bounding::BoundingVolume {
// //             todo!()
// //         }
// //     }

// //     impl EntityFiltering for A {
// //         fn pass(&self) -> bool {
// //             println!("111111");
// //             true
// //         }
// //     }

// //     impl WebGlEntity for A {
// //         fn draw_mode(&self) -> crate::core::web::webgl::draw::DrawMode {
// //             crate::core::web::webgl::draw::DrawMode::LineLoop
// //         }

// //         fn draw_range(&self) -> std::ops::Range<usize> {
// //             todo!()
// //         }

// //         fn tick(&mut self, ticking: &crate::core::clock::Tick) {
// //             todo!()
// //         }

// //         fn pre_render(&mut self, context: &crate::core::web::webgl::context::Context) {
// //             todo!()
// //         }

// //         fn post_render(&mut self, context: &crate::core::web::webgl::context::Context) {
// //             todo!()
// //         }

// //         fn as_indexed_entity(
// //             &self,
// //         ) -> Option<&dyn crate::core::web::webgl::entity::WebGlIndexedEntity> {
// //             todo!()
// //         }

// //         fn as_instanced_entity(
// //             &self,
// //         ) -> Option<&dyn crate::core::web::webgl::entity::WebGlInstancedEntity> {
// //             todo!()
// //         }
// //     }

// //     impl Entity for A {
// //         fn as_filtering(&self) -> Option<&dyn EntityFiltering> {
// //             Some(self)
// //         }

// //         fn as_transparency(&self) -> Option<&dyn crate::core::transparency::EntityTransparency> {
// //             None
// //         }

// //         fn as_webgl(&self) -> Option<&dyn WebGlEntity> {
// //             Some(self)
// //         }

// //         fn as_webgl_mut(&mut self) -> Option<&mut dyn WebGlEntity> {
// //             Some(self)
// //         }
// //     }

// //     let a = A {};
// //     let a: Box<dyn Any> = Box::new(a);
// //     a.downcast_ref::<A>().unwrap().pass();

// //     // println!("{:?}", a.type_id());
// //     // let a: Rc<Box<dyn Any>> = Rc::new(Box::new(a));
// //     // println!("{:?}", a.type_id());
// //     // let b = a.as_ref();
// //     // println!("{:?}", b.type_id());
// //     // let c = &**a;
// //     // println!("{:?}", c.type_id());
// //     // // c.downcast_ref::<A>().unwrap().filter();
// //     // c.downcast_ref::<&dyn EntityFiltering>().unwrap().filter();
// // }
