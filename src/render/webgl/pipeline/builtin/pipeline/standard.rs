use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

use smallvec::SmallVec;

use crate::{
    camera::Camera,
    entity::{Entity, EntityCollection, RenderEntity},
    render::webgl::{
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::processor::{
                ClearColor, ClearDepth, EnableBlend, EnableCullFace, EnableDepthTest, Reset,
                SetCullFaceMode, UpdateCamera, UpdateViewport,
            },
            drawer::Drawer,
            flow::{BeforeDrawFlow, BeforeEachDrawFlow, PreparationFlow},
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
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeDrawFlow, Error> {
        Ok(BeforeDrawFlow::FollowCollectedEntities)
    }

    #[inline(always)]
    fn before_each_draw(
        &mut self,
        _: &Rc<RefCell<Entity>>,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeEachDrawFlow, Error> {
        Ok(BeforeEachDrawFlow::FollowEntity)
    }

    #[inline(always)]
    fn after_each_draw(
        &mut self,
        _: &RenderEntity,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn after_draw(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }
}

pub struct StandardPipeline {
    pick_drawer: Rc<RefCell<PickDetectionDrawer>>,
    picked_entity: Rc<RefCell<Option<Weak<RefCell<Entity>>>>>,
    pre_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>,
    post_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>,
    drawers: SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>,
}

impl StandardPipeline {
    pub fn new() -> Self {
        let mut pre_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]> = SmallVec::new();
        pre_processors.push(Rc::new(RefCell::new(UpdateCamera)));
        pre_processors.push(Rc::new(RefCell::new(UpdateViewport)));
        pre_processors.push(Rc::new(RefCell::new(EnableDepthTest)));
        pre_processors.push(Rc::new(RefCell::new(EnableCullFace)));
        pre_processors.push(Rc::new(RefCell::new(EnableBlend)));
        pre_processors.push(Rc::new(RefCell::new(ClearColor::new(0.0, 0.0, 0.0, 0.0))));
        pre_processors.push(Rc::new(RefCell::new(ClearDepth::new(1.0))));
        pre_processors.push(Rc::new(RefCell::new(SetCullFaceMode::new(CullFace::Back))));

        let mut post_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]> = SmallVec::new();
        post_processors.push(Rc::new(RefCell::new(Reset)));

        let picked_entity = Rc::new(RefCell::new(None));
        let pick_drawer = Rc::new(RefCell::new(PickDetectionDrawer::new(Rc::clone(
            &picked_entity,
        ))));
        let mut drawers: SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]> = SmallVec::new();
        drawers.push(Rc::clone(&pick_drawer) as Rc<RefCell<dyn Drawer<Self>>>);
        drawers.push(Rc::new(RefCell::new(StandardDrawer)));

        Self {
            pick_drawer,
            picked_entity,
            pre_processors,
            post_processors,
            drawers,
        }
    }

    pub fn set_pick_position(&mut self, x: i32, y: i32) {
        self.pick_drawer.borrow_mut().set_position(x, y);
    }

    pub fn picked_entity(&self) -> Option<Rc<RefCell<Entity>>> {
        self.picked_entity
            .borrow()
            .as_ref()
            .and_then(|entity| entity.upgrade())
    }

    pub fn take_picked_entity(&mut self) -> Option<Rc<RefCell<Entity>>> {
        self.picked_entity
            .borrow_mut()
            .take()
            .and_then(|entity| entity.upgrade())
    }
}

impl RenderPipeline for StandardPipeline {
    #[inline(always)]
    fn prepare(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<PreparationFlow, Error> {
        Ok(PreparationFlow::Continue)
    }

    #[inline(always)]
    fn pre_processors(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error> {
        Ok(self.pre_processors.clone())
    }

    #[inline(always)]
    fn drawers(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>, Error> {
        Ok(self.drawers.clone())
    }

    #[inline(always)]
    fn post_processors(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error> {
        Ok(self.post_processors.clone())
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
