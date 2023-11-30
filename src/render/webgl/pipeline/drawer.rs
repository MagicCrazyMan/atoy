use std::{cell::RefCell, rc::Rc};

use crate::{
    entity::{Entity, RenderEntity},
    render::webgl::error::Error,
};

use super::{
    flow::{BeforeDrawFlow, BeforeEachDrawFlow},
    RenderPipeline, RenderState, RenderStuff,
};

pub trait Drawer<Pipeline>
where
    Pipeline: RenderPipeline,
{
    fn before_draw(
        &mut self,
        collected_entities: &[Rc<RefCell<Entity>>],
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<BeforeDrawFlow, Error>;

    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        drawing_index: usize,
        drawing_entities: &[Rc<RefCell<Entity>>],
        collected_entities: &[Rc<RefCell<Entity>>],
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<BeforeEachDrawFlow, Error>;

    fn after_each_draw(
        &mut self,
        entity: &RenderEntity,
        drawing_index: usize,
        drawing_entities: &[Rc<RefCell<Entity>>],
        collected_entities: &[Rc<RefCell<Entity>>],
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;

    fn after_draw(
        &mut self,
        drawing_entities: &[Rc<RefCell<Entity>>],
        collected_entities: &[Rc<RefCell<Entity>>],
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
