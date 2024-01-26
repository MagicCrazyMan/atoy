use std::ptr::NonNull;

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, notify::Notifier, scene::Scene};

use self::{
    abilities::Abilities, buffer::BufferStore, error::Error, program::ProgramStore,
    state::FrameState, texture::TextureStore,
};

use super::{Pipeline, Render};

pub mod abilities;
pub mod attribute;
pub mod buffer;
pub mod client_wait;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod framebuffer;
pub mod pipeline;
pub mod program;
pub mod renderbuffer;
pub mod shader;
pub mod state;
pub mod stencil;
pub mod texture;
pub mod uniform;
pub mod utilities;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WebGL2ContextOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    alpha: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stencil: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    desynchronized: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    antialias: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fail_if_major_performance_caveat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power_preference: Option<WebGL2ContextPowerPerformance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    premultiplied_alpha: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preserve_drawing_buffer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xr_compatible: Option<bool>,
}

impl Default for WebGL2ContextOptions {
    fn default() -> Self {
        Self {
            alpha: Some(true),
            depth: Some(true),
            stencil: Some(true),
            desynchronized: None,
            antialias: Some(true),
            fail_if_major_performance_caveat: None,
            power_preference: None,
            premultiplied_alpha: None,
            preserve_drawing_buffer: None,
            xr_compatible: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum WebGL2ContextPowerPerformance {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "high-performance")]
    HighPerformance,
    #[serde(rename = "low-power")]
    LowPower,
}

pub struct WebGL2Render {
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
    abilities: Abilities,

    pre_render_notifier: Notifier<RenderEvent>,
    post_render_notifier: Notifier<RenderEvent>,
}

impl Drop for WebGL2Render {
    fn drop(&mut self) {
        self.canvas.remove();
    }
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    pub fn new(
        canvas: HtmlCanvasElement,
        options: Option<WebGL2ContextOptions>,
    ) -> Result<Self, Error> {
        let options = options.unwrap_or(WebGL2ContextOptions::default());
        let gl = canvas
            .get_context_with_context_options(
                "webgl2",
                &serde_wasm_bindgen::to_value(&options).unwrap(),
            )
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WebGL2Unsupported)?;
        let abilities = Abilities::new(gl.clone());

        Ok(Self {
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            texture_store: TextureStore::new(gl.clone(), abilities.clone()),
            abilities,
            gl,
            canvas,

            pre_render_notifier: Notifier::new(),
            post_render_notifier: Notifier::new(),
        })
    }

    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Returns [`WebGl2RenderingContext`].
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Returns the [`ProgramStore`].
    #[inline]
    pub fn program_store(&self) -> &ProgramStore {
        &self.program_store
    }

    /// Returns the mutable [`ProgramStore`].
    #[inline]
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        &mut self.program_store
    }

    /// Returns the [`BufferStore`].
    #[inline]
    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    /// Returns the mutable [`BufferStore`].
    #[inline]
    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }

    /// Returns the [`TextureStore`].
    #[inline]
    pub fn texture_store(&self) -> &TextureStore {
        &self.texture_store
    }

    /// Returns the mutable [`TextureStore`].
    #[inline]
    pub fn texture_store_mut(&mut self) -> &mut TextureStore {
        &mut self.texture_store
    }

    /// Returns the [`Abilities`].
    #[inline]
    pub fn abilities(&self) -> &Abilities {
        &self.abilities
    }

    pub fn pre_render(&mut self) -> &mut Notifier<RenderEvent> {
        &mut self.pre_render_notifier
    }

    pub fn post_render(&mut self) -> &mut Notifier<RenderEvent> {
        &mut self.post_render_notifier
    }
}

impl Render for WebGL2Render {
    type State = FrameState;

    type Error = Error;

    fn render(
        &mut self,
        pipeline: &mut (dyn Pipeline<State = Self::State, Error = Self::Error> + 'static),
        camera: &mut (dyn Camera + 'static),
        scene: &mut Scene,
        timestamp: f64,
    ) -> Result<(), Self::Error> {
        let mut state = FrameState::new(
            timestamp,
            camera,
            self.gl.clone(),
            self.canvas.clone(),
            &mut self.program_store,
            &mut self.buffer_store,
            &mut self.texture_store,
            &mut self.abilities,
        );

        self.pre_render_notifier
            .notify(&mut RenderEvent::new(&mut state));
        pipeline.execute(&mut state, scene)?;
        self.post_render_notifier
            .notify(&mut RenderEvent::new(&mut state));

        Ok(())
    }
}

pub struct RenderEvent(NonNull<FrameState>);

impl RenderEvent {
    fn new(state: &mut FrameState) -> Self {
        Self(unsafe { NonNull::new_unchecked(state) })
    }

    pub fn state(&self) -> &FrameState {
        unsafe { self.0.as_ref() }
    }

    pub fn state_mut(&mut self) -> &mut FrameState {
        unsafe { self.0.as_mut() }
    }
}
