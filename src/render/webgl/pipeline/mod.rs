pub mod pick;
pub mod policy;
pub mod postprocess;
pub mod preprocess;
pub mod standard;

use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::Entity, material::Material};

use self::{
    policy::{GeometryPolicy, MaterialPolicy},
    postprocess::PostprocessOp,
    preprocess::PreprocessOp,
};

use super::error::Error;

/// Basic stuffs for running the render program.
pub trait RenderStuff {
    /// Rendering canvas.
    fn canvas(&self) -> &HtmlCanvasElement;

    /// WebGL2 context options.
    /// Checks [MDN References](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/getContext)
    /// for more details.
    fn ctx_options(&self) -> Option<&JsValue>;

    /// Gets entities that should be draw on current frame.
    fn entities(&mut self) -> &mut [Entity];

    /// Gets the main camera for current frame.
    fn camera(&mut self) -> &mut dyn Camera;
}

pub struct RenderState<'a, S> {
    gl: WebGl2RenderingContext,
    frame_time: f64,
    stuff: &'a mut S,
}

impl<'a, S> RenderState<'a, S>
where
    S: RenderStuff,
{
    pub fn new(gl: WebGl2RenderingContext, frame_time: f64, stuff: &'a mut S) -> Self {
        Self {
            gl,
            frame_time,
            stuff,
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn frame_time(&self) -> f64 {
        self.frame_time
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        self.stuff.canvas()
    }

    pub fn entities(&mut self) -> &mut [Entity] {
        self.stuff.entities()
    }

    pub fn camera(&mut self) -> &mut dyn Camera {
        self.stuff.camera()
    }
}

pub trait RenderPipeline<S>
where
    S: RenderStuff,
{
    fn dependencies(&self) -> Result<(), Error>;

    /// Preparation stage during render procedure.
    fn prepare(&mut self, stuff: &mut S) -> Result<(), Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`PreprocessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_process(&mut self, state: &mut RenderState<S>)
        -> Result<&[&dyn PreprocessOp<S>], Error>;

    /// Returns a [`MaterialPolicy`] which decides what material
    /// to use of each entity during entities collection procedure.
    // fn material_policy<'a>(&'a self, state: &'a RenderState<'a, S>) -> Result<M, Error>;
    fn material_policy<'a, 'b, 'c>(&'a self, state: &'b RenderState<S>) -> Result<MaterialPolicy<'c, S>, Error>;

    /// Returns a [`GeometryPolicy`] which decides what geometry
    /// to use of each entity during entities collection procedure.
    fn geometry_policy(&self, state: &RenderState<S>) -> Result<GeometryPolicy<S>, Error>;

    // fn collect_policy<'a>(
    //     &'a mut self,
    //     state: &'a mut RenderState<'a, S>,
    // ) -> Result<(MaterialPolicy<'a, S>, GeometryPolicy<'a, S>), Error> {
    //     Ok((self.material_policy(state)?, self.geometry_policy(state)?))
    // }

    /// Postprecess stages during render procedure.
    /// Just similar as `pre_process`,`post_precess`
    /// also accepts multiple [`PostprocessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn post_precess(
        &mut self,
        state: &mut RenderState<S>,
    ) -> Result<&[&dyn PostprocessOp<S>], Error>;
}
