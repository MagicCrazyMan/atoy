pub mod pick;
pub mod policy;
pub mod postprocess;
pub mod preprocess;
pub mod standard;

use wasm_bindgen::JsValue;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::EntityCollection};

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

    /// Gets entity collection that should be draw on current frame.
    fn entity_collection(&mut self) -> &mut EntityCollection;

    /// Gets the main camera for current frame.
    fn camera(&mut self) -> &mut dyn Camera;
}

pub struct RenderState<'a> {
    gl: WebGl2RenderingContext,
    frame_time: f64,
    stuff: &'a mut dyn RenderStuff,
}

impl<'a> RenderState<'a> {
    pub fn new<S>(gl: WebGl2RenderingContext, frame_time: f64, stuff: &'a mut S) -> Self
    where
        S: RenderStuff,
    {
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

    pub fn entity_collection(&mut self) -> &mut EntityCollection {
        self.stuff.entity_collection()
    }

    pub fn camera(&mut self) -> &mut dyn Camera {
        self.stuff.camera()
    }
}

pub trait RenderPipeline {
    fn dependencies(&self) -> Result<(), Error>;

    /// Preparation stage during render procedure.
    fn prepare(&mut self, stuff: &mut dyn RenderStuff) -> Result<(), Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`PreprocessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_process(&mut self, state: &mut RenderState) -> Result<&[&dyn PreprocessOp], Error>;

    /// Returns a [`MaterialPolicy`] which decides what material
    /// to use of each entity during entities collection procedure.
    fn material_policy(&self, state: &RenderState) -> Result<MaterialPolicy, Error>;

    /// Returns a [`GeometryPolicy`] which decides what geometry
    /// to use of each entity during entities collection procedure.
    fn geometry_policy(&self, state: &RenderState) -> Result<GeometryPolicy, Error>;

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
    fn post_precess(&mut self, state: &mut RenderState) -> Result<&[&dyn PostprocessOp], Error>;
}
