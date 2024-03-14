use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlSampler, WebglDebugShaders, WebglLoseContext};

use crate::share::Share;

use super::{
    a::{TextureCompressedFormat, TextureUncompressedInternalFormat, TextureUnit},
    error::Error,
};

struct Inner {
    gl: WebGl2RenderingContext,
    computation_sampler: Option<WebGlSampler>,

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

pub const EXTENSION_WEBGL_DEBUG_SHADERS: &'static str = "WEBGL_debug_shaders";
pub const EXTENSION_WEBGL_LOSE_CONTEXT: &'static str = "WEBGL_lose_context";
pub const EXTENSION_EXT_COLOR_BUFFER_FLOAT: &'static str = "EXT_color_buffer_float";
pub const EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC: &'static str = "EXT_texture_filter_anisotropic";
pub const EXTENSION_MOZ_EXT_TEXTURE_FILTER_ANISOTROPIC: &'static str =
    "MOZ_EXT_texture_filter_anisotropic";
pub const EXTENSION_WEBKIT_EXT_TEXTURE_FILTER_ANISOTROPIC: &'static str =
    "WEBKIT_EXT_texture_filter_anisotropic";
pub const EXTENSION_OES_DRAW_BUFFERS_INDEXED: &'static str = "OES_draw_buffers_indexed";
pub const EXTENSION_OES_TEXTURE_FLOAT_LINEAR: &'static str = "OES_texture_float_linear";
pub const EXTENSION_WEBGL_DEBUG_RENDERER_INFO: &'static str = "WEBGL_debug_renderer_info";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC: &'static str = "WEBGL_compressed_texture_s3tc";
pub const EXTENSION_MOZ_WEBGL_COMPRESSED_TEXTURE_S3TC: &'static str =
    "MOZ_WEBGL_compressed_texture_s3tc";
pub const EXTENSION_WEBKIT_WEBGL_COMPRESSED_TEXTURE_S3TC: &'static str =
    "WEBKIT_WEBGL_compressed_texture_s3tc";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC_SRGB: &'static str =
    "WEBGL_compressed_texture_s3tc_srgb";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC: &'static str = "WEBGL_compressed_texture_etc";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_PVRTC: &'static str = "WEBGL_compressed_texture_pvrtc";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC1: &'static str = "WEBGL_compressed_texture_etc1";
pub const EXTENSION_WEBGL_COMPRESSED_TEXTURE_ASTC: &'static str = "WEBGL_compressed_texture_astc";
pub const EXTENSION_EXT_TEXTURE_COMPRESSION_BPTC: &'static str = "EXT_texture_compression_bptc";
pub const EXTENSION_EXT_TEXTURE_COMPRESSION_RGTC: &'static str = "EXT_texture_compression_rgtc";

#[derive(Clone)]
pub struct Capabilities(Share<Inner>);

impl Capabilities {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self(Rc::new(RefCell::new(Inner {
            gl,
            computation_sampler: None,

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

    pub fn computation_sampler(&self) -> Result<WebGlSampler, Error> {
        let mut inner = self.0.borrow_mut();
        match inner.computation_sampler.as_mut() {
            Some(sampler) => Ok(sampler.clone()),
            None => {
                let sampler = inner
                    .gl
                    .create_sampler()
                    .ok_or(Error::CreateSamplerFailure)?;

                inner.gl.sampler_parameteri(
                    &sampler,
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::NEAREST as i32,
                );
                inner.gl.sampler_parameteri(
                    &sampler,
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::NEAREST as i32,
                );
                inner.gl.sampler_parameteri(
                    &sampler,
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                inner.gl.sampler_parameteri(
                    &sampler,
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                inner.computation_sampler = Some(sampler.clone());
                Ok(sampler)
            }
        }
    }

    pub fn debug_shaders_supported(&self) -> Option<WebglDebugShaders> {
        let mut inner = self.0.borrow_mut();
        if let Some(supported) = inner.debug_shaders.as_ref() {
            return supported.clone();
        }

        let supported = inner
            .gl
            .get_extension(EXTENSION_WEBGL_DEBUG_SHADERS)
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
            .get_extension(EXTENSION_WEBGL_LOSE_CONTEXT)
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
    (color_buffer_float_supported, color_buffer_float, EXTENSION_EXT_COLOR_BUFFER_FLOAT)
    (texture_filter_anisotropic_supported, texture_filter_anisotropic, EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC, EXTENSION_MOZ_EXT_TEXTURE_FILTER_ANISOTROPIC, EXTENSION_WEBKIT_EXT_TEXTURE_FILTER_ANISOTROPIC)
    (draw_buffers_indexed_supported, draw_buffers_indexed, EXTENSION_OES_DRAW_BUFFERS_INDEXED)
    (texture_float_linear_supported, texture_float_linear, EXTENSION_OES_TEXTURE_FLOAT_LINEAR)
    (debug_renderer_info_supported, debug_renderer_info, EXTENSION_WEBGL_DEBUG_RENDERER_INFO)
    (compressed_s3tc_supported, compressed_s3tc, EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC, EXTENSION_MOZ_WEBGL_COMPRESSED_TEXTURE_S3TC, EXTENSION_WEBKIT_WEBGL_COMPRESSED_TEXTURE_S3TC)
    (compressed_s3tc_srgb_supported, compressed_s3tc_srgb, EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC_SRGB)
    (compressed_etc_supported, compressed_etc, EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC)
    (compressed_pvrtc_supported, compressed_pvrtc, EXTENSION_WEBGL_COMPRESSED_TEXTURE_PVRTC)
    (compressed_etc1_supported, compressed_etc1, EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC1)
    (compressed_astc_supported, compressed_astc, EXTENSION_WEBGL_COMPRESSED_TEXTURE_ASTC)
    (compressed_bptc_supported, compressed_bptc, EXTENSION_EXT_TEXTURE_COMPRESSION_BPTC)
    (compressed_rgtc_supported, compressed_rgtc, EXTENSION_EXT_TEXTURE_COMPRESSION_RGTC)
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
        let unit = (unit.unit_index() + 1) as usize;
        let max = self.max_texture_image_units();
        if unit > max {
            return Err(Error::TextureUnitOverflowed { max, value: unit });
        }

        Ok(())
    }

    pub fn verify_internal_format_uncompressed(
        &self,
        internal_format: TextureUncompressedInternalFormat,
    ) -> Result<(), Error> {
        match internal_format {
            TextureUncompressedInternalFormat::R16F
            | TextureUncompressedInternalFormat::RG16F
            | TextureUncompressedInternalFormat::RGBA16F
            | TextureUncompressedInternalFormat::R32F
            | TextureUncompressedInternalFormat::RG32F
            | TextureUncompressedInternalFormat::RGBA32F
            | TextureUncompressedInternalFormat::R11F_G11F_B10F => {
                if self.color_buffer_float_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_EXT_COLOR_BUFFER_FLOAT,
                    ))
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
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC,
                    ))
                }
            }
            TextureCompressedFormat::SRGB_S3TC_DXT1
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3
            | TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5 => {
                if self.compressed_s3tc_srgb_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_S3TC_SRGB,
                    ))
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
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC,
                    ))
                }
            }
            TextureCompressedFormat::RGB_PVRTC_2BPPV1_IMG
            | TextureCompressedFormat::RGBA_PVRTC_2BPPV1_IMG
            | TextureCompressedFormat::RGB_PVRTC_4BPPV1_IMG
            | TextureCompressedFormat::RGBA_PVRTC_4BPPV1_IMG => {
                if self.compressed_pvrtc_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_PVRTC,
                    ))
                }
            }
            TextureCompressedFormat::RGB_ETC1_WEBGL => {
                if self.compressed_etc1_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_ETC1,
                    ))
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
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_WEBGL_COMPRESSED_TEXTURE_ASTC,
                    ))
                }
            }
            TextureCompressedFormat::RGBA_BPTC_UNORM
            | TextureCompressedFormat::SRGB_ALPHA_BPTC_UNORM
            | TextureCompressedFormat::RGB_BPTC_SIGNED_FLOAT
            | TextureCompressedFormat::RGB_BPTC_UNSIGNED_FLOAT => {
                if self.compressed_bptc_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_EXT_TEXTURE_COMPRESSION_BPTC,
                    ))
                }
            }
            TextureCompressedFormat::RED_RGTC1
            | TextureCompressedFormat::SIGNED_RED_RGTC1
            | TextureCompressedFormat::RED_GREEN_RGTC2
            | TextureCompressedFormat::SIGNED_RED_GREEN_RGTC2 => {
                if self.compressed_rgtc_supported() {
                    Ok(())
                } else {
                    Err(Error::ExtensionUnsupported(
                        EXTENSION_EXT_TEXTURE_COMPRESSION_RGTC,
                    ))
                }
            }
        }
    }
}
