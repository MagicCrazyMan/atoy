pub mod builtin;
pub mod policy;
pub mod process;

use std::any::Any;

use smallvec::SmallVec;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::EntityCollection};

use self::{
    policy::{CollectPolicy, GeometryPolicy, MaterialPolicy, PreparationPolicy},
    process::Processor,
};

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
}

pub trait RenderPipeline {
    fn dependencies(&mut self) -> Result<(), Error>;

    /// Preparation stage during render procedure.
    fn prepare(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<PreparationPolicy, Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`Processor`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_processors(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error>;

    /// Returns a [`MaterialPolicy`] which decides what material
    /// to use of each entity during entities collection procedure.
    fn material_policy(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<MaterialPolicy, Error>;

    /// Returns a [`GeometryPolicy`] which decides what geometry
    /// to use of each entity during entities collection procedure.
    fn geometry_policy(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<GeometryPolicy, Error>;

    fn collect_policy(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<CollectPolicy, Error>;

    /// Postprecess stages during render procedure.
    /// Just similar as `pre_process`,`post_precess`
    /// also accepts multiple [`Processor`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn post_processors(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
