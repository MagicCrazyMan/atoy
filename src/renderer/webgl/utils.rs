use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    ExtTextureFilterAnisotropic, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer,
    WebGlRenderbuffer, WebGlTexture,
};

use super::{
    conversion::ToGlEnum,
    texture::{TextureTarget, TextureUnpackColorSpaceConversion},
};

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

pub fn texture_active_texture_unit(gl: &WebGl2RenderingContext) -> u32 {
    gl.get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
        .ok()
        .map(|v| v.as_f64().unwrap())
        .map(|v| v as u32)
        .unwrap()
}

pub fn texture_binding(gl: &WebGl2RenderingContext, target: TextureTarget) -> Option<WebGlTexture> {
    gl.get_parameter(target.gl_enum())
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_binding_2d(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_binding_3d(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_3D)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_binding_2d_array(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D_ARRAY)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_binding_cube_map(gl: &WebGl2RenderingContext) -> Option<WebGlTexture> {
    gl.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_CUBE_MAP)
        .unwrap()
        .cast_into_unchecked::<WebGlTexture>()
}

pub fn texture_pixel_storage_pack_alignment(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::PACK_ALIGNMENT)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_pack_row_length(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::PACK_ROW_LENGTH)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_pack_skip_pixels(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::PACK_SKIP_PIXELS)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_pack_skip_rows(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::PACK_SKIP_ROWS)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_alignment(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_ALIGNMENT)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_flip_y_webgl(gl: &WebGl2RenderingContext) -> Option<bool> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
        .map(|v| v != 0)
}

pub fn texture_pixel_storage_unpack_premultiply_alpha_webgl(
    gl: &WebGl2RenderingContext,
) -> Option<bool> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
        .map(|v| v != 0)
}

pub fn texture_pixel_storage_unpack_colorspace_conversion_webgl(
    gl: &WebGl2RenderingContext,
) -> Option<TextureUnpackColorSpaceConversion> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
        .map(|v| {
            if v == WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL {
                TextureUnpackColorSpaceConversion::BROWSER_DEFAULT_WEBGL
            } else {
                TextureUnpackColorSpaceConversion::NONE
            }
        })
}

pub fn texture_pixel_storage_unpack_row_length(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_ROW_LENGTH)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_image_height(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_skip_pixels(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_PIXELS)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_skip_rows(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_ROWS)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_pixel_storage_unpack_skip_images(gl: &WebGl2RenderingContext) -> Option<i32> {
    gl.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_IMAGES)
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn texture_parameter_base_level(
    gl: &WebGl2RenderingContext,
    target: TextureTarget,
) -> Option<i32> {
    gl.get_tex_parameter(target.gl_enum(), WebGl2RenderingContext::TEXTURE_BASE_LEVEL)
        .as_f64()
        .map(|v| v as i32)
}

pub fn texture_parameter_max_level(
    gl: &WebGl2RenderingContext,
    target: TextureTarget,
) -> Option<i32> {
    gl.get_tex_parameter(target.gl_enum(), WebGl2RenderingContext::TEXTURE_MAX_LEVEL)
        .as_f64()
        .map(|v| v as i32)
}

pub fn texture_parameter_max_anisotropy(
    gl: &WebGl2RenderingContext,
    target: TextureTarget,
) -> Option<f32> {
    gl.get_tex_parameter(
        target.gl_enum(),
        ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT,
    )
    .as_f64()
    .map(|v| v as f32)
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
