use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebglDebugShaders, WebglLoseContext};

use super::{
    error::Error,
    texture::{TextureCompressedFormat, TextureInternalFormat, TextureUnit},
};

struct Inner {
    gl: WebGl2RenderingContext,

    max_texture_size: Option<usize>,
    max_cube_map_texture_size: Option<usize>,
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
pub struct Capabilities(Rc<RefCell<Inner>>);

impl Capabilities {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self(Rc::new(RefCell::new(Inner {
            gl,

            max_texture_size: None,
            max_cube_map_texture_size: None,
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
}

macro_rules! usize_parameters {
    ($(($func:ident, $field:ident, $pname:expr))+) => {
        impl Capabilities {
            $(
                pub fn $func(&self) -> usize {
                    let mut inner = self.0.borrow_mut();
                    if let Some(size) = inner.$field {
                        return size;
                    }

                    let size = inner
                        .gl
                        .get_parameter($pname)
                        .ok()
                        .and_then(|v| v.as_f64())
                        .map(|v| v as usize)
                        .unwrap();
                    inner.$field = Some(size);
                    size
                }
            )+
        }
    };
}

usize_parameters! {
    (max_texture_size, max_texture_size, WebGl2RenderingContext::MAX_TEXTURE_SIZE)
    (max_texture_image_units, max_texture_image_units, WebGl2RenderingContext::MAX_TEXTURE_IMAGE_UNITS)
    (max_cube_map_texture_size, max_cube_map_texture_size, WebGl2RenderingContext::MAX_CUBE_MAP_TEXTURE_SIZE)

}

macro_rules! extensions_supported {
    ($(($func:ident, $field:ident, $($extensions:tt),+))+) => {
        impl Capabilities {
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

extensions_supported! {
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

impl Capabilities {
    pub fn verify_texture_size(&self, width: usize, height: usize) -> Result<(), Error> {
        let max = self.max_texture_size();
        if width > max || height > max {
            return Err(Error::TextureSizeOverflowed {
                max: (max, max),
                value: (width, height),
            });
        }

        Ok(())
    }

    pub fn verify_texture_unit(&self, unit: TextureUnit) -> Result<(), Error> {
        let unit = unit.unit_index() + 1;
        let max = self.max_texture_image_units();
        if unit > max {
            return Err(Error::TextureUnitOverflowed { max, value: unit });
        }

        Ok(())
    }

    pub fn verify_internal_format_uncompressed(
        &self,
        internal_format: TextureInternalFormat,
    ) -> Result<(), Error> {
        match internal_format {
            TextureInternalFormat::R16F
            | TextureInternalFormat::RG16F
            | TextureInternalFormat::RGBA16F
            | TextureInternalFormat::R32F
            | TextureInternalFormat::RG32F
            | TextureInternalFormat::RGBA32F
            | TextureInternalFormat::R11F_G11F_B10F => {
                if self.color_buffer_float_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureInternalFormatUnsupported)
                }
            }
            _ => Ok(()),
        }
    }

    pub fn verify_internal_format_compressed(
        &self,
        compressed_format: TextureCompressedFormat,
    ) -> Result<(), Error> {
        match compressed_format {
            TextureCompressedFormat::RGB_S3TC_DXT1
            | TextureCompressedFormat::RGBA_S3TC_DXT1
            | TextureCompressedFormat::RGBA_S3TC_DXT3
            | TextureCompressedFormat::RGBA_S3TC_DXT5 => {
                if self.compressed_s3tc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::SRGB_S3TC_DXT1
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5 => {
                if self.compressed_s3tc_srgb_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::R11_EAC
            | TextureCompressedFormat::SIGNED_R11_EAC
            | TextureCompressedFormat::RG11_EAC
            | TextureCompressedFormat::SIGNED_RG11_EAC
            | TextureCompressedFormat::RGB8_ETC2
            | TextureCompressedFormat::RGBA8_ETC2_EAC
            | TextureCompressedFormat::SRGB8_ETC2
            | TextureCompressedFormat::SRGB8_ALPHA8_ETC2_EAC
            | TextureCompressedFormat::RGB8_PUNCHTHROUGH_ALPHA1_ETC2
            | TextureCompressedFormat::SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                if self.compressed_etc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::RGB_PVRTC_2BPPV1_IMG
            | TextureCompressedFormat::RGBA_PVRTC_2BPPV1_IMG
            | TextureCompressedFormat::RGB_PVRTC_4BPPV1_IMG
            | TextureCompressedFormat::RGBA_PVRTC_4BPPV1_IMG => {
                if self.compressed_pvrtc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::RGB_ETC1_WEBGL => {
                if self.compressed_etc1_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::RGBA_ASTC_4x4
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_4x4
            | TextureCompressedFormat::RGBA_ASTC_5x4
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x4
            | TextureCompressedFormat::RGBA_ASTC_5x5
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x5
            | TextureCompressedFormat::RGBA_ASTC_6x5
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x5
            | TextureCompressedFormat::RGBA_ASTC_6x6
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x6
            | TextureCompressedFormat::RGBA_ASTC_8x5
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x5
            | TextureCompressedFormat::RGBA_ASTC_8x6
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x6
            | TextureCompressedFormat::RGBA_ASTC_8x8
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x8
            | TextureCompressedFormat::RGBA_ASTC_10x5
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x5
            | TextureCompressedFormat::RGBA_ASTC_10x6
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x6
            | TextureCompressedFormat::RGBA_ASTC_10x10
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x10
            | TextureCompressedFormat::RGBA_ASTC_12x10
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x10
            | TextureCompressedFormat::RGBA_ASTC_12x12
            | TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x12 => {
                if self.compressed_astc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::RGBA_BPTC_UNORM
            | TextureCompressedFormat::SRGB_ALPHA_BPTC_UNORM
            | TextureCompressedFormat::RGB_BPTC_SIGNED_FLOAT
            | TextureCompressedFormat::RGB_BPTC_UNSIGNED_FLOAT => {
                if self.compressed_bptc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
            TextureCompressedFormat::RED_RGTC1
            | TextureCompressedFormat::SIGNED_RED_RGTC1
            | TextureCompressedFormat::RED_GREEN_RGTC2
            | TextureCompressedFormat::SIGNED_RED_GREEN_RGTC2 => {
                if self.compressed_rgtc_supported() {
                    Ok(())
                } else {
                    Err(Error::TextureCompressedFormatUnsupported)
                }
            }
        }
    }
}