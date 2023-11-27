// use wasm_bindgen::JsValue;
// use web_sys::HtmlCanvasElement;

// use crate::{
//     camera::Camera,
//     entity::Entity,
//     render::webgl::{draw::CullFace, error::Error},
//     scene::Scene,
// };

// use super::{
//     policy::{GeometryPolicy, MaterialPolicy},
//     postprocess::PostprocessOp,
//     preprocess::{InternalPreprocessOp, PreprocessOp},
//     RenderPipeline, RenderState, RenderStuff,
// };

// struct StandardRenderStuff<'s> {
//     scene: &'s mut Scene,
// }

// impl<'s> RenderStuff for StandardRenderStuff<'s> {
//     fn canvas(&self) -> &HtmlCanvasElement {
//         self.scene.canvas()
//     }

//     fn ctx_options(&self) -> Option<&JsValue> {
//         None
//     }

//     fn entities(&mut self) -> &mut [Entity] {
//         self.scene.root_entity_mut().children_mut()
//     }

//     fn camera(&mut self) -> &mut dyn Camera {
//         self.scene.active_camera_mut()
//     }
// }

// pub struct StandardPipeline;

// impl<'s> RenderPipeline<StandardRenderStuff<'s>> for StandardPipeline {
//     fn dependencies(&mut self) -> Result<(), Error> {
//         todo!()
//     }

//     fn prepare(&mut self, _: &mut StandardRenderStuff<'s>) -> Result<(), Error> {
//         Ok(())
//     }

//     fn pre_process(
//         &mut self,
//         _: &mut RenderState<StandardRenderStuff<'s>>,
//     ) -> Result<&[&dyn PreprocessOp<StandardRenderStuff<'s>>], Error> {
//         Ok(&[
//             &InternalPreprocessOp::UpdateViewport,
//             &InternalPreprocessOp::EnableDepthTest,
//             &InternalPreprocessOp::EnableCullFace,
//             &InternalPreprocessOp::EnableBlend,
//             &InternalPreprocessOp::ClearColor(0.0, 0.0, 0.0, 0.0),
//             &InternalPreprocessOp::ClearDepth(0.0),
//             &InternalPreprocessOp::SetCullFaceMode(CullFace::Back),
//         ])
//     }

//     fn material_policy(
//         &mut self,
//         _: &mut RenderState<StandardRenderStuff<'s>>,
//     ) -> Result<MaterialPolicy<StandardRenderStuff<'s>>, Error> {
//         Ok(MaterialPolicy::FollowEntity)
//     }

//     fn geometry_policy(
//         &mut self,
//         _: &mut RenderState<StandardRenderStuff<'s>>,
//     ) -> Result<GeometryPolicy<StandardRenderStuff<'s>>, Error> {
//         Ok(GeometryPolicy::FollowEntity)
//     }

//     fn post_precess(
//         &mut self,
//         _: &mut RenderState<StandardRenderStuff<'s>>,
//     ) -> Result<&[&dyn PostprocessOp<StandardRenderStuff<'s>>], Error> {
//         Ok(&[])
//     }
// }
