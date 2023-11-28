use crate::{
    camera::Camera,
    entity::EntityCollection,
    render::webgl::{draw::CullFace, error::Error},
    scene::Scene,
};

use super::{
    policy::{CollectPolicy, GeometryPolicy, MaterialPolicy},
    postprocess::{standard::Reset, PostProcessor},
    preprocess::{
        standard::{
            ClearColor, ClearDepth, EnableBlend, EnableCullFace, EnableDepthTest, SetCullFaceMode,
            UpdateCamera, UpdateViewport,
        },
        PreProcessor,
    },
    RenderPipeline, RenderState, RenderStuff,
};

pub struct StandardRenderStuff<'a> {
    scene: &'a mut Scene,
}

impl<'a> StandardRenderStuff<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        Self { scene }
    }
}

impl<'a> RenderStuff for StandardRenderStuff<'a> {
    fn camera(&self) -> &dyn Camera {
        self.scene.active_camera()
    }

    fn camera_mut(&mut self) -> &mut dyn Camera {
        self.scene.active_camera_mut()
    }

    fn entity_collection(&self) -> &EntityCollection {
        self.scene.entity_collection()
    }

    fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        self.scene.entity_collection_mut()
    }
}

pub struct StandardPipeline;

impl<'a> RenderPipeline for StandardPipeline {
    fn dependencies(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare(&mut self, _: &mut RenderState, _: &mut dyn RenderStuff) -> Result<(), Error> {
        Ok(())
    }

    fn pre_process(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Vec<Box<dyn PreProcessor>>, Error> {
        Ok(vec![
            Box::new(UpdateCamera),
            Box::new(UpdateViewport),
            Box::new(EnableDepthTest),
            Box::new(EnableCullFace),
            Box::new(EnableBlend),
            Box::new(ClearColor::new(0.0, 0.0, 0.0, 0.0)),
            Box::new(ClearDepth::new(1.0)),
            Box::new(SetCullFaceMode::new(CullFace::Back)),
        ])
    }

    fn material_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<MaterialPolicy, Error> {
        Ok(MaterialPolicy::FollowEntity)
    }

    fn geometry_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<GeometryPolicy, Error> {
        Ok(GeometryPolicy::FollowEntity)
    }

    fn collect_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<CollectPolicy, Error> {
        Ok(CollectPolicy::CollectAll)
    }

    fn post_precess(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Vec<Box<dyn PostProcessor>>, Error> {
        Ok(vec![Box::new(Reset)])
    }
}
