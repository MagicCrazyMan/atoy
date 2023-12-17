use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, HtmlElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
};

use crate::document;

use self::{buffer::BufferStore, error::Error, program::ProgramStore, texture::TextureStore};

use super::pp::{Pipeline, State, Stuff};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod program;
pub mod stencil;
pub mod texture;
pub mod uniform;

// #[wasm_bindgen(typescript_custom_section)]
// const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
// export type WebGL2RenderOptions = WebGLContextAttributes;
// "#;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
//     pub type WebGL2RenderOptionsObject;
// }

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
    mount: Option<HtmlElement>,
    canvas: HtmlCanvasElement,
    // require for storing callback closure function
    resize_observer: (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>),
    gl: WebGl2RenderingContext,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
}

impl WebGL2Render {
    pub fn new() -> Result<Self, Error> {
        Self::new_inner(None, None)
    }

    pub fn with_mount(mount: &str) -> Result<Self, Error> {
        Self::new_inner(Some(mount), None)
    }

    /// Constructs a new WebGL2 render.
    fn new_inner(
        mount: Option<&str>,
        options: Option<WebGL2ContextOptions>,
    ) -> Result<Self, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailure)?;
        canvas
            .set_attribute("tabindex", "0")
            .map_err(|_| Error::CreateCanvasFailure)?;
        canvas
            .style()
            .set_css_text("width: 100%; height: 100%; outline: none;");

        let resize_observer = Self::observer_canvas_size(&canvas);

        let options = options.unwrap_or(WebGL2ContextOptions::default());
        let gl = canvas
            .get_context_with_context_options(
                "webgl2",
                &serde_wasm_bindgen::to_value(&options).unwrap(),
            )
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WenGL2Unsupported)?;

        let mut render = Self {
            mount: None,
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            // buffer_store: BufferStore::with_max_memory(gl.clone(), 1000),
            texture_store: TextureStore::new(gl.clone()),
            canvas,
            gl,
            resize_observer,
        };

        render.set_mount(mount)?;

        Ok(render)
    }

    fn observer_canvas_size(
        canvas: &HtmlCanvasElement,
    ) -> (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>) {
        // create observer observing size change event of canvas
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Some(canvas) = target.dyn_ref::<HtmlCanvasElement>() else {
                return;
            };

            canvas.set_width(canvas.client_width() as u32);
            canvas.set_height(canvas.client_height() as u32);
        });

        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref()).unwrap();
        resize_observer.observe(canvas);

        (resize_observer, resize_observer_callback)
    }
}

impl WebGL2Render {
    /// Gets mounted target element.
    pub fn mount(&self) -> Option<&HtmlElement> {
        match &self.mount {
            Some(mount) => Some(mount),
            None => None,
        }
    }

    /// Mounts WebGl canvas to an element.
    pub fn set_mount(&mut self, mount: Option<&str>) -> Result<(), Error> {
        if let Some(mount) = mount {
            if !mount.is_empty() {
                // gets and sets mount target using `document.getElementById`
                let mount = document()
                    .get_element_by_id(&mount)
                    .and_then(|ele| ele.dyn_into::<HtmlElement>().ok())
                    .ok_or(Error::MountElementNotFound)?;

                // mounts canvas to target
                if let Err(err) = mount.append_child(&self.canvas) {
                    return Err(Error::MountElementFailure);
                };
                let width = mount.client_width() as u32;
                let height = mount.client_height() as u32;
                self.canvas.set_width(width);
                self.canvas.set_height(height);

                self.mount = Some(mount);

                return Ok(());
            }
        }

        // for all other situations, removes canvas from mount target
        self.canvas.remove();
        self.mount = None;
        Ok(())
    }
}

impl WebGL2Render {
    /// Renders a frame with stuff and a pipeline.
    pub fn render<S>(
        &mut self,
        pipeline: &mut Pipeline,
        stuff: &mut S,
        timestamp: f64,
    ) -> Result<(), Error>
    where
        S: Stuff,
    {
        // constructs render state
        let mut state = State::new(
            self.gl.clone(),
            self.canvas.clone(),
            timestamp,
            &mut self.program_store,
            &mut self.buffer_store,
            &mut self.texture_store,
        );
        let state = &mut state;

        pipeline.execute(state, stuff)?;

        Ok(())
    }
}

impl Drop for WebGL2Render {
    fn drop(&mut self) {
        // cleanups observers
        self.resize_observer.0.disconnect();
    }
}
