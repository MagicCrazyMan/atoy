use std::{cell::RefCell, rc::Rc};

use crate::{
    entity::{Entity, RenderEntity},
    geometry::Geometry,
    material::Material,
    render::webgl::error::Error,
};

use super::{RenderPipeline, RenderState, RenderStuff};

pub trait Drawer<Pipeline: RenderPipeline> {
    fn before_draw(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<Option<Vec<Rc<RefCell<Entity>>>>, Error>;

    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        filtered: &Vec<Rc<RefCell<Entity>>>,
        collected: &Vec<Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<
        Option<(
            Rc<RefCell<Entity>>,
            Rc<RefCell<dyn Geometry>>,
            Rc<RefCell<dyn Material>>,
        )>,
        Error,
    >;

    fn after_each_draw(
        &mut self,
        entity: &RenderEntity,
        filtered: &Vec<Rc<RefCell<Entity>>>,
        collected: &Vec<Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;

    fn after_draw(
        &mut self,
        filtered: &Vec<Rc<RefCell<Entity>>>,
        collected: &Vec<Rc<RefCell<Entity>>>,
        pipeline: &mut Pipeline,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<(), Error>;
}
