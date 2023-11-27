pub mod pick;
pub mod policy;
pub mod postprocess;
pub mod preprocess;
pub mod standard;

use wasm_bindgen::{JsCast, JsValue};
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
    fn entity_collection(&self) -> &EntityCollection;

    /// Gets mutable entity collection that should be draw on current frame.
    fn entity_collection_mut(&mut self) -> &mut EntityCollection;

    /// Gets the main camera for current frame.
    fn camera(&self) -> &dyn Camera;

    /// Gets mutable the main camera for current frame.
    fn camera_mut(&mut self) -> &mut dyn Camera;
}

pub struct RenderState {
    pub gl: WebGl2RenderingContext,
    pub frame_time: f64,
}

impl RenderState {
    pub(super) fn new(stuff: &dyn RenderStuff, frame_time: f64) -> Result<Self, Error> {
        let gl = stuff
            .canvas()
            .get_context_with_context_options(
                "webgl2",
                stuff.ctx_options().unwrap_or(&JsValue::undefined()),
            )
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WenGL2Unsupported)?;

        Ok(Self { gl, frame_time })
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
    fn pre_process(
        &mut self,
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<&[&dyn PreprocessOp], Error>;

    /// Returns a [`MaterialPolicy`] which decides what material
    /// to use of each entity during entities collection procedure.
    fn material_policy(
        &self,
        state: &RenderState,
        stuff: &dyn RenderStuff,
    ) -> Result<MaterialPolicy, Error>;

    /// Returns a [`GeometryPolicy`] which decides what geometry
    /// to use of each entity during entities collection procedure.
    fn geometry_policy(
        &self,
        state: &RenderState,
        stuff: &dyn RenderStuff,
    ) -> Result<GeometryPolicy, Error>;

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
        state: &mut RenderState,
        stuff: &mut dyn RenderStuff,
    ) -> Result<&[&dyn PostprocessOp], Error>;
}
