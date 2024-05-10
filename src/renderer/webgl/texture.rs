use std::{
    borrow::Cow,
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use js_sys::{
    DataView, Float32Array, Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
use log::warn;
use uuid::Uuid;
use web_sys::{
    ExtTextureFilterAnisotropic, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement,
    ImageBitmap, ImageData, WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use crate::lru::{Lru, LruNode};

use super::{
    capabilities::Capabilities, conversion::ToGlEnum, error::Error, params::GetWebGlParameters,
};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureTarget {
    TEXTURE_2D,
    TEXTURE_CUBE_MAP,
    TEXTURE_2D_ARRAY,
    TEXTURE_3D,
}

/// Available texture units mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnit {
    TEXTURE0,
    TEXTURE1,
    TEXTURE2,
    TEXTURE3,
    TEXTURE4,
    TEXTURE5,
    TEXTURE6,
    TEXTURE7,
    TEXTURE8,
    TEXTURE9,
    TEXTURE10,
    TEXTURE11,
    TEXTURE12,
    TEXTURE13,
    TEXTURE14,
    TEXTURE15,
    TEXTURE16,
    TEXTURE17,
    TEXTURE18,
    TEXTURE19,
    TEXTURE20,
    TEXTURE21,
    TEXTURE22,
    TEXTURE23,
    TEXTURE24,
    TEXTURE25,
    TEXTURE26,
    TEXTURE27,
    TEXTURE28,
    TEXTURE29,
    TEXTURE30,
    TEXTURE31,
}

impl TextureUnit {
    pub fn unit_index(&self) -> u32 {
        match self {
            TextureUnit::TEXTURE0 => 0,
            TextureUnit::TEXTURE1 => 1,
            TextureUnit::TEXTURE2 => 2,
            TextureUnit::TEXTURE3 => 3,
            TextureUnit::TEXTURE4 => 4,
            TextureUnit::TEXTURE5 => 5,
            TextureUnit::TEXTURE6 => 6,
            TextureUnit::TEXTURE7 => 7,
            TextureUnit::TEXTURE8 => 8,
            TextureUnit::TEXTURE9 => 9,
            TextureUnit::TEXTURE10 => 10,
            TextureUnit::TEXTURE11 => 11,
            TextureUnit::TEXTURE12 => 12,
            TextureUnit::TEXTURE13 => 13,
            TextureUnit::TEXTURE14 => 14,
            TextureUnit::TEXTURE15 => 15,
            TextureUnit::TEXTURE16 => 16,
            TextureUnit::TEXTURE17 => 17,
            TextureUnit::TEXTURE18 => 18,
            TextureUnit::TEXTURE19 => 19,
            TextureUnit::TEXTURE20 => 20,
            TextureUnit::TEXTURE21 => 21,
            TextureUnit::TEXTURE22 => 22,
            TextureUnit::TEXTURE23 => 23,
            TextureUnit::TEXTURE24 => 24,
            TextureUnit::TEXTURE25 => 25,
            TextureUnit::TEXTURE26 => 26,
            TextureUnit::TEXTURE27 => 27,
            TextureUnit::TEXTURE28 => 28,
            TextureUnit::TEXTURE29 => 29,
            TextureUnit::TEXTURE30 => 30,
            TextureUnit::TEXTURE31 => 31,
        }
    }
}

/// Available texture pixel data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUncompressedPixelDataType {
    FLOAT,
    HALF_FLOAT,
    BYTE,
    SHORT,
    INT,
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
    UNSIGNED_SHORT_5_6_5,
    UNSIGNED_SHORT_4_4_4_4,
    UNSIGNED_SHORT_5_5_5_1,
    UNSIGNED_INT_2_10_10_10_REV,
    UNSIGNED_INT_10F_11F_11F_REV,
    UNSIGNED_INT_5_9_9_9_REV,
    UNSIGNED_INT_24_8,
    FLOAT_32_UNSIGNED_INT_24_8_REV,
}

/// Available texture unpack color space conversions for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackColorSpaceConversion {
    NONE,
    BROWSER_DEFAULT_WEBGL,
}

/// Available texture pixel storages for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePixelStorage {
    PACK_ALIGNMENT(i32),
    PACK_ROW_LENGTH(i32),
    PACK_SKIP_PIXELS(i32),
    PACK_SKIP_ROWS(i32),
    UNPACK_ALIGNMENT(i32),
    UNPACK_FLIP_Y_WEBGL(bool),
    UNPACK_PREMULTIPLY_ALPHA_WEBGL(bool),
    UNPACK_COLORSPACE_CONVERSION_WEBGL(TextureUnpackColorSpaceConversion),
    UNPACK_ROW_LENGTH(i32),
    UNPACK_IMAGE_HEIGHT(i32),
    UNPACK_SKIP_PIXELS(i32),
    UNPACK_SKIP_ROWS(i32),
    UNPACK_SKIP_IMAGES(i32),
}

impl TexturePixelStorage {
    fn set(&self, gl: &WebGl2RenderingContext) {
        match self {
            TexturePixelStorage::PACK_ALIGNMENT(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, *v);
            }
            TexturePixelStorage::PACK_ROW_LENGTH(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, *v);
            }
            TexturePixelStorage::PACK_SKIP_PIXELS(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, *v);
            }
            TexturePixelStorage::PACK_SKIP_ROWS(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, *v);
            }
            TexturePixelStorage::UNPACK_ALIGNMENT(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, *v);
            }
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
                    if *v { 1 } else { 0 },
                );
            }
            TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                    if *v { 1 } else { 0 },
                );
            }
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    match v {
                        TextureUnpackColorSpaceConversion::NONE => {
                            WebGl2RenderingContext::NONE as i32
                        }
                        TextureUnpackColorSpaceConversion::BROWSER_DEFAULT_WEBGL => {
                            WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32
                        }
                    },
                );
            }
            TexturePixelStorage::UNPACK_ROW_LENGTH(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, *v);
            }
            TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, *v);
            }
            TexturePixelStorage::UNPACK_SKIP_PIXELS(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, *v);
            }
            TexturePixelStorage::UNPACK_SKIP_ROWS(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, *v);
            }
            TexturePixelStorage::UNPACK_SKIP_IMAGES(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, *v);
            }
        }
    }

    fn reset(&self, gl: &WebGl2RenderingContext) {
        match self {
            TexturePixelStorage::PACK_ALIGNMENT(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, 4);
            }
            TexturePixelStorage::PACK_ROW_LENGTH(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, 0);
            }
            TexturePixelStorage::PACK_SKIP_PIXELS(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, 0);
            }
            TexturePixelStorage::PACK_SKIP_ROWS(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, 0);
            }
            TexturePixelStorage::UNPACK_ALIGNMENT(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 4);
            }
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
            }
            TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
            }
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(_) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
                );
            }
            TexturePixelStorage::UNPACK_ROW_LENGTH(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, 0);
            }
            TexturePixelStorage::UNPACK_IMAGE_HEIGHT(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, 0);
            }
            TexturePixelStorage::UNPACK_SKIP_PIXELS(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, 0);
            }
            TexturePixelStorage::UNPACK_SKIP_ROWS(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, 0);
            }
            TexturePixelStorage::UNPACK_SKIP_IMAGES(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, 0);
            }
        }
    }
}

/// Available texture magnification filters for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMagnificationFilter {
    LINEAR,
    NEAREST,
}

/// Available texture minification filters for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMinificationFilter {
    LINEAR,
    NEAREST,
    NEAREST_MIPMAP_NEAREST,
    LINEAR_MIPMAP_NEAREST,
    NEAREST_MIPMAP_LINEAR,
    LINEAR_MIPMAP_LINEAR,
}

/// Available texture wrap methods for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapMethod {
    REPEAT,
    CLAMP_TO_EDGE,
    MIRRORED_REPEAT,
}

/// Available texture compare function for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareFunction {
    LEQUAL,
    GEQUAL,
    LESS,
    GREATER,
    EQUAL,
    NOTEQUAL,
    ALWAYS,
    NEVER,
}

/// Available texture compare modes for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareMode {
    NONE,
    COMPARE_REF_TO_TEXTURE,
}

/// Available texture parameter kinds mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameterKind {
    BASE_LEVEL,
    MAX_LEVEL,
    /// Available when extension `EXT_texture_filter_anisotropic` enabled.
    MAX_ANISOTROPY,
}

/// Available texture parameters mapped from [`WebGl2RenderingContext`].
///
/// Different from WebGL1, WebGL2 separates sampling parameters to a new object called [`WebGlSampler`],
/// those sampling parameters are no more included in this enum. Checks [`SamplerParameter`] for more details.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameter {
    BASE_LEVEL(i32),
    MAX_LEVEL(i32),
    /// Available when extension `EXT_texture_filter_anisotropic` enabled.
    MAX_ANISOTROPY(f32),
}

impl TextureParameter {
    /// Returns texture kind.
    pub fn kind(&self) -> TextureParameterKind {
        match self {
            TextureParameter::BASE_LEVEL(_) => TextureParameterKind::BASE_LEVEL,
            TextureParameter::MAX_LEVEL(_) => TextureParameterKind::MAX_LEVEL,
            TextureParameter::MAX_ANISOTROPY(_) => TextureParameterKind::MAX_ANISOTROPY,
        }
    }

    fn set(&self, gl: &WebGl2RenderingContext, target: TextureTarget, capabilities: &Capabilities) {
        match self {
            TextureParameter::BASE_LEVEL(v) => {
                gl.tex_parameteri(
                    target.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    *v,
                );
            }
            TextureParameter::MAX_LEVEL(v) => {
                gl.tex_parameteri(
                    target.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
                    *v,
                );
            }
            TextureParameter::MAX_ANISOTROPY(v) => {
                if !capabilities.texture_filter_anisotropic_supported() {
                    warn!("EXT_texture_filter_anisotropic unsupported");
                    return;
                }
                gl.tex_parameterf(
                    target.gl_enum(),
                    ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT,
                    *v,
                );
            }
        };
    }
}

/// Available sampling kinds for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerParameterKind {
    MAG_FILTER,
    MIN_FILTER,
    WRAP_S,
    WRAP_T,
    WRAP_R,
    COMPARE_FUNC,
    COMPARE_MODE,
    MAX_LOD,
    MIN_LOD,
}

/// Available sampling parameters for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplerParameter {
    MAG_FILTER(TextureMagnificationFilter),
    MIN_FILTER(TextureMinificationFilter),
    WRAP_S(TextureWrapMethod),
    WRAP_T(TextureWrapMethod),
    WRAP_R(TextureWrapMethod),
    COMPARE_FUNC(TextureCompareFunction),
    COMPARE_MODE(TextureCompareMode),
    MAX_LOD(f32),
    MIN_LOD(f32),
}

impl SamplerParameter {
    /// Returns sampler kind.
    pub fn kind(&self) -> SamplerParameterKind {
        match self {
            SamplerParameter::MAG_FILTER(_) => SamplerParameterKind::MAG_FILTER,
            SamplerParameter::MIN_FILTER(_) => SamplerParameterKind::MIN_FILTER,
            SamplerParameter::WRAP_S(_) => SamplerParameterKind::WRAP_S,
            SamplerParameter::WRAP_T(_) => SamplerParameterKind::WRAP_T,
            SamplerParameter::WRAP_R(_) => SamplerParameterKind::WRAP_R,
            SamplerParameter::COMPARE_FUNC(_) => SamplerParameterKind::COMPARE_FUNC,
            SamplerParameter::COMPARE_MODE(_) => SamplerParameterKind::COMPARE_MODE,
            SamplerParameter::MAX_LOD(_) => SamplerParameterKind::MAX_LOD,
            SamplerParameter::MIN_LOD(_) => SamplerParameterKind::MIN_LOD,
        }
    }

    fn set(&self, gl: &WebGl2RenderingContext, sampler: &WebGlSampler) {
        match self {
            SamplerParameter::MAG_FILTER(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::MIN_FILTER(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::WRAP_S(v)
            | SamplerParameter::WRAP_T(v)
            | SamplerParameter::WRAP_R(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::COMPARE_FUNC(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::COMPARE_MODE(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::MAX_LOD(v) | SamplerParameter::MIN_LOD(v) => {
                gl.sampler_parameterf(&sampler, self.gl_enum(), *v)
            }
        }
    }
}

/// Available texture formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUncompressedPixelFormat {
    RED,
    RED_INTEGER,
    RG,
    RG_INTEGER,
    RGB,
    RGB_INTEGER,
    RGBA,
    RGBA_INTEGER,
    LUMINANCE,
    LUMINANCE_ALPHA,
    ALPHA,
    DEPTH_COMPONENT,
    DEPTH_STENCIL,
}

/// Available texture color internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUncompressedInternalFormat {
    RGBA32I,
    RGBA32UI,
    RGBA16I,
    RGBA16UI,
    RGBA8,
    RGBA8I,
    RGBA8UI,
    SRGB8_ALPHA8,
    RGB10_A2,
    RGB10_A2UI,
    RGBA4,
    RGB5_A1,
    RGB8,
    RGB565,
    RG32I,
    RG32UI,
    RG16I,
    RG16UI,
    RG8,
    RG8I,
    RG8UI,
    R32I,
    R32UI,
    R16I,
    R16UI,
    R8,
    R8I,
    R8UI,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RGBA32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RGBA16F,
    RGBA8_SNORM,
    RGB32F,
    RGB32I,
    RGB32UI,
    RGB16F,
    RGB16I,
    RGB16UI,
    RGB8_SNORM,
    RGB8I,
    RGB8UI,
    SRGB8,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R11F_G11F_B10F,
    RGB9_E5,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RG32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RG16F,
    RG8_SNORM,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R16F,
    R8_SNORM,
    DEPTH_COMPONENT32F,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT16,
    DEPTH32F_STENCIL8,
    DEPTH24_STENCIL8,
}

impl TextureUncompressedInternalFormat {
    /// Calculates the bytes length of a specified internal format with specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureUncompressedInternalFormat::RGBA32I => width * height * 16,
            TextureUncompressedInternalFormat::RGBA32UI => width * height * 16,
            TextureUncompressedInternalFormat::RGBA16I => width * height * 4,
            TextureUncompressedInternalFormat::RGBA16UI => width * height * 4,
            TextureUncompressedInternalFormat::RGBA8 => width * height * 4,
            TextureUncompressedInternalFormat::RGBA8I => width * height * 4,
            TextureUncompressedInternalFormat::RGBA8UI => width * height * 4,
            TextureUncompressedInternalFormat::SRGB8_ALPHA8 => width * height * 4,
            TextureUncompressedInternalFormat::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureUncompressedInternalFormat::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureUncompressedInternalFormat::RGBA4 => width * height * 2,
            TextureUncompressedInternalFormat::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
            TextureUncompressedInternalFormat::RGB8 => width * height * 3,
            TextureUncompressedInternalFormat::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
            TextureUncompressedInternalFormat::RG32I => width * height * 4,
            TextureUncompressedInternalFormat::RG32UI => width * height * 4,
            TextureUncompressedInternalFormat::RG16I => width * height * 4,
            TextureUncompressedInternalFormat::RG16UI => width * height * 4,
            TextureUncompressedInternalFormat::RG8 => width * height * 2,
            TextureUncompressedInternalFormat::RG8I => width * height * 2,
            TextureUncompressedInternalFormat::RG8UI => width * height * 2,
            TextureUncompressedInternalFormat::R32I => width * height * 4,
            TextureUncompressedInternalFormat::R32UI => width * height * 4,
            TextureUncompressedInternalFormat::R16I => width * height * 2,
            TextureUncompressedInternalFormat::R16UI => width * height * 2,
            TextureUncompressedInternalFormat::R8 => width * height * 1,
            TextureUncompressedInternalFormat::R8I => width * height * 1,
            TextureUncompressedInternalFormat::R8UI => width * height * 1,
            TextureUncompressedInternalFormat::RGBA32F => width * height * 16,
            TextureUncompressedInternalFormat::RGBA16F => width * height * 4,
            TextureUncompressedInternalFormat::RGBA8_SNORM => width * height * 4,
            TextureUncompressedInternalFormat::RGB32F => width * height * 12,
            TextureUncompressedInternalFormat::RGB32I => width * height * 12,
            TextureUncompressedInternalFormat::RGB32UI => width * height * 12,
            TextureUncompressedInternalFormat::RGB16F => width * height * 6,
            TextureUncompressedInternalFormat::RGB16I => width * height * 6,
            TextureUncompressedInternalFormat::RGB16UI => width * height * 6,
            TextureUncompressedInternalFormat::RGB8_SNORM => width * height * 3,
            TextureUncompressedInternalFormat::RGB8I => width * height * 3,
            TextureUncompressedInternalFormat::RGB8UI => width * height * 3,
            TextureUncompressedInternalFormat::SRGB8 => width * height * 3,
            TextureUncompressedInternalFormat::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
            TextureUncompressedInternalFormat::RGB9_E5 => width * height * 4, // 9 + 9 + 9 + 5 in bits
            TextureUncompressedInternalFormat::RG32F => width * height * 4,
            TextureUncompressedInternalFormat::RG16F => width * height * 4,
            TextureUncompressedInternalFormat::RG8_SNORM => width * height * 2,
            TextureUncompressedInternalFormat::R32F => width * height * 4,
            TextureUncompressedInternalFormat::R16F => width * height * 2,
            TextureUncompressedInternalFormat::R8_SNORM => width * height * 1,
            TextureUncompressedInternalFormat::DEPTH_COMPONENT32F => width * height * 4,
            TextureUncompressedInternalFormat::DEPTH_COMPONENT24 => width * height * 3,
            TextureUncompressedInternalFormat::DEPTH_COMPONENT16 => width * height * 2,
            TextureUncompressedInternalFormat::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
            TextureUncompressedInternalFormat::DEPTH24_STENCIL8 => width * height * 4,
        }
    }
}

/// Available texture compressed internal and upload formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompressedFormat {
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SIGNED_R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SIGNED_RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGBA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_ALPHA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGB_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGBA_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGB_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGBA_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_etc1` enabled.
    RGB_ETC1_WEBGL,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_12x12,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_12x12,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGBA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    SRGB_ALPHA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGB_BPTC_SIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGB_BPTC_UNSIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    SIGNED_RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    RED_GREEN_RGTC2,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    SIGNED_RED_GREEN_RGTC2,
}

impl TextureCompressedFormat {
    /// Calculates the bytes length of a specified internal format with specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            // for S3TC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc/ for more details
            TextureCompressedFormat::RGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::RGBA_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::RGBA_S3TC_DXT3 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::RGBA_S3TC_DXT5 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            // for S3TC RGBA, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc_srgb/ for more details
            TextureCompressedFormat::SRGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 8
            }
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            // for ETC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc/ for more details
            TextureCompressedFormat::R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::SIGNED_R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SIGNED_RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::RGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::SRGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::RGBA8_ETC2_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ETC2_EAC => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::RGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 8
            }
            TextureCompressedFormat::SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 8
            }
            // for PVRTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_pvrtc/ for more details
            TextureCompressedFormat::RGB_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            TextureCompressedFormat::RGBA_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            TextureCompressedFormat::RGB_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            TextureCompressedFormat::RGBA_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            // for ETC1, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc1/ for more details
            TextureCompressedFormat::RGB_ETC1_WEBGL => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            // for ASTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_astc/ for more details
            TextureCompressedFormat::RGBA_ASTC_4x4 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_4x4 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_5x4 => ((width + 4) / 5) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x4 => {
                ((width + 4) / 5) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_5x5 => ((width + 4) / 5) * ((height + 4) / 5) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x5 => {
                ((width + 4) / 5) * ((height + 4) / 5) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_6x5 => ((width + 5) / 6) * ((height + 4) / 5) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x5 => {
                ((width + 5) / 6) * ((height + 4) / 5) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_6x6 => ((width + 5) / 6) * ((height + 5) / 6) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x6 => {
                ((width + 5) / 6) * ((height + 5) / 6) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_8x5 => ((width + 7) / 8) * ((height + 4) / 5) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x5 => {
                ((width + 7) / 8) * ((height + 4) / 5) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_8x6 => ((width + 7) / 8) * ((height + 5) / 6) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x6 => {
                ((width + 7) / 8) * ((height + 5) / 6) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_8x8 => ((width + 7) / 8) * ((height + 7) / 8) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x8 => {
                ((width + 7) / 8) * ((height + 7) / 8) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_10x5 => ((width + 9) / 10) * ((height + 4) / 5) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x5 => {
                ((width + 9) / 10) * ((height + 4) / 5) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_10x6 => ((width + 9) / 10) * ((height + 5) / 6) * 16,
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x6 => {
                ((width + 9) / 10) * ((height + 5) / 6) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_10x10 => {
                ((width + 9) / 10) * ((height + 9) / 10) * 16
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x10 => {
                ((width + 9) / 10) * ((height + 9) / 10) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_12x10 => {
                ((width + 11) / 12) * ((height + 9) / 10) * 16
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x10 => {
                ((width + 11) / 12) * ((height + 9) / 10) * 16
            }
            TextureCompressedFormat::RGBA_ASTC_12x12 => {
                ((width + 11) / 12) * ((height + 11) / 12) * 16
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x12 => {
                ((width + 11) / 12) * ((height + 11) / 12) * 16
            }
            // for BPTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_bptc/ for more details
            TextureCompressedFormat::RGBA_BPTC_UNORM => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SRGB_ALPHA_BPTC_UNORM => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::RGB_BPTC_SIGNED_FLOAT => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            TextureCompressedFormat::RGB_BPTC_UNSIGNED_FLOAT => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
            // for RGTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_rgtc/ for more details
            TextureCompressedFormat::RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::SIGNED_RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            TextureCompressedFormat::RED_GREEN_RGTC2 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            TextureCompressedFormat::SIGNED_RED_GREEN_RGTC2 => {
                ((width + 3) / 4) * ((height + 3) / 4) * 16
            }
        }
    }
}

/// Texture internal formats containing both uncompressed and compressed formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureInternalFormat {
    Uncompressed(TextureUncompressedInternalFormat),
    Compressed(TextureCompressedFormat),
}

impl TextureInternalFormat {
    /// Calculates the bytes length of a specified internal format with specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureInternalFormat::Uncompressed(format) => format.byte_length(width, height),
            TextureInternalFormat::Compressed(format) => format.byte_length(width, height),
        }
    }
}

pub enum TextureUncompressedData<'a> {
    Bytes {
        width: usize,
        height: usize,
        data: Box<dyn AsRef<[u8]>>,
        src_element_offset: Option<usize>,
    },
    BytesBorrowed {
        width: usize,
        height: usize,
        data: &'a [u8],
        src_element_offset: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        pbo_offset: Option<usize>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        src_element_offset: Option<usize>,
    },
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        src_element_offset: Option<usize>,
    },
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
    },
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        src_element_offset: Option<usize>,
    },
    Uint16Array {
        width: usize,
        height: usize,
        data: Uint16Array,
        src_element_offset: Option<usize>,
    },
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        src_element_offset: Option<usize>,
    },
    Uint32Array {
        width: usize,
        height: usize,
        data: Uint32Array,
        src_element_offset: Option<usize>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        src_element_offset: Option<usize>,
    },
    DataView {
        width: usize,
        height: usize,
        data: DataView,
        src_element_offset: Option<usize>,
    },
    HtmlCanvasElement {
        data: HtmlCanvasElement,
    },
    HtmlImageElement {
        data: HtmlImageElement,
    },
    HtmlVideoElement {
        data: HtmlVideoElement,
    },
    ImageData {
        data: ImageData,
    },
    ImageBitmap {
        data: ImageBitmap,
    },
}

pub enum TextureCompressedData<'a> {
    Bytes {
        width: usize,
        height: usize,
        data: Box<dyn AsMut<[u8]>>,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    BytesBorrowed {
        width: usize,
        height: usize,
        data: &'a mut [u8],
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        image_size: usize,
        pbo_offset: Option<usize>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Uint16Array {
        width: usize,
        height: usize,
        data: Uint16Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Uint32Array {
        width: usize,
        height: usize,
        data: Uint32Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
    DataView {
        width: usize,
        height: usize,
        data: DataView,
        src_element_offset: Option<usize>,
        src_element_length_override: Option<usize>,
    },
}

/// Texture data for uploading data to WebGL runtime.
pub enum TextureData<'a> {
    Uncompressed {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        data: TextureUncompressedData<'a>,
    },
    Compressed {
        pixel_format: TextureCompressedFormat,
        data: TextureCompressedData<'a>,
    },
}

impl<'a> TextureData<'a> {
    fn width(&self) -> usize {
        match self {
            TextureData::Uncompressed { data, .. } => match data {
                TextureUncompressedData::Bytes { width, .. }
                | TextureUncompressedData::BytesBorrowed { width, .. }
                | TextureUncompressedData::PixelBufferObject { width, .. }
                | TextureUncompressedData::Int8Array { width, .. }
                | TextureUncompressedData::Uint8Array { width, .. }
                | TextureUncompressedData::Uint8ClampedArray { width, .. }
                | TextureUncompressedData::Int16Array { width, .. }
                | TextureUncompressedData::Uint16Array { width, .. }
                | TextureUncompressedData::Int32Array { width, .. }
                | TextureUncompressedData::Uint32Array { width, .. }
                | TextureUncompressedData::Float32Array { width, .. }
                | TextureUncompressedData::DataView { width, .. } => *width,
                TextureUncompressedData::HtmlCanvasElement { data, .. } => data.width() as usize,
                TextureUncompressedData::HtmlImageElement { data, .. } => {
                    data.natural_width() as usize
                }
                TextureUncompressedData::HtmlVideoElement { data, .. } => {
                    data.video_width() as usize
                }
                TextureUncompressedData::ImageData { data, .. } => data.width() as usize,
                TextureUncompressedData::ImageBitmap { data, .. } => data.width() as usize,
            },
            TextureData::Compressed { data, .. } => match data {
                TextureCompressedData::Bytes { width, .. }
                | TextureCompressedData::BytesBorrowed { width, .. }
                | TextureCompressedData::PixelBufferObject { width, .. }
                | TextureCompressedData::Int8Array { width, .. }
                | TextureCompressedData::Uint8Array { width, .. }
                | TextureCompressedData::Uint8ClampedArray { width, .. }
                | TextureCompressedData::Int16Array { width, .. }
                | TextureCompressedData::Uint16Array { width, .. }
                | TextureCompressedData::Int32Array { width, .. }
                | TextureCompressedData::Uint32Array { width, .. }
                | TextureCompressedData::Float32Array { width, .. }
                | TextureCompressedData::DataView { width, .. } => *width,
            },
        }
    }

    fn height(&self) -> usize {
        match self {
            TextureData::Uncompressed { data, .. } => match data {
                TextureUncompressedData::Bytes { height, .. }
                | TextureUncompressedData::BytesBorrowed { height, .. }
                | TextureUncompressedData::PixelBufferObject { height, .. }
                | TextureUncompressedData::Int8Array { height, .. }
                | TextureUncompressedData::Uint8Array { height, .. }
                | TextureUncompressedData::Uint8ClampedArray { height, .. }
                | TextureUncompressedData::Int16Array { height, .. }
                | TextureUncompressedData::Uint16Array { height, .. }
                | TextureUncompressedData::Int32Array { height, .. }
                | TextureUncompressedData::Uint32Array { height, .. }
                | TextureUncompressedData::Float32Array { height, .. }
                | TextureUncompressedData::DataView { height, .. } => *height,
                TextureUncompressedData::HtmlCanvasElement { data, .. } => data.height() as usize,
                TextureUncompressedData::HtmlImageElement { data, .. } => {
                    data.natural_height() as usize
                }
                TextureUncompressedData::HtmlVideoElement { data, .. } => {
                    data.video_height() as usize
                }
                TextureUncompressedData::ImageData { data, .. } => data.height() as usize,
                TextureUncompressedData::ImageBitmap { data, .. } => data.height() as usize,
            },
            TextureData::Compressed { data, .. } => match data {
                TextureCompressedData::Bytes { height, .. }
                | TextureCompressedData::BytesBorrowed { height, .. }
                | TextureCompressedData::PixelBufferObject { height, .. }
                | TextureCompressedData::Int8Array { height, .. }
                | TextureCompressedData::Uint8Array { height, .. }
                | TextureCompressedData::Uint8ClampedArray { height, .. }
                | TextureCompressedData::Int16Array { height, .. }
                | TextureCompressedData::Uint16Array { height, .. }
                | TextureCompressedData::Int32Array { height, .. }
                | TextureCompressedData::Uint32Array { height, .. }
                | TextureCompressedData::Float32Array { height, .. }
                | TextureCompressedData::DataView { height, .. } => *height,
            },
        }
    }
}

pub trait TextureSource {
    fn data(&self) -> TextureData;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture2DArray;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture3D;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureCubeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCubeMapFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextureLayout {
    target: TextureTarget,
    internal_format: TextureInternalFormat,
    levels: usize,
    width: usize,
    height: usize,
    depth: usize,
}

impl TextureLayout {
    fn target(&self) -> TextureTarget {
        self.target
    }

    fn internal_format(&self) -> TextureInternalFormat {
        self.internal_format
    }

    fn byte_length(&self) -> usize {
        match self.target {
            TextureTarget::TEXTURE_2D => (0..self.levels)
                .map(|level| {
                    let width = (self.width >> level).max(1);
                    let height = (self.height >> level).max(1);
                    self.internal_format.byte_length(width, height)
                })
                .sum::<usize>(),
            TextureTarget::TEXTURE_2D_ARRAY => (0..self.levels)
                .map(|level| {
                    let width = (self.width >> level).max(1);
                    let height = (self.height >> level).max(1);
                    self.internal_format.byte_length(width, height) * self.depth
                })
                .sum::<usize>(),
            TextureTarget::TEXTURE_3D => (0..self.levels)
                .map(|level| {
                    let width = (self.width >> level).max(1);
                    let height = (self.height >> level).max(1);
                    let depth = (self.depth >> level).max(1);
                    self.internal_format.byte_length(width, height) * depth
                })
                .sum::<usize>(),
            TextureTarget::TEXTURE_CUBE_MAP => (0..self.levels)
                .map(|level| {
                    let width = (self.width >> level).max(1);
                    let height = (self.height >> level).max(1);
                    self.internal_format.byte_length(width, height) * 6
                })
                .sum::<usize>(),
        }
    }
}

struct QueueItem {
    source: Box<dyn TextureSource>,
    target: TextureTarget,
    cube_map_face: Option<TextureCubeMapFace>,
    generate_mipmaps: bool,
    level: usize,
    x_offset: Option<usize>,
    y_offset: Option<usize>,
    z_offset: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    depth: Option<usize>,
}

struct TextureRuntime {
    gl: WebGl2RenderingContext,
    capabilities: Capabilities,
    byte_length: usize,
    texture: Option<(WebGlTexture, WebGlSampler)>,
    bindings: HashSet<TextureUnit>,
}

impl TextureRuntime {
    fn get_or_create_texture(
        &mut self,
        layout: &TextureLayout,
        texture_params: &[TextureParameter],
        sampler_params: &[SamplerParameter],
        registered: Option<&mut TextureRegistered>,
    ) -> Result<(WebGlTexture, WebGlSampler), Error> {
        match self.texture.as_ref() {
            Some((texture, sampler)) => Ok((texture.clone(), sampler.clone())),
            None => {
                if !self
                    .capabilities
                    .internal_format_supported(layout.internal_format)
                {
                    return Err(Error::TextureInternalFormatUnsupported(
                        layout.internal_format,
                    ));
                }

                let texture = self
                    .gl
                    .create_texture()
                    .ok_or(Error::CreateTextureFailure)?;
                let sampler = self
                    .gl
                    .create_sampler()
                    .ok_or(Error::CreateSamplerFailure)?;

                let target = layout.target();
                let binding = if cfg!(feature = "rebind") {
                    self.gl.texture_binding(target)
                } else {
                    None
                };

                self.gl.bind_texture(target.gl_enum(), Some(&texture));

                // sets sampler parameters
                for param in sampler_params {
                    param.set(&self.gl, &sampler);
                }
                // sets texture parameters
                for param in texture_params {
                    param.set(&self.gl, target, &self.capabilities);
                }
                match target {
                    TextureTarget::TEXTURE_2D | TextureTarget::TEXTURE_CUBE_MAP => {
                        self.gl.tex_storage_2d(
                            target.gl_enum(),
                            layout.levels as i32,
                            layout.internal_format.gl_enum(),
                            layout.width as i32,
                            layout.height as i32,
                        )
                    }
                    TextureTarget::TEXTURE_2D_ARRAY | TextureTarget::TEXTURE_3D => {
                        self.gl.tex_storage_3d(
                            target.gl_enum(),
                            layout.levels as i32,
                            layout.internal_format.gl_enum(),
                            layout.width as i32,
                            layout.height as i32,
                            layout.depth as i32,
                        )
                    }
                }

                self.gl.bind_texture(target.gl_enum(), binding.as_ref());
                self.byte_length = layout.byte_length();

                let (texture, sampler) = self.texture.insert((texture, sampler));

                if let Some(registered) = registered {
                    if let Some(store) = registered.store.upgrade() {
                        store.borrow_mut().increase_used_memory(self.byte_length);
                    }
                }

                Ok((texture.clone(), sampler.clone()))
            }
        }
    }

    fn upload(&self, layout: &TextureLayout, queue: &mut Vec<QueueItem>) -> Result<(), Error> {
        let internal_format = layout.internal_format();
        for QueueItem {
            source,
            target,
            cube_map_face,
            generate_mipmaps,
            level,
            x_offset,
            y_offset,
            z_offset,
            width,
            height,
            depth,
        } in queue.drain(..)
        {
            let is_3d = match target {
                TextureTarget::TEXTURE_2D | TextureTarget::TEXTURE_CUBE_MAP => false,
                TextureTarget::TEXTURE_2D_ARRAY | TextureTarget::TEXTURE_3D => true,
            };
            let target = if target == TextureTarget::TEXTURE_CUBE_MAP {
                // unwrap_or should never reach
                match cube_map_face.unwrap_or(TextureCubeMapFace::PositiveX) {
                    TextureCubeMapFace::PositiveX => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X
                    }
                    TextureCubeMapFace::NegativeX => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X
                    }
                    TextureCubeMapFace::PositiveY => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y
                    }
                    TextureCubeMapFace::NegativeY => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y
                    }
                    TextureCubeMapFace::PositiveZ => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z
                    }
                    TextureCubeMapFace::NegativeZ => {
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z
                    }
                }
            } else {
                target.gl_enum()
            };
            let data = source.data();
            let level = level as i32;
            let x_offset = x_offset.unwrap_or(0) as i32;
            let y_offset = y_offset.unwrap_or(0) as i32;
            let z_offset = z_offset.unwrap_or(0) as i32;
            let width = width.unwrap_or(data.width()) as i32;
            let height = height.unwrap_or(data.height()) as i32;
            let depth = depth.unwrap_or(0) as i32;

            match (data, internal_format) {
                (
                    TextureData::Uncompressed {
                        data,
                        pixel_format,
                        pixel_storages,
                        pixel_data_type,
                    },
                    TextureInternalFormat::Uncompressed(_),
                ) => {
                    for storage in &pixel_storages {
                        storage.set(&self.gl);
                    }

                    let result = match data {
                        TextureUncompressedData::Bytes { .. }
                        | TextureUncompressedData::BytesBorrowed { .. } => {
                            enum Data<'a> {
                                Borrowed(&'a [u8]),
                                Owned(Box<dyn AsRef<[u8]>>),
                            }

                            impl<'a> Data<'a> {
                                fn as_bytes(&self) -> &[u8] {
                                    match self {
                                        Data::Borrowed(data) => *data,
                                        Data::Owned(data) => data.as_ref().as_ref(),
                                    }
                                }
                            }

                            let (data, data_type, src_element_offset) = match data {
                                TextureUncompressedData::Bytes {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Data::Owned(data), pixel_data_type, src_element_offset),
                                TextureUncompressedData::BytesBorrowed {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Data::Borrowed(data), pixel_data_type, src_element_offset),
                                _ => unreachable!(),
                            };

                            if is_3d {
                                self.gl.tex_sub_image_3d_with_opt_u8_array_and_src_offset(
                                    target,
                                    level,
                                    x_offset,
                                    y_offset,
                                    z_offset,
                                    width,
                                    height,
                                    depth,
                                    pixel_format.gl_enum(),
                                    data_type.gl_enum(),
                                    Some(data.as_bytes()),
                                    src_element_offset.unwrap_or(0) as u32,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                                    target,
                                    level,
                                    x_offset,
                                    y_offset,
                                    width,
                                    height,
                                    pixel_format.gl_enum(),
                                    data_type.gl_enum(),
                                    data.as_bytes(),
                                    src_element_offset.unwrap_or(0) as u32,
                                )
                            }
                        }
                        TextureUncompressedData::PixelBufferObject {
                            buffer, pbo_offset, ..
                        } => {
                            let binding = if cfg!(feature = "rebind") {
                                self.gl.pixel_unpack_buffer_binding()
                            } else {
                                None
                            };

                            self.gl.bind_buffer(
                                WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                                Some(&buffer),
                            );
                            let result = if is_3d {
                                self.gl.tex_sub_image_3d_with_i32(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    pbo_offset.unwrap_or(0) as i32,
                                )
                            } else {
                                self.gl
                                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                                        target,
                                        level as i32,
                                        x_offset as i32,
                                        y_offset as i32,
                                        width as i32,
                                        height as i32,
                                        pixel_format.gl_enum(),
                                        pixel_data_type.gl_enum(),
                                        pbo_offset.unwrap_or(0) as i32,
                                    )
                            };

                            self.gl.bind_buffer(
                                WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                                binding.as_ref(),
                            );
                            result
                        }
                        TextureUncompressedData::Int8Array { .. }
                        | TextureUncompressedData::Uint8Array { .. }
                        | TextureUncompressedData::Uint8ClampedArray { .. }
                        | TextureUncompressedData::Int16Array { .. }
                        | TextureUncompressedData::Uint16Array { .. }
                        | TextureUncompressedData::Int32Array { .. }
                        | TextureUncompressedData::Uint32Array { .. }
                        | TextureUncompressedData::Float32Array { .. }
                        | TextureUncompressedData::DataView { .. } => {
                            let (data, src_element_offset) = match data {
                                TextureUncompressedData::Int8Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Uint8Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Uint8ClampedArray {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Int16Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Uint16Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Int32Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Uint32Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::Float32Array {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                TextureUncompressedData::DataView {
                                    data,
                                    src_element_offset,
                                    ..
                                } => (Object::from(data), src_element_offset),
                                _ => unreachable!(),
                            };

                            if is_3d {
                                self.gl
                                    .tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                                        target,
                                        level as i32,
                                        x_offset as i32,
                                        y_offset as i32,
                                        z_offset as i32,
                                        width as i32,
                                        height as i32,
                                        depth as i32,
                                        pixel_format.gl_enum(),
                                        pixel_data_type.gl_enum(),
                                        Some(&data),
                                        src_element_offset.unwrap_or(0) as u32,
                                    )
                            } else {
                                self.gl
                                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                                        target,
                                        level as i32,
                                        x_offset as i32,
                                        y_offset as i32,
                                        width as i32,
                                        height as i32,
                                        pixel_format.gl_enum(),
                                        pixel_data_type.gl_enum(),
                                        &data,
                                        src_element_offset.unwrap_or(0) as u32,
                                    )
                            }
                        }
                        TextureUncompressedData::HtmlCanvasElement { data } => {
                            if is_3d {
                                self.gl.tex_sub_image_3d_with_html_canvas_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            }
                        }
                        TextureUncompressedData::HtmlImageElement { data } => {
                            if is_3d {
                                self.gl.tex_sub_image_3d_with_html_image_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            }
                        }
                        TextureUncompressedData::HtmlVideoElement { data } => {
                            if is_3d {
                                self.gl.tex_sub_image_3d_with_html_video_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            }
                        }
                        TextureUncompressedData::ImageData { data } => {
                            if is_3d {
                                self.gl.tex_sub_image_3d_with_image_data(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            }
                        }
                        TextureUncompressedData::ImageBitmap { data } => {
                            if is_3d {
                                self.gl.tex_sub_image_3d_with_image_bitmap(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            } else {
                                self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    pixel_data_type.gl_enum(),
                                    &data,
                                )
                            }
                        }
                    };

                    if let Err(err) = result {
                        return Err(Error::TextureUploadImageFailure(err.as_string()));
                    }

                    if generate_mipmaps {
                        self.gl.generate_mipmap(target);
                    }

                    for storage in &pixel_storages {
                        storage.reset(&self.gl);
                    }
                }
                (
                    TextureData::Compressed { data, pixel_format },
                    TextureInternalFormat::Compressed(internal_format),
                ) => {
                    if pixel_format != internal_format {
                        return Err(Error::TextureInternalFormatMismatched);
                    }

                    match data {
                        TextureCompressedData::Bytes { .. }
                        | TextureCompressedData::BytesBorrowed { .. } => {
                            enum Data<'a> {
                                Borrowed(&'a mut [u8]),
                                Owned(Box<dyn AsMut<[u8]>>),
                            }

                            impl<'a> Data<'a> {
                                fn as_bytes(&mut self) -> &mut [u8] {
                                    match self {
                                        Data::Borrowed(data) => *data,
                                        Data::Owned(data) => data.as_mut().as_mut(),
                                    }
                                }
                            }

                            let (mut data, src_element_offset, src_element_length_override) =
                                match data {
                                    TextureCompressedData::Bytes {
                                        data,
                                        src_element_offset,
                                        src_element_length_override,
                                        ..
                                    } => (
                                        Data::Owned(data),
                                        src_element_offset,
                                        src_element_length_override,
                                    ),
                                    TextureCompressedData::BytesBorrowed {
                                        data,
                                        src_element_offset,
                                        src_element_length_override,
                                        ..
                                    } => (
                                        Data::Borrowed(data),
                                        src_element_offset,
                                        src_element_length_override,
                                    ),
                                    _ => unreachable!(),
                                };

                            if is_3d {
                                self.gl.compressed_tex_sub_image_3d_with_u8_array_and_u32_and_src_length_override(
                                    target,
                                    level,
                                    x_offset,
                                    y_offset,
                                    z_offset,
                                    width,
                                    height,
                                    depth,
                                    pixel_format.gl_enum(),
                                    data.as_bytes(),
                                    src_element_offset.unwrap_or(0) as u32,
                                    src_element_length_override.unwrap_or(0) as u32,
                                )
                            } else {
                                self.gl.compressed_tex_sub_image_2d_with_u8_array_and_u32_and_src_length_override(
                                    target,
                                    level,
                                    x_offset,
                                    y_offset,
                                    width,
                                    height,
                                    pixel_format.gl_enum(),
                                    data.as_bytes(),
                                    src_element_offset.unwrap_or(0) as u32,
                                    src_element_length_override.unwrap_or(0) as u32,
                                )
                            }
                        }
                        TextureCompressedData::PixelBufferObject {
                            width,
                            height,
                            buffer,
                            image_size,
                            pbo_offset,
                        } => {
                            let binding = if cfg!(feature = "rebind") {
                                self.gl.pixel_unpack_buffer_binding()
                            } else {
                                None
                            };

                            self.gl.bind_buffer(
                                WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                                Some(&buffer),
                            );
                            if is_3d {
                                self.gl.compressed_tex_sub_image_3d_with_i32_and_i32(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    image_size as i32,
                                    pbo_offset.unwrap_or(0) as i32,
                                )
                            } else {
                                self.gl.compressed_tex_sub_image_2d_with_i32_and_i32(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    image_size as i32,
                                    pbo_offset.unwrap_or(0) as i32,
                                )
                            };

                            self.gl.bind_buffer(
                                WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                                binding.as_ref(),
                            );
                        }
                        TextureCompressedData::Int8Array { .. }
                        | TextureCompressedData::Uint8Array { .. }
                        | TextureCompressedData::Uint8ClampedArray { .. }
                        | TextureCompressedData::Int16Array { .. }
                        | TextureCompressedData::Uint16Array { .. }
                        | TextureCompressedData::Int32Array { .. }
                        | TextureCompressedData::Uint32Array { .. }
                        | TextureCompressedData::Float32Array { .. }
                        | TextureCompressedData::DataView { .. } => {
                            let (
                                width,
                                height,
                                data,
                                src_element_offset,
                                src_element_length_override,
                            ) = match data {
                                TextureCompressedData::Int8Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Uint8Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Uint8ClampedArray {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Int16Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Uint16Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Int32Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Uint32Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::Float32Array {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                TextureCompressedData::DataView {
                                    width,
                                    height,
                                    data,
                                    src_element_offset,
                                    src_element_length_override,
                                } => (
                                    width,
                                    height,
                                    Object::from(data),
                                    src_element_offset,
                                    src_element_length_override,
                                ),
                                _ => unreachable!(),
                            };

                            if is_3d {
                                self.gl.compressed_tex_sub_image_3d_with_array_buffer_view_and_u32_and_src_length_override(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    z_offset as i32,
                                    width as i32,
                                    height as i32,
                                    depth as i32,
                                    pixel_format.gl_enum(),
                                    &data,
                                    src_element_offset.unwrap_or(0) as u32,
                                    src_element_length_override.unwrap_or(0) as u32,
                                )
                            } else {
                                self.gl.compressed_tex_sub_image_2d_with_array_buffer_view_and_u32_and_src_length_override(
                                    target,
                                    level as i32,
                                    x_offset as i32,
                                    y_offset as i32,
                                    width as i32,
                                    height as i32,
                                    pixel_format.gl_enum(),
                                    &data,
                                    src_element_offset.unwrap_or(0) as u32,
                                    src_element_length_override.unwrap_or(0) as u32,
                                )
                            }
                        }
                    }
                }
                (TextureData::Uncompressed { .. }, TextureInternalFormat::Compressed(_))
                | (TextureData::Compressed { .. }, TextureInternalFormat::Uncompressed(_)) => {
                    return Err(Error::TextureInternalFormatMismatched);
                }
            };
        }

        Ok(())
    }
}

struct TextureRegistered {
    store: Weak<RefCell<StoreShared>>,
    lru_node: *mut LruNode<Uuid>,
}

struct TextureShared {
    id: Uuid,
    name: Option<Cow<'static, str>>,
    layout: TextureLayout,
    sampler_params: Vec<SamplerParameter>,
    texture_params: Vec<TextureParameter>,
    memory_policy: MemoryPolicyShared,
    queue: Vec<QueueItem>,
    registered: Option<TextureRegistered>,
    runtime: Option<TextureRuntime>,
}

impl Drop for TextureShared {
    fn drop(&mut self) {
        if let Some(mut runtime) = self.runtime.take() {
            if let Some((texture, sampler)) = runtime.texture.take() {
                let target = self.layout.target;

                let activing = if cfg!(feature = "rebind") {
                    Some(runtime.gl.texture_active_texture_unit())
                } else {
                    None
                };

                for unit in runtime.bindings.iter() {
                    runtime.gl.active_texture(unit.gl_enum());
                    runtime.gl.bind_sampler(unit.unit_index(), None);
                    runtime.gl.bind_texture(target.gl_enum(), None);
                }

                runtime.gl.delete_texture(Some(&texture));
                runtime.gl.delete_sampler(Some(&sampler));

                if let Some(activing) = activing {
                    runtime.gl.active_texture(activing);
                }
            }

            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    store.borrow_mut().unregister(
                        &self.id,
                        registered.lru_node,
                        runtime.byte_length,
                        self.layout.target,
                        runtime.bindings.iter(),
                    );
                }
            }
        }
    }
}

impl Debug for TextureShared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("layout", &self.layout)
            .field("id", &self.id)
            .field("name", &self.name)
            .field("texture_params", &self.texture_params)
            .field("sampler_params", &self.sampler_params)
            .finish()
    }
}

impl TextureShared {
    fn init(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if let Some(runtime) = self.runtime.as_ref() {
            if &runtime.gl != gl {
                return Err(Error::TextureAlreadyInitialized);
            } else {
                return Ok(());
            }
        }

        self.runtime = Some(TextureRuntime {
            capabilities: Capabilities::new(gl.clone()),
            gl: gl.clone(),
            byte_length: 0,
            texture: None,
            bindings: HashSet::new(),
        });
        Ok(())
    }

    fn bind(&mut self, unit: TextureUnit) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

        let target = self.layout.target();

        if let Some(registered) = self.registered.as_mut() {
            if let Some(store) = registered.store.upgrade() {
                if store.borrow().is_occupied(unit, target, &self.id) {
                    return Err(Error::TextureTargetOccupied(unit, target));
                }
            }
        }

        if runtime.bindings.contains(&unit) {
            runtime.upload(&self.layout, &mut self.queue)?;

            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    let mut store = store.borrow_mut();
                    store.update_lru(registered.lru_node);
                    store.free();
                }
            }
        } else {
            let (texture, sampler) = runtime.get_or_create_texture(
                &self.layout,
                &self.texture_params,
                &self.sampler_params,
                self.registered.as_mut(),
            )?;
            let active_texture_unit = if cfg!(feature = "rebind") {
                Some(runtime.gl.texture_active_texture_unit())
            } else {
                None
            };

            runtime.gl.active_texture(unit.gl_enum());
            runtime.gl.bind_texture(target.gl_enum(), Some(&texture));
            runtime.gl.bind_sampler(unit.unit_index(), Some(&sampler));
            runtime.upload(&self.layout, &mut self.queue)?;
            runtime.bindings.insert(unit);

            if let Some(unit) = active_texture_unit {
                runtime.gl.active_texture(unit);
            }

            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    let mut store = store.borrow_mut();
                    store.add_binding(unit, target, self.id);
                    store.update_lru(registered.lru_node);
                    store.free();
                }
            }
        }

        Ok(())
    }

    fn unbind(&mut self, unit: TextureUnit) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

        let target = self.layout.target();
        if runtime.bindings.remove(&unit) {
            let active_texture_unit = if cfg!(feature = "rebind") {
                Some(runtime.gl.texture_active_texture_unit())
            } else {
                None
            };

            runtime.gl.active_texture(unit.gl_enum());
            runtime.gl.bind_texture(target.gl_enum(), None);
            runtime.gl.bind_sampler(unit.unit_index(), None);

            if let Some(unit) = active_texture_unit {
                runtime.gl.active_texture(unit);
            }

            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    store.borrow_mut().remove_binding(unit, target);
                }
            }
        }

        Ok(())
    }

    fn unbind_all(&mut self) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

        let active_texture_unit = if cfg!(feature = "rebind") {
            Some(runtime.gl.texture_active_texture_unit())
        } else {
            None
        };

        let target = self.layout.target();
        for unit in runtime.bindings.drain() {
            runtime.gl.active_texture(unit.unit_index());
            runtime.gl.bind_texture(target.gl_enum(), None);

            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    store.borrow_mut().remove_binding(unit, target);
                }
            }
        }

        if let Some(unit) = active_texture_unit {
            runtime.gl.active_texture(unit);
        }

        Ok(())
    }

    fn upload(&mut self) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

        let (texture, _) = runtime.get_or_create_texture(
            &self.layout,
            &self.texture_params,
            &self.sampler_params,
            self.registered.as_mut(),
        )?;
        let target = self.layout.target();
        let binding = if cfg!(feature = "rebind") {
            runtime.gl.texture_binding(target)
        } else {
            None
        };
        runtime.gl.bind_texture(target.gl_enum(), Some(&texture));
        runtime.upload(&self.layout, &mut self.queue)?;
        runtime.gl.bind_texture(target.gl_enum(), binding.as_ref());

        Ok(())
    }

    fn set_texture_parameter(&mut self, texture_param: TextureParameter) {
        let index = self
            .texture_params
            .iter()
            .position(|p| p.kind() == texture_param.kind());
        match index {
            Some(index) => {
                let _ = std::mem::replace::<TextureParameter>(
                    &mut self.texture_params[index],
                    texture_param,
                );
            }
            None => self.texture_params.push(texture_param),
        };

        if let Some(runtime) = self.runtime.as_ref() {
            if let Some((texture, _)) = runtime.texture.as_ref() {
                let target = self.layout.target();
                let binding = if cfg!(feature = "rebind") {
                    runtime.gl.texture_binding(target)
                } else {
                    None
                };

                runtime.gl.bind_texture(target.gl_enum(), Some(texture));
                texture_param.set(&runtime.gl, target, &runtime.capabilities);

                runtime.gl.bind_texture(target.gl_enum(), binding.as_ref());
            }
        }
    }

    fn set_texture_parameters<I>(&mut self, texture_params: I)
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        texture_params
            .into_iter()
            .for_each(|param| self.set_texture_parameter(param))
    }

    fn set_sampler_parameter(&mut self, sampler_param: SamplerParameter) {
        let index = self
            .sampler_params
            .iter()
            .position(|p| p.kind() == sampler_param.kind());
        match index {
            Some(index) => {
                let _ = std::mem::replace::<SamplerParameter>(
                    &mut self.sampler_params[index],
                    sampler_param,
                );
            }
            None => self.sampler_params.push(sampler_param),
        };

        if let Some(runtime) = self.runtime.as_ref() {
            if let Some((_, sampler)) = runtime.texture.as_ref() {
                sampler_param.set(&runtime.gl, &sampler);
            }
        }
    }

    fn set_sampler_parameters<I>(&mut self, sampler_params: I)
    where
        I: IntoIterator<Item = SamplerParameter>,
    {
        sampler_params
            .into_iter()
            .for_each(|param| self.set_sampler_parameter(param))
    }

    fn free(&mut self) -> bool {
        let Some(runtime) = self.runtime.as_mut() else {
            return false;
        };

        // skips if using
        if runtime.bindings.len() != 0 {
            return false;
        }

        let queue = match &self.memory_policy {
            MemoryPolicyShared::Texture2D(memory_policy) => match memory_policy {
                MemoryPolicy::Unfree => return false,
                MemoryPolicy::Restorable(restorer) => {
                    let mut restore = RestoreReceiver::<Texture2D>::new();
                    restorer.restore(&mut restore);
                    restore.queue
                }
            },
            MemoryPolicyShared::Texture2DArray(memory_policy) => match memory_policy {
                MemoryPolicy::Unfree => return false,
                MemoryPolicy::Restorable(restorer) => {
                    let mut restore = RestoreReceiver::<Texture2DArray>::new();
                    restorer.restore(&mut restore);
                    restore.queue
                }
            },
            MemoryPolicyShared::Texture3D(memory_policy) => match memory_policy {
                MemoryPolicy::Unfree => return false,
                MemoryPolicy::Restorable(restorer) => {
                    let mut restore = RestoreReceiver::<Texture3D>::new();
                    restorer.restore(&mut restore);
                    restore.queue
                }
            },
            MemoryPolicyShared::TextureCubeMap(memory_policy) => match memory_policy {
                MemoryPolicy::Unfree => return false,
                MemoryPolicy::Restorable(restorer) => {
                    let mut builder = RestoreReceiver::<TextureCubeMap>::new();
                    restorer.restore(&mut builder);
                    builder.queue
                }
            },
        };

        if let Some((texture, sampler)) = runtime.texture.take() {
            if let Some(registered) = self.registered.as_mut() {
                if let Some(store) = registered.store.upgrade() {
                    store
                        .borrow_mut()
                        .remove(runtime.byte_length, registered.lru_node);
                }
            }

            runtime.gl.delete_sampler(Some(&sampler));
            runtime.gl.delete_texture(Some(&texture));
            runtime.byte_length = 0;
        }

        self.queue = queue.into_iter().chain(self.queue.drain(..)).collect();

        true
    }
}

#[derive(Debug, Clone)]
pub struct TextureUnbinder {
    unit: TextureUnit,
    shared: Weak<RefCell<TextureShared>>,
}

impl TextureUnbinder {
    /// Unbinds texture.
    pub fn unbind(self) {
        let Some(shared) = self.shared.upgrade() else {
            return;
        };
        let _ = shared.borrow_mut().unbind(self.unit);
    }
}

#[derive(Debug, Clone)]
pub struct Texture<L> {
    layout: PhantomData<L>,
    shared: Rc<RefCell<TextureShared>>,
}

impl<L> Texture<L> {
    /// Returns id of this buffer.
    pub fn id(&self) -> Uuid {
        self.shared.borrow().id
    }

    /// Sets name.
    pub fn set_name(&self, name: Option<Cow<'static, str>>) {
        self.shared.borrow_mut().name = name;
    }

    /// Returns name.
    pub fn name(&self) -> Option<String> {
        match self.shared.borrow().name.as_ref() {
            Some(name) => Some(name.to_string()),
            None => None,
        }
    }

    /// Initializes texture.
    pub fn init(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        self.shared.borrow_mut().init(gl)
    }

    /// Binds texture to specified target in specified texture unit.
    pub fn bind(&self, unit: TextureUnit) -> Result<TextureUnbinder, Error> {
        self.shared.borrow_mut().bind(unit)?;
        Ok(TextureUnbinder {
            unit,
            shared: Rc::downgrade(&self.shared),
        })
    }

    /// Unbinds texture from specified target in specified texture unit.
    pub fn unbind(&self, unit: TextureUnit) -> Result<(), Error> {
        self.shared.borrow_mut().unbind(unit)
    }

    /// Unbinds texture from all bound texture unit.
    pub fn unbind_all(&self) -> Result<(), Error> {
        self.shared.borrow_mut().unbind_all()
    }

    /// Uploads texture data to WebGL runtime.
    pub fn upload(&self) -> Result<(), Error> {
        self.shared.borrow_mut().upload()
    }

    /// Returns a list of texture parameters.
    pub fn texture_parameters(&self) -> Vec<TextureParameter> {
        self.shared.borrow().texture_params.clone()
    }

    /// Sets texture parameter.
    pub fn set_texture_parameter(&self, texture_param: TextureParameter) {
        self.shared
            .borrow_mut()
            .set_texture_parameter(texture_param)
    }

    /// Sets texture parameters.
    pub fn set_texture_parameters<I>(&self, texture_params: I)
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        self.shared
            .borrow_mut()
            .set_texture_parameters(texture_params)
    }

    /// Returns a list of sampler parameters.
    pub fn sampler_parameters(&self) -> Vec<SamplerParameter> {
        self.shared.borrow().sampler_params.clone()
    }

    /// Sets sampler parameter.
    pub fn set_sampler_parameter(&self, sampler_param: SamplerParameter) {
        self.shared
            .borrow_mut()
            .set_sampler_parameter(sampler_param)
    }

    /// Sets sampler parameters.
    pub fn set_sampler_parameters<I>(&self, sampler_params: I)
    where
        I: IntoIterator<Item = SamplerParameter>,
    {
        self.shared
            .borrow_mut()
            .set_sampler_parameters(sampler_params)
    }
}

impl Texture<Texture2D> {
    /// Creates a new 2d texture.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        texture_params: Vec<TextureParameter>,
        sampler_params: Vec<SamplerParameter>,
        memory_policy: MemoryPolicy<Texture2D>,
    ) -> Self {
        let shared = Rc::new(RefCell::new(TextureShared {
            id: Uuid::new_v4(),
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_2D,
                internal_format,
                levels,
                width,
                height,
                depth: 0,
            },
            memory_policy: MemoryPolicyShared::Texture2D(memory_policy),
            texture_params,
            sampler_params,
            queue: Vec::new(),
            registered: None,
            runtime: None,
        }));

        Self {
            layout: PhantomData,
            shared,
        }
    }

    /// Returns texture target.
    pub fn target(&self) -> TextureTarget {
        self.shared.borrow().layout.target
    }

    /// Returns texture internal format.
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.shared.borrow().layout.internal_format
    }

    /// Returns mipmap levels.
    pub fn levels(&self) -> usize {
        self.shared.borrow().layout.levels
    }

    /// Returns texture width at level 0.
    pub fn width(&self) -> usize {
        self.shared.borrow().layout.width
    }

    /// Returns texture height at level 0.
    pub fn height(&self) -> usize {
        self.shared.borrow().layout.height
    }

    /// Returns texture width at specified level.
    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let width = (layout.width >> level).max(1);
        Some(width)
    }

    /// Returns texture height at specified level.
    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let height = (layout.height >> level).max(1);
        Some(height)
    }

    /// Sets memory policy
    pub fn set_memory_policy(&self, memory_policy: MemoryPolicy<Texture2D>) {
        self.shared.borrow_mut().memory_policy = MemoryPolicyShared::Texture2D(memory_policy);
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }
}

impl Texture<Texture2DArray> {
    /// Creates a new 2d array texture.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        len: usize,
        texture_params: Vec<TextureParameter>,
        sampler_params: Vec<SamplerParameter>,
        memory_policy: MemoryPolicy<Texture2DArray>,
    ) -> Self {
        let shared = Rc::new(RefCell::new(TextureShared {
            id: Uuid::new_v4(),
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_2D_ARRAY,
                internal_format,
                levels,
                width,
                height,
                depth: len,
            },
            memory_policy: MemoryPolicyShared::Texture2DArray(memory_policy),
            texture_params,
            sampler_params,
            queue: Vec::new(),
            registered: None,
            runtime: None,
        }));

        Self {
            layout: PhantomData,
            shared,
        }
    }

    /// Returns texture target.
    pub fn target(&self) -> TextureTarget {
        self.shared.borrow().layout.target
    }

    /// Returns texture internal format.
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.shared.borrow().layout.internal_format
    }

    /// Returns texture mipmap levels.
    pub fn levels(&self) -> usize {
        self.shared.borrow().layout.levels
    }

    /// Returns texture width at level 0.
    pub fn width(&self) -> usize {
        self.shared.borrow().layout.width
    }

    /// Returns texture height at level 0.
    pub fn height(&self) -> usize {
        self.shared.borrow().layout.height
    }

    /// Returns texture array length.
    pub fn len(&self) -> usize {
        self.shared.borrow().layout.depth
    }

    /// Returns texture width at specified level.
    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let width = (layout.width >> level).max(1);
        Some(width)
    }

    /// Returns texture height at specified level.
    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let height = (layout.height >> level).max(1);
        Some(height)
    }

    /// Sets memory policy
    pub fn set_memory_policy(&self, memory_policy: MemoryPolicy<Texture2DArray>) {
        self.shared.borrow_mut().memory_policy = MemoryPolicyShared::Texture2DArray(memory_policy);
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        array_index_offset: usize,
        width: usize,
        height: usize,
        array_index: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(array_index_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(array_index),
        });
    }
}

impl Texture<Texture3D> {
    /// Creates a new 3d texture.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        depth: usize,
        texture_params: Vec<TextureParameter>,
        sampler_params: Vec<SamplerParameter>,
        memory_policy: MemoryPolicy<Texture3D>,
    ) -> Self {
        let shared = Rc::new(RefCell::new(TextureShared {
            id: Uuid::new_v4(),
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_3D,
                internal_format,
                levels,
                width,
                height,
                depth,
            },
            memory_policy: MemoryPolicyShared::Texture3D(memory_policy),
            texture_params,
            sampler_params,
            queue: Vec::new(),
            registered: None,
            runtime: None,
        }));

        Self {
            layout: PhantomData,
            shared,
        }
    }

    /// Returns texture target.
    pub fn target(&self) -> TextureTarget {
        self.shared.borrow().layout.target
    }

    /// Returns texture internal format.
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.shared.borrow().layout.internal_format
    }

    /// Returns mipmap levels.
    pub fn levels(&self) -> usize {
        self.shared.borrow().layout.levels
    }

    /// Returns texture width at level 0.
    pub fn width(&self) -> usize {
        self.shared.borrow().layout.width
    }

    /// Returns texture height at level 0.
    pub fn height(&self) -> usize {
        self.shared.borrow().layout.height
    }

    /// Returns texture depth at level 0.
    pub fn depth(&self) -> usize {
        self.shared.borrow().layout.depth
    }

    /// Returns texture width at specified level.
    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let width = (layout.width >> level).max(1);
        Some(width)
    }

    /// Returns texture height at specified level.
    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let height = (layout.height >> level).max(1);
        Some(height)
    }

    /// Returns texture depth at specified level.
    pub fn depth_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let depth = (layout.depth >> level).max(1);
        Some(depth)
    }

    /// Sets memory policy
    pub fn set_memory_policy(&self, memory_policy: MemoryPolicy<Texture3D>) {
        self.shared.borrow_mut().memory_policy = MemoryPolicyShared::Texture3D(memory_policy);
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        width: usize,
        height: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(z_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(depth),
        });
    }
}

impl Texture<TextureCubeMap> {
    /// Creates a new cube map texture.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        texture_params: Vec<TextureParameter>,
        sampler_params: Vec<SamplerParameter>,
        memory_policy: MemoryPolicy<TextureCubeMap>,
    ) -> Self {
        let shared = Rc::new(RefCell::new(TextureShared {
            id: Uuid::new_v4(),
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_CUBE_MAP,
                internal_format,
                levels,
                width,
                height,
                depth: 0,
            },
            memory_policy: MemoryPolicyShared::TextureCubeMap(memory_policy),
            texture_params,
            sampler_params,
            queue: Vec::new(),
            registered: None,
            runtime: None,
        }));

        Self {
            layout: PhantomData,
            shared,
        }
    }

    /// Returns texture target.
    pub fn target(&self) -> TextureTarget {
        self.shared.borrow().layout.target
    }

    /// Returns texture internal format.
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.shared.borrow().layout.internal_format
    }

    /// Returns texture mipmap levels.
    pub fn levels(&self) -> usize {
        self.shared.borrow().layout.levels
    }

    /// Returns texture width at level 0.
    pub fn width(&self) -> usize {
        self.shared.borrow().layout.width
    }

    /// Returns texture height at level 0.
    pub fn height(&self) -> usize {
        self.shared.borrow().layout.height
    }

    /// Returns texture width at specified level.
    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let width = (layout.width >> level).max(1);
        Some(width)
    }

    /// Returns texture height at specified level.
    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        let layout = self.shared.borrow().layout;

        if level > layout.levels {
            return None;
        }

        let height = (layout.height >> level).max(1);
        Some(height)
    }

    /// Sets memory policy
    pub fn set_memory_policy(&self, memory_policy: MemoryPolicy<TextureCubeMap>) {
        self.shared.borrow_mut().memory_policy = MemoryPolicyShared::TextureCubeMap(memory_policy);
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.shared.borrow_mut().queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }
}

pub struct Builder<T> {
    name: Option<Cow<'static, str>>,
    layout: TextureLayout,
    memory_policy: MemoryPolicy<T>,
    texture_params: Vec<TextureParameter>,
    sampler_params: Vec<SamplerParameter>,
    queue: Vec<QueueItem>,
}

impl<T> Builder<T> {
    /// Sets texture name.
    pub fn set_name<S>(&mut self, name: S)
    where
        S: Into<String>,
    {
        self.name = Some(Cow::Owned(name.into()));
    }

    /// Sets texture name by static str.
    pub fn set_name_str(&mut self, name: &'static str) {
        self.name = Some(Cow::Borrowed(name.into()));
    }

    /// Sets a single texture parameters.
    pub fn set_texture_parameter(&mut self, texture_param: TextureParameter) {
        let old = self
            .texture_params
            .iter()
            .position(|p| p.kind() == texture_param.kind());
        match old {
            Some(index) => {
                let _ = std::mem::replace(&mut self.texture_params[index], texture_param);
            }
            None => self.texture_params.push(texture_param),
        }
    }

    /// Sets texture parameters.
    pub fn set_texture_parameters<I: IntoIterator<Item = TextureParameter>>(
        &mut self,
        texture_params: I,
    ) {
        self.texture_params.clear();
        self.texture_params.extend(texture_params);
    }

    /// Sets a single sampler parameters.
    pub fn set_sampler_parameter(&mut self, sampler_param: SamplerParameter) {
        let old = self
            .sampler_params
            .iter()
            .position(|p| p.kind() == sampler_param.kind());
        match old {
            Some(index) => {
                let _ = std::mem::replace(&mut self.sampler_params[index], sampler_param);
            }
            None => self.sampler_params.push(sampler_param),
        }
    }

    /// Sets sampler parameters.
    pub fn set_sampler_parameters<I: IntoIterator<Item = SamplerParameter>>(
        &mut self,
        sampler_params: I,
    ) {
        self.sampler_params.clear();
        self.sampler_params.extend(sampler_params);
    }

    /// Sets memory policy.
    pub fn set_memory_policy(&mut self, memory_policy: MemoryPolicy<T>) {
        self.memory_policy = memory_policy;
    }
}

impl Builder<Texture2D> {
    /// Creates a new 2d texture builder.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_2D,
                internal_format,
                levels,
                width,
                height,
                depth: 0,
            },
            memory_policy: MemoryPolicy::Unfree,
            texture_params: Vec::new(),
            sampler_params: Vec::new(),
            queue: Vec::new(),
        }
    }

    /// Creates a new 2d texture builder with automatically calculated mipmaps levels.
    pub fn with_auto_levels(
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
    ) -> Self {
        let levels = (width.max(height) as f64).log2().floor() as usize + 1;
        Self::new(internal_format, levels, width, height)
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }

    /// Builds 2d texture.
    pub fn build(self) -> Texture<Texture2D> {
        let shared = TextureShared {
            id: Uuid::new_v4(),
            name: self.name,
            layout: self.layout,
            memory_policy: MemoryPolicyShared::Texture2D(self.memory_policy),
            sampler_params: self.sampler_params,
            texture_params: self.texture_params,
            queue: self.queue,
            registered: None,
            runtime: None,
        };
        Texture::<Texture2D> {
            layout: PhantomData,
            shared: Rc::new(RefCell::new(shared)),
        }
    }
}

impl Builder<Texture2DArray> {
    /// Creates a new 2d array texture builder.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        len: usize,
    ) -> Self {
        Self {
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_2D_ARRAY,
                internal_format,
                levels,
                width,
                height,
                depth: len,
            },
            memory_policy: MemoryPolicy::Unfree,
            texture_params: Vec::new(),
            sampler_params: Vec::new(),
            queue: Vec::new(),
        }
    }

    /// Creates a new 2d array texture builder with automatically calculated mipmaps levels.
    pub fn with_auto_levels(
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
        len: usize,
    ) -> Self {
        let levels = (width.max(height) as f64).log2().floor() as usize + 1;
        Self::new(internal_format, levels, width, height, len)
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        array_index_offset: usize,
        width: usize,
        height: usize,
        array_index: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(array_index_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(array_index),
        });
    }

    /// Builds 2d array texture.
    pub fn build(self) -> Texture<Texture2DArray> {
        let shared = TextureShared {
            id: Uuid::new_v4(),
            name: self.name,
            layout: self.layout,
            memory_policy: MemoryPolicyShared::Texture2DArray(self.memory_policy),
            sampler_params: self.sampler_params,
            texture_params: self.texture_params,
            queue: self.queue,
            registered: None,
            runtime: None,
        };
        Texture::<Texture2DArray> {
            layout: PhantomData,
            shared: Rc::new(RefCell::new(shared)),
        }
    }
}

impl Builder<Texture3D> {
    /// Creates a new 3d texture builder.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
        depth: usize,
    ) -> Self {
        Self {
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_3D,
                internal_format,
                levels,
                width,
                height,
                depth,
            },
            memory_policy: MemoryPolicy::Unfree,
            texture_params: Vec::new(),
            sampler_params: Vec::new(),
            queue: Vec::new(),
        }
    }

    /// Creates a new 3d texture builder with automatically calculated mipmaps levels.
    pub fn with_auto_levels(
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
        depth: usize,
    ) -> Self {
        let levels = (width.max(height).max(depth) as f64).log2().floor() as usize + 1;
        Self::new(internal_format, levels, width, height, depth)
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        width: usize,
        height: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(z_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(depth),
        });
    }

    /// Builds 3d texture.
    pub fn build(self) -> Texture<Texture3D> {
        let shared = TextureShared {
            id: Uuid::new_v4(),
            name: self.name,
            layout: self.layout,
            memory_policy: MemoryPolicyShared::Texture3D(self.memory_policy),
            sampler_params: self.sampler_params,
            texture_params: self.texture_params,
            queue: self.queue,
            registered: None,
            runtime: None,
        };
        Texture::<Texture3D> {
            layout: PhantomData,
            shared: Rc::new(RefCell::new(shared)),
        }
    }
}

impl Builder<TextureCubeMap> {
    /// Creates a new cube map texture builder.
    pub fn new(
        internal_format: TextureInternalFormat,
        levels: usize,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            name: None,
            layout: TextureLayout {
                target: TextureTarget::TEXTURE_CUBE_MAP,
                internal_format,
                levels,
                width,
                height,
                depth: 0,
            },
            memory_policy: MemoryPolicy::Unfree,
            texture_params: Vec::new(),
            sampler_params: Vec::new(),
            queue: Vec::new(),
        }
    }

    /// Creates a new cube map texture builder with automatically calculated mipmaps levels.
    pub fn with_auto_levels(
        internal_format: TextureInternalFormat,
        width: usize,
        height: usize,
    ) -> Self {
        let levels = (width.max(height) as f64).log2().floor() as usize + 1;
        Self::new(internal_format, levels, width, height)
    }

    /// Uploads new texture image data.
    pub fn tex_image<S>(
        &mut self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Uploads new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }

    /// Builds 3d texture.
    pub fn build(self) -> Texture<TextureCubeMap> {
        let shared = TextureShared {
            id: Uuid::new_v4(),
            name: self.name,
            layout: self.layout,
            memory_policy: MemoryPolicyShared::TextureCubeMap(self.memory_policy),
            sampler_params: self.sampler_params,
            texture_params: self.texture_params,
            queue: self.queue,
            registered: None,
            runtime: None,
        };
        Texture::<TextureCubeMap> {
            layout: PhantomData,
            shared: Rc::new(RefCell::new(shared)),
        }
    }
}

/// Memory policies kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryPolicyKind {
    Unfree,
    Restorable,
}

enum MemoryPolicyShared {
    Texture2D(MemoryPolicy<Texture2D>),
    Texture2DArray(MemoryPolicy<Texture2DArray>),
    Texture3D(MemoryPolicy<Texture3D>),
    TextureCubeMap(MemoryPolicy<TextureCubeMap>),
}

pub struct RestoreReceiver<T> {
    layout: PhantomData<T>,
    queue: Vec<QueueItem>,
}

impl<T> TextureSource for RestoreReceiver<T> {
    fn data(&self) -> TextureData {
        todo!()
    }
}

impl RestoreReceiver<Texture2D> {
    fn new() -> Self {
        Self {
            layout: PhantomData,
            queue: Vec::new(),
        }
    }

    /// Restores new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Restores new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }
}

impl RestoreReceiver<Texture2DArray> {
    fn new() -> Self {
        Self {
            layout: PhantomData,
            queue: Vec::new(),
        }
    }

    /// Restores new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Restores new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        array_index_offset: usize,
        width: usize,
        height: usize,
        array_index: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_2D_ARRAY,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(array_index_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(array_index),
        });
    }
}

impl RestoreReceiver<Texture3D> {
    fn new() -> Self {
        Self {
            layout: PhantomData,
            queue: Vec::new(),
        }
    }

    /// Restores new texture image data.
    pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Restores new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        width: usize,
        height: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_3D,
            cube_map_face: None,
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: Some(z_offset),
            width: Some(width),
            height: Some(height),
            depth: Some(depth),
        });
    }
}

impl RestoreReceiver<TextureCubeMap> {
    fn new() -> Self {
        Self {
            layout: PhantomData,
            queue: Vec::new(),
        }
    }

    /// Restores new texture image data.
    pub fn tex_image<S>(
        &mut self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: None,
            y_offset: None,
            z_offset: None,
            width: None,
            height: None,
            depth: None,
        });
    }

    /// Restores new texture sub image data.
    pub fn tex_sub_image<S>(
        &mut self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        width: usize,
        height: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSource + 'static,
    {
        self.queue.push(QueueItem {
            source: Box::new(source),
            target: TextureTarget::TEXTURE_CUBE_MAP,
            cube_map_face: Some(face),
            generate_mipmaps,
            level,
            x_offset: Some(x_offset),
            y_offset: Some(y_offset),
            z_offset: None,
            width: Some(width),
            height: Some(height),
            depth: None,
        });
    }
}

/// Tetxure restorer for restoring a texture.
pub trait Restorer<T> {
    /// Restors necessary data to a texture builder.
    fn restore(&self, builder: &mut RestoreReceiver<T>);
}

/// Memory policies.
pub enum MemoryPolicy<T> {
    Unfree,
    Restorable(Rc<dyn Restorer<T>>),
}

impl<T> Debug for MemoryPolicy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unfree => write!(f, "Unfree"),
            Self::Restorable(_) => write!(f, "Restorable"),
        }
    }
}

impl<T> MemoryPolicy<T> {
    /// Constructs a unfree-able memory policy.
    pub fn unfree() -> Self {
        Self::Unfree
    }

    /// Constructs a restorable memory policy.
    pub fn restorable<R>(restorer: R) -> Self
    where
        R: Restorer<T> + 'static,
    {
        Self::Restorable(Rc::new(restorer))
    }

    /// Returns [`MemoryPolicyKind`] associated with the [`MemoryPolicy`].
    pub fn kind(&self) -> MemoryPolicyKind {
        match self {
            MemoryPolicy::Unfree => MemoryPolicyKind::Unfree,
            MemoryPolicy::Restorable(_) => MemoryPolicyKind::Restorable,
        }
    }
}

struct StoreShared {
    gl: WebGl2RenderingContext,
    id: Uuid,

    available_memory: usize,
    used_memory: usize,

    lru: Lru<Uuid>,
    textures: HashMap<Uuid, Weak<RefCell<TextureShared>>>,
    bindings: HashMap<(TextureUnit, TextureTarget), Uuid>,
}

impl Drop for StoreShared {
    fn drop(&mut self) {
        for texture in self.textures.values_mut() {
            let Some(texture) = texture.upgrade() else {
                continue;
            };
            texture.borrow_mut().registered = None;
        }
    }
}

impl StoreShared {
    fn update_lru(&mut self, lru_node: *mut LruNode<Uuid>) {
        unsafe {
            self.lru.cache(lru_node);
        }
    }

    fn increase_used_memory(&mut self, byte_length: usize) {
        self.used_memory += byte_length;
    }

    fn decrease_used_memory(&mut self, byte_length: usize) {
        self.used_memory -= byte_length;
    }

    fn add_binding(&mut self, unit: TextureUnit, target: TextureTarget, id: Uuid) {
        self.bindings.insert((unit, target), id);
    }

    fn remove_binding(&mut self, unit: TextureUnit, target: TextureTarget) {
        self.bindings.remove(&(unit, target));
    }

    fn remove(&mut self, byte_length: usize, lru_node: *mut LruNode<Uuid>) {
        self.decrease_used_memory(byte_length);
        unsafe {
            self.lru.remove(lru_node);
        }
    }

    fn is_occupied(&self, unit: TextureUnit, target: TextureTarget, id: &Uuid) -> bool {
        self.bindings
            .get(&(unit, target))
            .map(|v| v != id)
            .unwrap_or(false)
    }

    fn free(&mut self) {
        unsafe {
            if self.used_memory <= self.available_memory {
                return;
            }

            let mut next_node = self.lru.least_recently();
            while self.used_memory > self.available_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };
                let id = (*current_node).data();
                let Entry::Occupied(occupied) = self.textures.entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };

                let texture = occupied.get();
                let Some(texture) = texture.upgrade() else {
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };

                if let Ok(mut texture) = texture.try_borrow_mut() {
                    if !texture.free() {
                        next_node = (*current_node).more_recently();
                        continue;
                    }
                }

                occupied.remove();
                next_node = (*current_node).more_recently();
            }
        }
    }

    fn unregister<'a, B>(
        &mut self,
        id: &Uuid,
        lru_node: *mut LruNode<Uuid>,
        byte_length: usize,
        target: TextureTarget,
        bindings: B,
    ) where
        B: IntoIterator<Item = &'a TextureUnit>,
    {
        bindings.into_iter().for_each(|unit| {
            self.bindings.remove(&(*unit, target));
        });
        self.used_memory -= byte_length;
        self.textures.remove(id);
        unsafe {
            self.lru.remove(lru_node);
        }
    }
}

pub struct TextureStore {
    shared: Rc<RefCell<StoreShared>>,
}

impl TextureStore {
    /// Constructs a new texture store with [`i32::MAX`] bytes memory limitation.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_available_memory(gl, i32::MAX as usize)
    }

    /// Constructs a new texture store with a maximum available memory.
    /// Maximum available memory is clamped to [`i32::MAX`] if larger than [`i32::MAX`];
    pub fn with_available_memory(gl: WebGl2RenderingContext, available_memory: usize) -> Self {
        let shared = StoreShared {
            gl,
            id: Uuid::new_v4(),

            available_memory,
            used_memory: 0,

            lru: Lru::new(),
            textures: HashMap::new(),
            bindings: HashMap::new(),
        };

        Self {
            shared: Rc::new(RefCell::new(shared)),
        }
    }

    /// Returns store id.
    pub fn id(&self) -> Uuid {
        self.shared.borrow().id
    }

    /// Returns the maximum available memory in bytes.
    /// Returns [`i32::MAX`] if not specified.
    pub fn available_memory(&self) -> usize {
        self.shared.borrow().available_memory
    }

    /// Returns current used memory in bytes.
    pub fn used_memory(&self) -> usize {
        self.shared.borrow().used_memory
    }

    /// Registers a texture to store, and initializes the texture.
    pub fn register<L>(&self, texture: &Texture<L>) -> Result<(), Error> {
        unsafe {
            let mut store_shared = self.shared.borrow_mut();
            let mut texture_shared = texture.shared.borrow_mut();

            if let Some(store) = texture_shared
                .registered
                .as_ref()
                .and_then(|registered| registered.store.upgrade())
            {
                if let Ok(store) = store.try_borrow() {
                    if &store.id != &store_shared.id {
                        return Err(Error::RegisterTextureToMultipleStore);
                    } else {
                        return Ok(());
                    }
                } else {
                    // if store is borrowed, it means that store of registered is the same store as self.
                    return Ok(());
                }
            }

            texture_shared.init(&store_shared.gl)?;

            let runtime = texture_shared.runtime.as_ref().unwrap();
            store_shared.used_memory += runtime.byte_length;
            let target = texture_shared.layout.target;
            for unit in &runtime.bindings {
                let key = (*unit, target);
                if store_shared.bindings.contains_key(&key) {
                    return Err(Error::TextureTargetOccupied(*unit, target));
                }
                store_shared.bindings.insert(key, texture_shared.id);
            }

            texture_shared.registered = Some(TextureRegistered {
                store: Rc::downgrade(&self.shared),
                lru_node: LruNode::new(texture_shared.id),
            });

            store_shared
                .textures
                .insert(texture_shared.id, Rc::downgrade(&texture.shared));

            Ok(())
        }
    }

    /// Unregisters a texture from store.
    pub fn unregister<L>(&self, texture: &Texture<L>) {
        unsafe {
            let mut store_shared = self.shared.borrow_mut();
            let mut texture_shared = texture.shared.borrow_mut();

            if store_shared.textures.remove(&texture_shared.id).is_none() {
                return;
            }

            let runtime = texture_shared.runtime.as_ref().unwrap();
            store_shared.used_memory -= runtime.byte_length;
            let target = texture_shared.layout.target;
            for unit in &runtime.bindings {
                let key = (*unit, target);
                if let Entry::Occupied(entry) = store_shared.bindings.entry(key) {
                    if &texture_shared.id == entry.get() {
                        entry.remove();
                    }
                }
            }

            if let Some(registered) = texture_shared.registered.take() {
                store_shared.lru.remove(registered.lru_node);
            }
        }
    }
}
