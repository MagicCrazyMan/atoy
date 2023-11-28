use crate::{
    camera::Camera,
    entity::EntityCollection,
    render::webgl::{draw::CullFace, error::Error},
    scene::Scene,
};

use super::{
    policy::{GeometryPolicy, MaterialPolicy},
    postprocess::PostprocessOp,
    preprocess::{InternalPreprocessOp, PreprocessOp},
    RenderPipeline, RenderState, RenderStuff,
};

struct StandardRenderStuff<'s> {
    scene: &'s mut Scene,
}

impl<'s> RenderStuff for StandardRenderStuff<'s> {
    fn camera(&self) -> &dyn Camera {
        self.scene.active_camera_mut()
    }

    fn camera_mut(&mut self) -> &mut dyn Camera {
        todo!()
    }

    fn entity_collection(&self) -> &EntityCollection {
        todo!()
    }

    fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        todo!()
    }
}

pub struct StandardPipeline;

impl<'s> RenderPipeline for StandardPipeline {
    fn dependencies(&self) -> Result<(), Error> {
        todo!()
    }

    fn prepare(&mut self, _: &mut dyn RenderStuff) -> Result<(), Error> {
        Ok(())
    }

    fn pre_process(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<&[&dyn PreprocessOp], Error> {
        Ok(&[
            &InternalPreprocessOp::UpdateViewport,
            &InternalPreprocessOp::EnableDepthTest,
            &InternalPreprocessOp::EnableCullFace,
            &InternalPreprocessOp::EnableBlend,
            &InternalPreprocessOp::ClearColor(0.0, 0.0, 0.0, 0.0),
            &InternalPreprocessOp::ClearDepth(0.0),
            &InternalPreprocessOp::SetCullFaceMode(CullFace::Back),
        ])
    }

    fn material_policy(
        &self,
        _: &RenderState,
        _: &dyn RenderStuff,
    ) -> Result<MaterialPolicy, Error> {
        Ok(MaterialPolicy::FollowEntity)
    }

    fn geometry_policy(
        &self,
        _: &RenderState,
        _: &dyn RenderStuff,
    ) -> Result<GeometryPolicy, Error> {
        Ok(GeometryPolicy::FollowEntity)
    }

    fn collect_policy(
        &mut self,
        _: &RenderState,
        _: &dyn RenderStuff,
    ) -> Result<super::policy::CollectPolicy, Error> {
        todo!()
    }

    fn post_precess(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<&[&dyn PostprocessOp], Error> {
        Ok(&[])
    }
}
