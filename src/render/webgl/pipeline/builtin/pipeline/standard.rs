use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

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

use super::picking::PickDetectionDrawer;

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
    ) -> Result<Option<(Rc<RefCell<Entity>>, *mut dyn Geometry, *mut dyn Material)>, Error> {
        let mut entity_guard = entity.borrow_mut();
        if let (Some(geometry), Some(material)) =
            (entity_guard.geometry_raw(), entity_guard.material_raw())
        {
            Ok(Some((Rc::clone(entity), geometry, material)))
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
    pick_drawer: Rc<RefCell<PickDetectionDrawer>>,
    picked_entity: Option<Weak<RefCell<Entity>>>,
}

impl StandardPipeline {
    pub fn new() -> Self {
        Self {
            pick_drawer: Rc::new(RefCell::new(PickDetectionDrawer::new())),
            picked_entity: None,
        }
    }

    pub fn set_pick_detection(&mut self, x: i32, y: i32) {
        self.pick_drawer.borrow_mut().set_position(x, y);
    }

    pub fn picked_entity(&self) -> Option<Rc<RefCell<Entity>>> {
        self.picked_entity
            .as_ref()
            .and_then(|entity| entity.upgrade())
    }

    pub fn take_picked_entity(&mut self) -> Option<Rc<RefCell<Entity>>> {
        self.picked_entity
            .take()
            .and_then(|entity| entity.upgrade())
    }

    pub(super) fn set_picked_entity(&mut self, entity: Option<Weak<RefCell<Entity>>>) {
        self.picked_entity = entity;
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
    ) -> Result<SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>, Error> {
        let mut drawers: SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]> = SmallVec::new();
        drawers.push(Rc::clone(&self.pick_drawer) as Rc<RefCell<dyn Drawer<Self>>>);
        drawers.push(Rc::new(RefCell::new(StandardDrawer)));
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
