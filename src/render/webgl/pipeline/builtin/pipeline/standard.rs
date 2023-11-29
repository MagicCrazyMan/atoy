use std::any::Any;

use smallvec::SmallVec;

use crate::{
    camera::Camera,
    entity::EntityCollection,
    render::webgl::{
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::processor::{
                ClearColor, ClearDepth, EnableBlend, EnableCullFace, EnableDepthTest, Reset,
                SetCullFaceMode, UpdateCamera, UpdateViewport,
            },
            policy::{CollectPolicy, GeometryPolicy, MaterialPolicy, PreparationPolicy},
            process::Processor,
            RenderPipeline, RenderState, RenderStuff,
        },
    },
    scene::Scene,
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

    fn prepare(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<PreparationPolicy, Error> {
        Ok(PreparationPolicy::Continue)
    }

    fn pre_processors(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 12]> = SmallVec::new();
        processors.push(Box::new(UpdateCamera));
        processors.push(Box::new(UpdateViewport));
        processors.push(Box::new(EnableDepthTest));
        processors.push(Box::new(EnableCullFace));
        processors.push(Box::new(EnableBlend));
        processors.push(Box::new(ClearColor::new(0.0, 0.0, 0.0, 0.0)));
        processors.push(Box::new(ClearDepth::new(1.0)));
        processors.push(Box::new(SetCullFaceMode::new(CullFace::Back)));
        Ok(processors)
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

    fn post_processors(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 12]> = SmallVec::new();
        processors.push(Box::new(Reset));
        Ok(processors)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
