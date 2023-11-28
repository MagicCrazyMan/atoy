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

pub struct StandardRenderStuff<'s> {
    scene: &'s mut Scene,
}

impl<'s> StandardRenderStuff<'s> {
    pub fn new(scene: &'s mut Scene) -> Self {
        Self { scene }
    }
}

impl<'s> RenderStuff for StandardRenderStuff<'s> {
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

pub struct StandardPipeline {
    // pre_processor: Vec<Box<dyn PreProcessor>>,
}

impl StandardPipeline {
    pub fn new() -> Self {
        Self {
            // pre_processor: vec![
            //     Box::new(UpdateCamera),
            //     Box::new(UpdateViewport),
            //     Box::new(EnableDepthTest),
            //     Box::new(EnableCullFace),
            //     Box::new(EnableBlend),
            //     Box::new(ClearColor::new(0.0, 0.0, 0.0, 0.0)),
            //     Box::new(ClearDepth::new(0.0)),
            //     Box::new(SetCullFaceMode::new(CullFace::Back)),
            // ],
        }
    }
}

impl<'s, Stuff: RenderStuff> RenderPipeline<Stuff> for StandardPipeline {
    fn dependencies(&self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare(&mut self, _: &mut RenderState, _: &mut Stuff) -> Result<(), Error> {
        Ok(())
    }

    fn pre_process(
        &mut self,
        _: &mut RenderState,
        _: &mut Stuff,
    ) -> Result<Vec<Box<dyn PreProcessor<Stuff>>>, Error> {
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

    fn material_policy(&self, _: &RenderState, _: &Stuff) -> Result<MaterialPolicy, Error> {
        Ok(MaterialPolicy::FollowEntity)
    }

    fn geometry_policy(&self, _: &RenderState, _: &Stuff) -> Result<GeometryPolicy, Error> {
        Ok(GeometryPolicy::FollowEntity)
    }

    fn collect_policy(&mut self, _: &RenderState, _: &Stuff) -> Result<CollectPolicy, Error> {
        Ok(CollectPolicy::CollectAll)
    }

    fn post_precess(
        &mut self,
        _: &mut RenderState,
        _: &mut Stuff,
    ) -> Result<Vec<Box<dyn PostProcessor<Stuff>>>, Error> {
        Ok(vec![Box::new(Reset)])
    }
}
