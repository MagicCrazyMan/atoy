use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebglDebugShaders, WebglLoseContext};

struct Inner {
    gl: WebGl2RenderingContext,

    max_texture_size: Option<usize>,
    max_texture_image_units: Option<usize>,

    color_buffer_float: Option<bool>,
    texture_filter_anisotropic: Option<bool>,
    draw_buffers_indexed: Option<bool>,
    texture_float_linear: Option<bool>,
    debug_renderer_info: Option<bool>,
    debug_shaders: Option<Option<WebglDebugShaders>>,
    lose_context: Option<Option<WebglLoseContext>>,
    compressed_s3tc: Option<bool>,
    compressed_s3tc_srgb: Option<bool>,
    compressed_etc: Option<bool>,
    compressed_pvrtc: Option<bool>,
    compressed_etc1: Option<bool>,
    compressed_astc: Option<bool>,
    compressed_bptc: Option<bool>,
    compressed_rgtc: Option<bool>,
}

#[derive(Clone)]
pub struct Abilities(Rc<RefCell<Inner>>);

impl Abilities {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self(Rc::new(RefCell::new(Inner {
            gl,

            max_texture_size: None,
            max_texture_image_units: None,
            
            color_buffer_float: None,
            texture_filter_anisotropic: None,
            draw_buffers_indexed: None,
            texture_float_linear: None,
            debug_renderer_info: None,
            debug_shaders: None,
            lose_context: None,
            compressed_s3tc: None,
            compressed_s3tc_srgb: None,
            compressed_etc: None,
            compressed_pvrtc: None,
            compressed_etc1: None,
            compressed_astc: None,
            compressed_bptc: None,
            compressed_rgtc: None,
        })))
    }

    pub fn debug_shaders_supported(&self) -> Option<WebglDebugShaders> {
        let mut inner = self.0.borrow_mut();
        if let Some(supported) = inner.debug_shaders.as_ref() {
            return supported.clone();
        }

        let supported = inner
            .gl
            .get_extension("WEBGL_debug_shaders")
            .ok()
            .and_then(|v| v)
            .and_then(|v| v.dyn_into::<WebglDebugShaders>().ok());
        inner.debug_shaders = Some(supported.clone());
        supported
    }

    pub fn lose_context(&self) -> Option<WebglLoseContext> {
        let mut inner = self.0.borrow_mut();
        if let Some(supported) = inner.lose_context.as_ref() {
            return supported.clone();
        }

        let supported = inner
            .gl
            .get_extension("WEBGL_lose_context")
            .ok()
            .and_then(|v| v)
            .and_then(|v| v.dyn_into::<WebglLoseContext>().ok());
        inner.lose_context = Some(supported.clone());
        supported
    }

    pub fn max_texture_size(&self) -> usize {
        let mut inner = self.0.borrow_mut();
        if let Some(size) = inner.max_texture_size {
            return size;
        }

        let size = inner
            .gl
            .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as usize)
            .unwrap();
        inner.max_texture_size = Some(size);
        size
    }

    pub fn max_texture_image_units(&self) -> usize {
        let mut inner = self.0.borrow_mut();
        if let Some(size) = inner.max_texture_image_units {
            return size;
        }

        let size = inner
            .gl
            .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_IMAGE_UNITS)
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as usize)
            .unwrap();
        inner.max_texture_image_units = Some(size);
        size
    }
}

macro_rules! bool_supported {
    ($(($func:ident, $field:ident, $($extensions:tt),+))+) => {
        impl Abilities {
            $(
                pub fn $func(&self) -> bool {
                    let mut inner = self.0.borrow_mut();
                    if let Some(supported) = inner.$field {
                        return supported;
                    }

                    let supported = $(
                        inner.gl.get_extension($extensions)
                        .map(|extension| extension.is_some())
                        .unwrap_or(false)
                    ) || +;
                    inner.$field = Some(supported);
                    supported
                }
            )+
        }
    };
}

bool_supported! {
    (color_buffer_float_supported, color_buffer_float, "EXT_color_buffer_float")
    (texture_filter_anisotropic_supported, texture_filter_anisotropic, "EXT_texture_filter_anisotropic", "MOZ_EXT_texture_filter_anisotropic", "WEBKIT_EXT_texture_filter_anisotropic")
    (draw_buffers_indexed_supported, draw_buffers_indexed, "OES_draw_buffers_indexed")
    (texture_float_linear_supported, texture_float_linear, "OES_texture_float_linear")
    (debug_renderer_info_supported, debug_renderer_info, "WEBGL_debug_renderer_info")
    (compressed_s3tc_supported, compressed_s3tc, "WEBGL_compressed_texture_s3tc", "MOZ_WEBGL_compressed_texture_s3tc", "WEBKIT_WEBGL_compressed_texture_s3tc")
    (compressed_s3tc_srgb_supported, compressed_s3tc_srgb, "WEBGL_compressed_texture_s3tc_srgb")
    (compressed_etc_supported, compressed_etc, "WEBGL_compressed_texture_etc")
    (compressed_pvrtc_supported, compressed_pvrtc, "WEBGL_compressed_texture_pvrtc")
    (compressed_etc1_supported, compressed_etc1, "WEBGL_compressed_texture_etc1")
    (compressed_astc_supported, compressed_astc, "WEBGL_compressed_texture_astc")
    (compressed_bptc_supported, compressed_bptc, "EXT_texture_compression_bptc")
    (compressed_rgtc_supported, compressed_rgtc, "EXT_texture_compression_rgtc")
}
