use std::{any::Any, cell::RefCell, rc::Rc};

use smallvec::SmallVec;

use crate::{
    camera::Camera,
    entity::{Entity, EntityCollection, RenderEntity},
    geometry::Geometry,
    material::Material,
    render::webgl::{
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::processor::{
                ClearColor, ClearDepth, EnableBlend, EnableCullFace, EnableDepthTest, Reset,
                SetCullFaceMode, UpdateCamera, UpdateViewport,
            },
            drawer::Drawer,
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

pub struct StandardDrawer;

impl<Pipeline> Drawer<Pipeline> for StandardDrawer
where
    Pipeline: RenderPipeline,
{
    #[inline(always)]
    fn before_draw(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Option<Vec<Rc<RefCell<Entity>>>>, Error> {
        Ok(Some(collected.clone()))
    }

    #[inline(always)]
    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<
        Option<(
            Rc<RefCell<Entity>>,
            Rc<RefCell<dyn Geometry>>,
            Rc<RefCell<dyn Material>>,
        )>,
        Error,
    > {
        let entity_guard = entity.borrow();
        if let (Some(geometry), Some(material)) = (entity_guard.geometry(), entity_guard.material())
        {
            Ok(Some((
                Rc::clone(entity),
                Rc::clone(geometry),
                Rc::clone(material),
            )))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn after_each_draw(
        &mut self,
        _: &RenderEntity,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn after_draw(
        &mut self,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub struct StandardPipeline {
    pick_detection_position: Option<(u32, u32)>,
}

impl StandardPipeline {
    pub fn new() -> Self {
        Self {
            pick_detection_position: None,
        }
    }

    pub fn set_pick_detection(&mut self, x: u32, y: u32) {
        self.pick_detection_position = Some((x, y));
    }
}

impl RenderPipeline for StandardPipeline {
    #[inline(always)]
    fn prepare(&mut self, _: &mut RenderState, _: &mut dyn RenderStuff) -> Result<bool, Error> {
        Ok(true)
    }

    #[inline(always)]
    fn pre_processors(
        &mut self,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 16]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 16]> = SmallVec::new();
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

    #[inline(always)]
    fn drawers(
        &mut self,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Drawer<Self>>; 8]>, Error> {
        let mut drawers: SmallVec<[Box<dyn Drawer<Self>>; 8]> = SmallVec::new();
        drawers.push(Box::new(StandardDrawer));
        Ok(drawers)
    }

    #[inline(always)]
    fn post_processors(
        &mut self,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 16]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 16]> = SmallVec::new();
        processors.push(Box::new(Reset));
        Ok(processors)
    }

    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline(always)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
