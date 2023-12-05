pub mod builtin;
pub mod drawer;
pub mod flow;
pub mod process;

use std::{any::Any, cell::RefCell, rc::Rc};

use gl_matrix4rust::mat4::Mat4;
use smallvec::SmallVec;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    camera::Camera,
    entity::{EntityCollection, Strong},
};

use self::{drawer::Drawer, flow::PreparationFlow, process::Processor};

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
    canvas: HtmlCanvasElement,
    gl: WebGl2RenderingContext,
    frame_time: f64,
    view_matrix: Mat4,
    proj_matrix: Mat4,
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

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn frame_time(&self) -> f64 {
        self.frame_time
    }

    pub fn view_matrix(&self) -> &Mat4 {
        &self.view_matrix
    }

    pub fn proj_matrix(&self) -> &Mat4 {
        &self.proj_matrix
    }
}

pub trait RenderPipeline {
    fn prepare(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<PreparationFlow, Error>;

    fn pre_processors(
        &mut self,
        collected: &[Strong],
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error>;

    fn drawers(
        &mut self,
        collected: &[Strong],
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>, Error>;

    fn post_processors(
        &mut self,
        collected: &[Strong],
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
