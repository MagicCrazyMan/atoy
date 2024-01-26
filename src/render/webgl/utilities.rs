use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlRenderbuffer, WebGlTexture};

pub fn array_buffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlBuffer> {
    gl.get_parameter(WebGl2RenderingContext::ARRAY_BUFFER_BINDING)
        .and_then(|v| v.dyn_into::<WebGlBuffer>())
        .ok()
}

pub fn active_texture_unit(gl: &WebGl2RenderingContext) -> u32 {
    gl.get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
        .unwrap()
}

pub fn texture_binding_2d(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)
        .and_then(|v| v.dyn_into::<WebGlTexture>())
        .ok()
}


pub fn renderbuffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlRenderbuffer> {
    gl.get_parameter(WebGl2RenderingContext::RENDERBUFFER_BINDING)
        .and_then(|v| v.dyn_into::<WebGlRenderbuffer>())
        .ok()
}
