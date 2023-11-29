pub mod builtin;
pub mod drawer;
pub mod process;

use std::{any::Any, cell::RefCell, rc::Rc};

use gl_matrix4rust::mat4::Mat4;
use smallvec::SmallVec;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    camera::Camera,
    entity::{Entity, EntityCollection},
};

use self::{drawer::Drawer, process::Processor};

use super::error::Error;

/// Basic stuffs for running the render program.
pub trait RenderStuff {
    /// Gets entity collection that should be draw on current frame.
    fn entity_collection(&self) -> &EntityCollection;

    /// Gets mutable entity collection that should be draw on current frame.
    fn entity_collection_mut(&mut self) -> &mut EntityCollection;

    /// Gets the main camera for current frame.
    fn camera(&self) -> &dyn Camera;

    /// Gets mutable the main camera for current frame.
    fn camera_mut(&mut self) -> &mut dyn Camera;
}

pub struct RenderState {
    pub canvas: HtmlCanvasElement,
    pub gl: WebGl2RenderingContext,
    pub frame_time: f64,
    pub view_matrix: Mat4,
    pub proj_matrix: Mat4,
}

impl RenderState {
    pub(super) fn new(
        canvas: HtmlCanvasElement,
        gl: WebGl2RenderingContext,
        stuff: &dyn RenderStuff,
        frame_time: f64,
    ) -> Self {
        Self {
            canvas,
            gl,
            frame_time,
            view_matrix: stuff.camera().view_matrix(),
            proj_matrix: stuff.camera().proj_matrix(),
        }
    }
}

pub trait RenderPipeline {
    /// Preparation stage during render procedure.
    fn prepare(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<bool, Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`Processor`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_processors(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error>;

    fn drawers(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>, Error>;

    /// Postprecess stages during render procedure.
    /// Just similar as `pre_process`,`post_precess`
    /// also accepts multiple [`Processor`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn post_processors(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
