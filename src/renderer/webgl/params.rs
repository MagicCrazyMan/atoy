use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    ExtTextureFilterAnisotropic, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer,
    WebGlRenderbuffer, WebGlSampler, WebGlTexture,
};

use super::{
    texture::{TextureTarget, TextureUnpackColorSpaceConversion},
    conversion::ToGlEnum,
};

pub trait GetWebGlParameters {
    fn array_buffer_binding(&self) -> Option<WebGlBuffer>;

    fn uniform_buffer_binding(&self) -> Option<WebGlBuffer>;

    fn pixel_unpack_buffer_binding(&self) -> Option<WebGlBuffer>;

    fn renderbuffer_binding(&self) -> Option<WebGlRenderbuffer>;

    fn framebuffer_binding(&self) -> Option<WebGlFramebuffer>;

    fn texture_active_texture_unit(&self) -> u32;

    fn sampler_binding(&self) -> Option<WebGlSampler>;

    fn texture_binding(&self, target: TextureTarget) -> Option<WebGlTexture>;

    fn texture_binding_2d(&self) -> Option<WebGlTexture>;

    fn texture_binding_3d(&self) -> Option<WebGlTexture>;

    fn texture_binding_2d_array(&self) -> Option<WebGlTexture>;

    fn texture_binding_cube_map(&self) -> Option<WebGlTexture>;

    fn texture_pixel_storage_pack_alignment(&self) -> Option<i32>;

    fn texture_pixel_storage_pack_row_length(&self) -> Option<i32>;

    fn texture_pixel_storage_pack_skip_pixels(&self) -> Option<i32>;

    fn texture_pixel_storage_pack_skip_rows(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_alignment(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_flip_y(&self) -> Option<bool>;

    fn texture_pixel_storage_unpack_premultiply_alpha(&self) -> Option<bool>;

    fn texture_pixel_storage_unpack_colorspace_conversion(
        &self,
    ) -> Option<TextureUnpackColorSpaceConversion>;

    fn texture_pixel_storage_unpack_row_length(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_image_height(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_skip_pixels(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_skip_rows(&self) -> Option<i32>;

    fn texture_pixel_storage_unpack_skip_images(&self) -> Option<i32>;

    fn texture_parameter_base_level(&self, target: TextureTarget) -> Option<i32>;

    fn texture_parameter_max_level(&self, target: TextureTarget) -> Option<i32>;

    fn texture_parameter_max_anisotropy(&self, target: TextureTarget) -> Option<f32>;
}

impl GetWebGlParameters for WebGl2RenderingContext {
    fn array_buffer_binding(&self) -> Option<WebGlBuffer> {
        self.get_parameter(WebGl2RenderingContext::ARRAY_BUFFER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlBuffer>()
    }

    fn uniform_buffer_binding(&self) -> Option<WebGlBuffer> {
        self.get_parameter(WebGl2RenderingContext::UNIFORM_BUFFER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlBuffer>()
    }

    fn pixel_unpack_buffer_binding(&self) -> Option<WebGlBuffer> {
        self.get_parameter(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlBuffer>()
    }

    fn renderbuffer_binding(&self) -> Option<WebGlRenderbuffer> {
        self.get_parameter(WebGl2RenderingContext::RENDERBUFFER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlRenderbuffer>()
    }

    fn framebuffer_binding(&self) -> Option<WebGlFramebuffer> {
        self.get_parameter(WebGl2RenderingContext::FRAMEBUFFER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlFramebuffer>()
    }

    fn texture_active_texture_unit(&self) -> u32 {
        self.get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
            .ok()
            .map(|v| v.as_f64().unwrap())
            .map(|v| v as u32)
            .unwrap()
    }

    fn sampler_binding(&self) -> Option<WebGlSampler> {
        self.get_parameter(WebGl2RenderingContext::SAMPLER_BINDING)
            .unwrap()
            .cast_into_unchecked::<WebGlSampler>()
    }

    fn texture_binding(&self, target: TextureTarget) -> Option<WebGlTexture> {
        self.get_parameter(target.to_gl_enum())
            .unwrap()
            .cast_into_unchecked::<WebGlTexture>()
    }

    fn texture_binding_2d(&self) -> Option<WebGlTexture> {
        self.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)
            .unwrap()
            .cast_into_unchecked::<WebGlTexture>()
    }

    fn texture_binding_3d(&self) -> Option<WebGlTexture> {
        self.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_3D)
            .unwrap()
            .cast_into_unchecked::<WebGlTexture>()
    }

    fn texture_binding_2d_array(&self) -> Option<WebGlTexture> {
        self.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D_ARRAY)
            .unwrap()
            .cast_into_unchecked::<WebGlTexture>()
    }

    fn texture_binding_cube_map(&self) -> Option<WebGlTexture> {
        self.get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_CUBE_MAP)
            .unwrap()
            .cast_into_unchecked::<WebGlTexture>()
    }

    fn texture_pixel_storage_pack_alignment(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::PACK_ALIGNMENT)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_pack_row_length(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::PACK_ROW_LENGTH)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_pack_skip_pixels(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::PACK_SKIP_PIXELS)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_pack_skip_rows(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::PACK_SKIP_ROWS)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_alignment(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_ALIGNMENT)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_flip_y(&self) -> Option<bool> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
            .map(|v| v != 0)
    }

    fn texture_pixel_storage_unpack_premultiply_alpha(&self) -> Option<bool> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
            .map(|v| v != 0)
    }

    fn texture_pixel_storage_unpack_colorspace_conversion(
        &self,
    ) -> Option<TextureUnpackColorSpaceConversion> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL)
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

    fn texture_pixel_storage_unpack_row_length(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_ROW_LENGTH)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_image_height(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_skip_pixels(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_PIXELS)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_skip_rows(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_ROWS)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_pixel_storage_unpack_skip_images(&self) -> Option<i32> {
        self.get_parameter(WebGl2RenderingContext::UNPACK_SKIP_IMAGES)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as i32)
    }

    fn texture_parameter_base_level(&self, target: TextureTarget) -> Option<i32> {
        self.get_tex_parameter(target.to_gl_enum(), WebGl2RenderingContext::TEXTURE_BASE_LEVEL)
            .as_f64()
            .map(|v| v as i32)
    }

    fn texture_parameter_max_level(&self, target: TextureTarget) -> Option<i32> {
        self.get_tex_parameter(target.to_gl_enum(), WebGl2RenderingContext::TEXTURE_MAX_LEVEL)
            .as_f64()
            .map(|v| v as i32)
    }

    fn texture_parameter_max_anisotropy(&self, target: TextureTarget) -> Option<f32> {
        self.get_tex_parameter(
            target.to_gl_enum(),
            ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT,
        )
        .as_f64()
        .map(|v| v as f32)
    }
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
