use gl_matrix4rust::vec4::Vec4;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsError, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::scene::Scene;

pub mod buffer;
pub mod compiler;

#[derive(Serialize)]
pub enum PowerPerformance {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "high-performance")]
    HighPerformance,
    #[serde(rename = "low-power")]
    LowPower,
}

#[derive(Default, Serialize)]
pub struct WebGL2RenderContextOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stencil: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desynchronized: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antialias: Option<bool>,
    #[serde(
        rename(serialize = "failIfMajorPerformanceCaveat"),
        skip_serializing_if = "Option::is_none"
    )]
    pub fail_if_major_performance_caveat: Option<bool>,
    #[serde(
        rename(serialize = "powerPreference"),
        skip_serializing_if = "Option::is_none"
    )]
    pub power_preference: Option<PowerPerformance>,
    #[serde(
        rename(serialize = "premultipliedAlpha"),
        skip_serializing_if = "Option::is_none"
    )]
    pub premultiplied_alpha: Option<bool>,
    #[serde(
        rename(serialize = "preserveDrawingBuffer"),
        skip_serializing_if = "Option::is_none"
    )]
    pub preserve_drawing_buffer: Option<bool>,
    #[serde(
        rename(serialize = "xrCompatible"),
        skip_serializing_if = "Option::is_none"
    )]
    pub xr_compatible: Option<bool>,
}

pub struct WebGL2Render {
    gl: WebGl2RenderingContext,
    depth_test: bool,
    cull_face_mode: Option<u32>,
    clear_color: Vec4,
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render, without any context options.
    pub fn new(scene: &Scene) -> Result<Self, JsError> {
        Ok(Self {
            gl: Self::gl_context(scene.canvas(), None)?,
            depth_test: true,
            cull_face_mode: None,
            clear_color: Vec4::new(),
        })
    }

    /// Constructs a new WebGL2 render, with context options.
    pub fn with_context_options(
        scene: &Scene,
        context_options: &WebGL2RenderContextOptions,
    ) -> Result<Self, JsError> {
        Ok(Self {
            gl: Self::gl_context(scene.canvas(), Some(context_options))?,
            depth_test: true,
            cull_face_mode: None,
            clear_color: Vec4::new(),
        })
    }

    /// Gets WebGl2RenderingContext.
    fn gl_context(
        canvas: &HtmlCanvasElement,
        context_options: Option<&WebGL2RenderContextOptions>,
    ) -> Result<WebGl2RenderingContext, JsError> {
        let context_options = match context_options {
            Some(context_options) => serde_wasm_bindgen::to_value(context_options)
                .or(Err(JsError::new("failed to parse WebGL2 context options")))?,
            None => JsValue::UNDEFINED,
        };

        let gl = canvas
            .get_context_with_context_options("webgl2", &context_options)
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(JsError::new("failed to get WebGL2 context"))?;

        Ok(gl)
    }

    pub fn render(&self, scene: &Scene) -> Result<(), JsError> {
        let gl = &self.gl;

        // clear scene
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        Ok(())
    }

    pub fn depth_test(&self) -> bool {
        self.depth_test
    }

    pub fn set_depth_test(&mut self, depth_test: bool) {
        self.depth_test = depth_test;
        if self.depth_test {
            self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        } else {
            self.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
    }

    pub fn cull_face(&self) -> Option<u32> {
        self.cull_face_mode
    }

    pub fn set_cull_face(&mut self, cull_face_mode: Option<u32>) {
        self.cull_face_mode = cull_face_mode;
        match self.cull_face_mode {
            Some(cull_face_mode) => {
                self.gl.enable(WebGl2RenderingContext::CULL_FACE);
                self.gl.cull_face(cull_face_mode)
            }
            None => self.gl.disable(WebGl2RenderingContext::CULL_FACE),
        }
    }

    pub fn clear_color(&self) -> Vec4<f32> {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
        self.gl.clear_color(
            self.clear_color.0[0],
            self.clear_color.0[1],
            self.clear_color.0[2],
            self.clear_color.0[3],
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::render::webgl2render::PowerPerformance;

//     use super::WebGL2RenderContextOptions;

//     #[test]
//     fn test_context_options() {
//         let mut options = WebGL2RenderContextOptions::default();
//         options.power_preference = Some(PowerPerformance::HighPerformance);
//         println!("{}", serde_json::to_string_pretty(&options).unwrap());
//     }
// }
