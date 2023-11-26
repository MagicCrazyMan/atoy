pub mod pick;
pub mod postprocess;
pub mod preprocess;
pub mod standard;

use wasm_bindgen::JsValue;
use web_sys::{js_sys::Object, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::Entity, material::Material, scene::Scene};

use self::{postprocess::PostprocessOp, preprocess::PreprocessOp};

use super::{draw::CullFace, error::Error};

/// Basic stuffs for running the render program.
pub trait RenderStuff<'s> {
    /// Rendering canvas.
    fn canvas(&'s self) -> &'s HtmlCanvasElement;

    /// WebGL2 context options.
    /// Checks [MDN References](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/getContext)
    /// for more details.
    fn ctx_options(&'s self) -> Option<&'s JsValue>;

    /// Gets entities that should be draw on current frame.
    fn entities(&'s mut self) -> &'s mut [Entity];

    /// Gets the main camera for current frame.
    fn camera(&'s mut self) -> &'s mut dyn Camera;
}

pub struct RenderState<S> {
    stuff: S,
    gl: WebGl2RenderingContext,
    frame_time: f64,
}

impl<'s, S> RenderState<S>
where
    S: RenderStuff<'s>,
{
    pub fn new(gl: WebGl2RenderingContext, frame_time: f64, stuff: S) -> Self {
        Self {
            gl,
            frame_time,
            stuff,
        }
    }

    pub fn gl(&'s self) -> &'s WebGl2RenderingContext {
        &self.gl
    }

    pub fn frame_time(&'s self) -> f64 {
        self.frame_time
    }

    pub fn canvas(&'s self) -> &'s HtmlCanvasElement {
        self.stuff.canvas()
    }

    pub fn entities(&'s mut self) -> &'s mut [Entity] {
        self.stuff.entities()
    }

    pub fn camera(&'s mut self) -> &'s mut dyn Camera {
        self.stuff.camera()
    }
}

/// Material policy telling render program what material should be used for a entity.
pub enum MaterialPolicy<'s> {
    /// Uses the material provides by entity.
    FollowEntity,
    /// Forces all entities render with a specified material.
    Overwrite(&'s mut dyn Material),
    /// Decides what material to use for each entity.
    Custom(&'s dyn Fn(&Entity)),
}

pub trait GeometryPolicy {
    fn name(&self) -> &str;
}

pub trait RenderPipeline<'s, 'p: 's, S>
where
    S: RenderStuff<'s>,
{
    fn dependencies(&'p mut self) -> Result<(), Error>;

    /// Preparation stage during render procedure.
    /// Developer should provide a [`RenderState`] telling
    /// render program how to render current frame.
    fn prepare(&'p mut self) -> Result<S, Error>;

    /// Preprocess stages during render procedure.
    /// Developer could provide multiple [`PreprocessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn pre_process(
        &'p mut self,
        state: &'p RenderState<S>,
    ) -> Result<&'p [&'p dyn PreprocessOp<S>], Error>;

    // fn geometry_policy(&'p mut self, state: &'p RenderState<S>) -> Result<(), Error>;

    /// Postprecess stages during render procedure.
    /// Just similar as `pre_process`,`post_precess`
    /// also accepts multiple [`PostprocessOp`]s
    /// and render program will execute them in order.
    /// Returning a empty slice makes render program do nothing.
    fn post_precess(
        &'p mut self,
        state: &'p RenderState<S>,
    ) -> Result<&'p [&'p dyn PostprocessOp<S>], Error>;
}
