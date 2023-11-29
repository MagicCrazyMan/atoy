use std::{cell::RefCell, collections::HashMap, rc::Rc};

use uuid::Uuid;

use crate::{
    entity::{Entity, RenderEntity},
    render::webgl::error::Error,
};

use super::{RenderPipeline, RenderState, RenderStuff};

pub trait Drawer<Pipeline: RenderPipeline> {
    fn before_draw(
        &mut self,
        collected: &HashMap<Uuid, Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<Vec<Rc<RefCell<Entity>>>, Error>;

    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        collected: &HashMap<Uuid, Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<Option<RenderEntity>, Error>;

    fn after_each_draw(
        &mut self,
        entity: &RenderEntity,
        collected: &HashMap<Uuid, Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;

    fn after_draw(
        &mut self,
        collected: &HashMap<Uuid, Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
