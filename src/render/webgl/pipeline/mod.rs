pub mod pick;
pub mod policy;
pub mod postprocess;
pub mod preprocess;
pub mod standard;

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::EntityCollection};

use self::{
    policy::{CollectPolicy, GeometryPolicy, MaterialPolicy},
    postprocess::PostProcessor,
    preprocess::PreProcessor,
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

pub trait RenderPipeline<Stuff>
where
    Stuff: RenderStuff,
{
    fn dependencies(&self) -> Result<(), Error>;

    /// Preparation stage during render procedure.
    fn prepare(&mut self, state: &mut RenderState, stuff: &mut Stuff) -> Result<(), Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`PreProcessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_process(
        &mut self,
        state: &mut RenderState,
        stuff: &mut Stuff,
    ) -> Result<Vec<Box<dyn PreProcessor<Stuff>>>, Error>;

    /// Returns a [`MaterialPolicy`] which decides what material
    /// to use of each entity during entities collection procedure.
    fn material_policy(&self, state: &RenderState, stuff: &Stuff) -> Result<MaterialPolicy, Error>;

    /// Returns a [`GeometryPolicy`] which decides what geometry
    /// to use of each entity during entities collection procedure.
    fn geometry_policy(&self, state: &RenderState, stuff: &Stuff) -> Result<GeometryPolicy, Error>;

    fn collect_policy(
        &mut self,
        state: &RenderState,
        stuff: &Stuff,
    ) -> Result<CollectPolicy, Error>;

    /// Postprecess stages during render procedure.
    /// Just similar as `pre_process`,`post_precess`
    /// also accepts multiple [`PostProcessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn post_precess(
        &mut self,
        state: &mut RenderState,
        stuff: &mut Stuff,
    ) -> Result<Vec<Box<dyn PostProcessor<Stuff>>>, Error>;
}
