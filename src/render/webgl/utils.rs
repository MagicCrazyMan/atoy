use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use super::{conversion::ToGlEnum, texture::TextureTarget};

pub fn array_buffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlBuffer> {
    gl.get_parameter(WebGl2RenderingContext::ARRAY_BUFFER_BINDING)
        .unwrap()
        .cast_into_unchecked::<WebGlBuffer>()
}

pub fn pixel_unpack_buffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlBuffer> {
    gl.get_parameter(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER_BINDING)
        .unwrap()
        .cast_into_unchecked::<WebGlBuffer>()
}

pub fn active_texture_unit(gl: &WebGl2RenderingContext) -> u32 {
    gl.get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
        .ok()
        .map(|v| v.as_f64().unwrap())
        .map(|v| v as u32)
        .unwrap()
}

pub fn texture_binding_2d(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_binding_cube_map(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_CUBE_MAP)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_base_level(gl: &WebGl2RenderingContext, target: TextureTarget) -> Option<usize> {
    gl.get_tex_parameter(target.gl_enum(), WebGl2RenderingContext::TEXTURE_BASE_LEVEL)
        .as_f64()
        .map(|v| v as usize)
}

pub fn texture_max_level(gl: &WebGl2RenderingContext, target: TextureTarget) -> Option<usize> {
    gl.get_tex_parameter(target.gl_enum(), WebGl2RenderingContext::TEXTURE_MAX_LEVEL)
        .as_f64()
        .map(|v| v as usize)
}

pub fn renderbuffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlRenderbuffer> {
    gl.get_parameter(WebGl2RenderingContext::RENDERBUFFER_BINDING)
        .unwrap()
        .cast_into_unchecked::<WebGlRenderbuffer>()
}

pub fn framebuffer_binding(gl: &WebGl2RenderingContext) -> Option<WebGlFramebuffer> {
    gl.get_parameter(WebGl2RenderingContext::FRAMEBUFFER_BINDING)
        .unwrap()
        .cast_into_unchecked::<WebGlFramebuffer>()
}

trait CastIfTruthy {
    fn cast_into<T>(self) -> Result<T, Self>
    where
        Self: Sized,
        T: JsCast;

    fn cast_ref<T>(&self) -> Option<&T>
    where
        T: JsCast;

    fn cast_into_unchecked<T>(self) -> Option<T>
    where
        T: JsCast;

    fn cast_ref_unchecked<T>(&self) -> Option<&T>
    where
        T: JsCast;
}

impl CastIfTruthy for JsValue {
    fn cast_into<T>(self) -> Result<T, Self>
    where
        Self: Sized,
        T: JsCast,
    {
        if self.is_truthy() {
            self.dyn_into::<T>()
        } else {
            Err(self)
        }
    }

    fn cast_ref<T>(&self) -> Option<&T>
    where
        T: JsCast,
    {
        if self.is_truthy() {
            self.dyn_ref::<T>()
        } else {
            None
        }
    }

    fn cast_into_unchecked<T>(self) -> Option<T>
    where
        T: JsCast,
    {
        if self.is_truthy() {
            Some(self.dyn_into::<T>().unwrap())
        } else {
            None
        }
    }

    fn cast_ref_unchecked<T>(&self) -> Option<&T>
    where
        T: JsCast,
    {
        if self.is_truthy() {
            Some(self.dyn_ref::<T>().unwrap())
        } else {
            None
        }
    }
}
