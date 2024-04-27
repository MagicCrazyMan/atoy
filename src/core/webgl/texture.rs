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

use super::{conversion::ToGlEnum, error::Error};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureTarget {
    Texture2d,
    TextureCubeMap,
    Texture2dArray,
    Texture3d,
}

/// Available texture units mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnit {
    Texture0,
    Texture1,
    Texture2,
    Texture3,
    Texture4,
    Texture5,
    Texture6,
    Texture7,
    Texture8,
    Texture9,
    Texture10,
    Texture11,
    Texture12,
    Texture13,
    Texture14,
    Texture15,
    Texture16,
    Texture17,
    Texture18,
    Texture19,
    Texture20,
    Texture21,
    Texture22,
    Texture23,
    Texture24,
    Texture25,
    Texture26,
    Texture27,
    Texture28,
    Texture29,
    Texture30,
    Texture31,
}

impl TextureUnit {
    pub fn unit_index(&self) -> u32 {
        match self {
            TextureUnit::Texture0 => 0,
            TextureUnit::Texture1 => 1,
            TextureUnit::Texture2 => 2,
            TextureUnit::Texture3 => 3,
            TextureUnit::Texture4 => 4,
            TextureUnit::Texture5 => 5,
            TextureUnit::Texture6 => 6,
            TextureUnit::Texture7 => 7,
            TextureUnit::Texture8 => 8,
            TextureUnit::Texture9 => 9,
            TextureUnit::Texture10 => 10,
            TextureUnit::Texture11 => 11,
            TextureUnit::Texture12 => 12,
            TextureUnit::Texture13 => 13,
            TextureUnit::Texture14 => 14,
            TextureUnit::Texture15 => 15,
            TextureUnit::Texture16 => 16,
            TextureUnit::Texture17 => 17,
            TextureUnit::Texture18 => 18,
            TextureUnit::Texture19 => 19,
            TextureUnit::Texture20 => 20,
            TextureUnit::Texture21 => 21,
            TextureUnit::Texture22 => 22,
            TextureUnit::Texture23 => 23,
            TextureUnit::Texture24 => 24,
            TextureUnit::Texture25 => 25,
            TextureUnit::Texture26 => 26,
            TextureUnit::Texture27 => 27,
            TextureUnit::Texture28 => 28,
            TextureUnit::Texture29 => 29,
            TextureUnit::Texture30 => 30,
            TextureUnit::Texture31 => 31,
        }
    }
}

/// Available texture pixel data types mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUncompressedPixelDataType {
    Float,
    HalfFloat,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    UnsignedShort5_6_5,
    UnsignedShort4_4_4_4,
    UnsignedShort5_5_5_1,
    UnsignedInt2_10_10_10Rev,
    #[allow(non_camel_case_types)]
    UnsignedInt10F_11F_11F_Rev,
    UnsignedInt5_9_9_9Rev,
    UnsignedInt24_8,
    Float32UnsignedInt24_8Rev,
}

/// Available texture unpack color space conversions for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackColorSpaceConversion {
    None,
    BrowserDefault,
}

/// Available texture pixel storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePackPixelStorage {
    PackAlignment(i32),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
}

impl TexturePackPixelStorage {
    fn set(&self, gl: &WebGl2RenderingContext) {
        match self {
            TexturePackPixelStorage::PackAlignment(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, *v);
            }
            TexturePackPixelStorage::PackRowLength(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, *v);
            }
            TexturePackPixelStorage::PackSkipPixels(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, *v);
            }
            TexturePackPixelStorage::PackSkipRows(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, *v);
            }
        }
    }

    fn reset(&self, gl: &WebGl2RenderingContext) {
        match self {
            TexturePackPixelStorage::PackAlignment(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, 4);
            }
            TexturePackPixelStorage::PackRowLength(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, 0);
            }
            TexturePackPixelStorage::PackSkipPixels(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, 0);
            }
            TexturePackPixelStorage::PackSkipRows(_) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, 0);
            }
        }
    }
}

/// Available texture pixel storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackPixelStorage {
    UnpackAlignment(i32),
    UnpackFlipY(bool),
    UnpackPremultiplyAlpha(bool),
    UnpackColorSpaceConversion(TextureUnpackColorSpaceConversion),
    UnpackRowLength(i32),
    UnpackImageHeight(i32),
    UnpackSkipPixels(i32),
    UnpackSkipRows(i32),
    UnpackSkipImages(i32),
}

impl TextureUnpackPixelStorage {
    fn set(&self, gl: &WebGl2RenderingContext) {
        match self {
            TextureUnpackPixelStorage::UnpackAlignment(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, *v);
            }
            TextureUnpackPixelStorage::UnpackFlipY(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
                    if *v { 1 } else { 0 },
                );
            }
            TextureUnpackPixelStorage::UnpackPremultiplyAlpha(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                    if *v { 1 } else { 0 },
                );
            }
            TextureUnpackPixelStorage::UnpackColorSpaceConversion(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    match v {
                        TextureUnpackColorSpaceConversion::None => {
                            WebGl2RenderingContext::NONE as i32
                        }
                        TextureUnpackColorSpaceConversion::BrowserDefault => {
                            WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32
                        }
                    },
                );
            }
            TextureUnpackPixelStorage::UnpackRowLength(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, *v);
            }
            TextureUnpackPixelStorage::UnpackImageHeight(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, *v);
            }
            TextureUnpackPixelStorage::UnpackSkipPixels(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, *v);
            }
            TextureUnpackPixelStorage::UnpackSkipRows(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, *v);
            }
            TextureUnpackPixelStorage::UnpackSkipImages(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, *v);
            }
        }
    }

    fn reset(&self, gl: &WebGl2RenderingContext) {
        match self {
            TextureUnpackPixelStorage::UnpackAlignment(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, 4);
            }
            TextureUnpackPixelStorage::UnpackFlipY(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 0);
            }
            TextureUnpackPixelStorage::UnpackPremultiplyAlpha(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0);
            }
            TextureUnpackPixelStorage::UnpackColorSpaceConversion(_) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32,
                );
            }
            TextureUnpackPixelStorage::UnpackRowLength(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, 0);
            }
            TextureUnpackPixelStorage::UnpackImageHeight(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, 0);
            }
            TextureUnpackPixelStorage::UnpackSkipPixels(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, 0);
            }
            TextureUnpackPixelStorage::UnpackSkipRows(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, 0);
            }
            TextureUnpackPixelStorage::UnpackSkipImages(_) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, 0);
            }
        }
    }
}

/// Available texture magnification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMagnificationFilter {
    Linear,
    Nearest,
}

/// Available texture minification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMinificationFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

/// Available texture wrap methods for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapMethod {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

/// Available texture compare function for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareFunction {
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    Equal,
    NotEqual,
    Always,
    Never,
}

/// Available texture compare modes for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareMode {
    None,
    CompareRefToTexture,
}

/// Available texture parameter kinds mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameterKind {
    BaseLevel,
    MaxLevel,
    /// Available when extension `EXT_texture_filter_anisotropic` enabled.
    MaxAnisotropy,
}

/// Available texture parameters mapped from [`WebGl2RenderingContext`].
///
/// Different from WebGL1, WebGL2 separates sampling parameters to a new object called [`WebGlSampler`],
/// those sampling parameters are no more included in this enum. Checks [`SamplerParameter`] for more details.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameter {
    BaseLevel(i32),
    MaxLevel(i32),
    /// Available when extension `EXT_texture_filter_anisotropic` enabled.
    MaxAnisotropy(f32),
}

impl TextureParameter {
    /// Returns texture kind.
    pub fn kind(&self) -> TextureParameterKind {
        match self {
            TextureParameter::BaseLevel(_) => TextureParameterKind::BaseLevel,
            TextureParameter::MaxLevel(_) => TextureParameterKind::MaxLevel,
            TextureParameter::MaxAnisotropy(_) => TextureParameterKind::MaxAnisotropy,
        }
    }

    fn set(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        match self {
            TextureParameter::BaseLevel(v) => {
                gl.tex_parameteri(
                    target.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    *v,
                );
            }
            TextureParameter::MaxLevel(v) => {
                gl.tex_parameteri(
                    target.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
                    *v,
                );
            }
            TextureParameter::MaxAnisotropy(v) => {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplerParameterKind {
    MagnificationFilter,
    MinificationFilter,
    WrapS,
    WrapT,
    WrapR,
    CompareFunction,
    CompareMode,
    MaxLod,
    MinLod,
}

/// Available sampling parameters for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplerParameter {
    MagnificationFilter(TextureMagnificationFilter),
    MinificationFilter(TextureMinificationFilter),
    WrapS(TextureWrapMethod),
    WrapT(TextureWrapMethod),
    WrapR(TextureWrapMethod),
    CompareFunction(TextureCompareFunction),
    CompareMode(TextureCompareMode),
    MaxLod(f32),
    MinLod(f32),
}

impl SamplerParameter {
    /// Returns sampler kind.
    pub fn kind(&self) -> SamplerParameterKind {
        match self {
            SamplerParameter::MagnificationFilter(_) => SamplerParameterKind::MagnificationFilter,
            SamplerParameter::MinificationFilter(_) => SamplerParameterKind::MinificationFilter,
            SamplerParameter::WrapS(_) => SamplerParameterKind::WrapS,
            SamplerParameter::WrapT(_) => SamplerParameterKind::WrapT,
            SamplerParameter::WrapR(_) => SamplerParameterKind::WrapR,
            SamplerParameter::CompareFunction(_) => SamplerParameterKind::CompareFunction,
            SamplerParameter::CompareMode(_) => SamplerParameterKind::CompareMode,
            SamplerParameter::MaxLod(_) => SamplerParameterKind::MaxLod,
            SamplerParameter::MinLod(_) => SamplerParameterKind::MinLod,
        }
    }

    fn set(&self, gl: &WebGl2RenderingContext, sampler: &WebGlSampler) {
        match self {
            SamplerParameter::MagnificationFilter(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::MinificationFilter(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::WrapS(v)
            | SamplerParameter::WrapT(v)
            | SamplerParameter::WrapR(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::CompareFunction(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::CompareMode(v) => {
                gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
            }
            SamplerParameter::MaxLod(v) | SamplerParameter::MinLod(v) => {
                gl.sampler_parameterf(&sampler, self.gl_enum(), *v)
            }
        }
    }
}

/// Available texture formats mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUncompressedPixelFormat {
    Red,
    RedInteger,
    Rg,
    RgInteger,
    Rgb,
    RgbInteger,
    Rgba,
    RgbaInteger,
    Luminance,
    LuminanceAlpha,
    Alpha,
    DepthComponent,
    DepthStencil,
}

pub trait TextureInternalFormat: ToGlEnum {
    /// Returns byte length of this internal format in specified size.
    fn byte_length(&self, width: usize, height: usize) -> usize;
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

impl TextureInternalFormat for TextureUncompressedInternalFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
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

impl TextureInternalFormat for TextureCompressedFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
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

#[derive(Debug, Clone)]
pub enum TextureUncompressedData {
    PixelBufferObject {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        pbo_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int8Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Int8Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint8Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Uint8Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint8ClampedArray {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int16Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Int16Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint16Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Uint16Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int32Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Int32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint32Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Uint32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Float32Array {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: Float32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    DataView {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        width: usize,
        height: usize,
        data: DataView,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlCanvasElement {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        data: HtmlCanvasElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlImageElement {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        data: HtmlImageElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlVideoElement {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        data: HtmlVideoElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    ImageData {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        data: ImageData,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    ImageBitmap {
        pixel_format: TextureUncompressedPixelFormat,
        pixel_data_type: TextureUncompressedPixelDataType,
        data: ImageBitmap,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
}

#[derive(Debug)]
pub enum TextureCompressedData {
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

// impl TextureData {
//     fn width(&self) -> usize {
//         match self {
//             TextureData::Uncompressed { data, .. } => match data {
//                 TextureUncompressedData::PixelBufferObject { width, .. }
//                 | TextureUncompressedData::Int8Array { width, .. }
//                 | TextureUncompressedData::Uint8Array { width, .. }
//                 | TextureUncompressedData::Uint8ClampedArray { width, .. }
//                 | TextureUncompressedData::Int16Array { width, .. }
//                 | TextureUncompressedData::Uint16Array { width, .. }
//                 | TextureUncompressedData::Int32Array { width, .. }
//                 | TextureUncompressedData::Uint32Array { width, .. }
//                 | TextureUncompressedData::Float32Array { width, .. }
//                 | TextureUncompressedData::DataView { width, .. } => *width,
//                 TextureUncompressedData::HtmlCanvasElement { data, .. } => data.width() as usize,
//                 TextureUncompressedData::HtmlImageElement { data, .. } => {
//                     data.natural_width() as usize
//                 }
//                 TextureUncompressedData::HtmlVideoElement { data, .. } => {
//                     data.video_width() as usize
//                 }
//                 TextureUncompressedData::ImageData { data, .. } => data.width() as usize,
//                 TextureUncompressedData::ImageBitmap { data, .. } => data.width() as usize,
//             },
//             TextureData::Compressed { data, .. } => match data {
//                 TextureCompressedData::PixelBufferObject { width, .. }
//                 | TextureCompressedData::Int8Array { width, .. }
//                 | TextureCompressedData::Uint8Array { width, .. }
//                 | TextureCompressedData::Uint8ClampedArray { width, .. }
//                 | TextureCompressedData::Int16Array { width, .. }
//                 | TextureCompressedData::Uint16Array { width, .. }
//                 | TextureCompressedData::Int32Array { width, .. }
//                 | TextureCompressedData::Uint32Array { width, .. }
//                 | TextureCompressedData::Float32Array { width, .. }
//                 | TextureCompressedData::DataView { width, .. } => *width,
//             },
//         }
//     }

//     fn height(&self) -> usize {
//         match self {
//             TextureData::Uncompressed { data, .. } => match data {
//                 TextureUncompressedData::PixelBufferObject { height, .. }
//                 | TextureUncompressedData::Int8Array { height, .. }
//                 | TextureUncompressedData::Uint8Array { height, .. }
//                 | TextureUncompressedData::Uint8ClampedArray { height, .. }
//                 | TextureUncompressedData::Int16Array { height, .. }
//                 | TextureUncompressedData::Uint16Array { height, .. }
//                 | TextureUncompressedData::Int32Array { height, .. }
//                 | TextureUncompressedData::Uint32Array { height, .. }
//                 | TextureUncompressedData::Float32Array { height, .. }
//                 | TextureUncompressedData::DataView { height, .. } => *height,
//                 TextureUncompressedData::HtmlCanvasElement { data, .. } => data.height() as usize,
//                 TextureUncompressedData::HtmlImageElement { data, .. } => {
//                     data.natural_height() as usize
//                 }
//                 TextureUncompressedData::HtmlVideoElement { data, .. } => {
//                     data.video_height() as usize
//                 }
//                 TextureUncompressedData::ImageData { data, .. } => data.height() as usize,
//                 TextureUncompressedData::ImageBitmap { data, .. } => data.height() as usize,
//             },
//             TextureData::Compressed { data, .. } => match data {
//                 TextureCompressedData::PixelBufferObject { height, .. }
//                 | TextureCompressedData::Int8Array { height, .. }
//                 | TextureCompressedData::Uint8Array { height, .. }
//                 | TextureCompressedData::Uint8ClampedArray { height, .. }
//                 | TextureCompressedData::Int16Array { height, .. }
//                 | TextureCompressedData::Uint16Array { height, .. }
//                 | TextureCompressedData::Int32Array { height, .. }
//                 | TextureCompressedData::Uint32Array { height, .. }
//                 | TextureCompressedData::Float32Array { height, .. }
//                 | TextureCompressedData::DataView { height, .. } => *height,
//             },
//         }
//     }
// }

pub trait TextureLayout2D {
    fn levels(&self) -> usize;

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize;
}

pub trait TextureLayout3D {
    fn levels(&self) -> usize;

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn depth(&self) -> usize;

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture2D {
    levels: usize,
    width: usize,
    height: usize,
}

impl TextureLayout2D for Texture2D {
    fn levels(&self) -> usize {
        self.levels
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize {
        (0..self.levels)
            .map(|level| {
                let width = (self.width >> level).max(1);
                let height = (self.height >> level).max(1);
                internal_format.byte_length(width, height)
            })
            .sum::<usize>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture2DArray {
    levels: usize,
    width: usize,
    height: usize,
    length: usize,
}

impl TextureLayout3D for Texture2DArray {
    fn levels(&self) -> usize {
        self.levels
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn depth(&self) -> usize {
        self.length
    }

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize {
        (0..self.levels)
            .map(|level| {
                let width = (self.width >> level).max(1);
                let height = (self.height >> level).max(1);
                internal_format.byte_length(width, height) * self.length
            })
            .sum::<usize>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture3D {
    levels: usize,
    width: usize,
    height: usize,
    depth: usize,
}

impl TextureLayout3D for Texture3D {
    fn levels(&self) -> usize {
        self.levels
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn depth(&self) -> usize {
        self.depth
    }

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize {
        (0..self.levels)
            .map(|level| {
                let width = (self.width >> level).max(1);
                let height = (self.height >> level).max(1);
                let depth = (self.depth >> level).max(1);
                internal_format.byte_length(width, height) * depth
            })
            .sum::<usize>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureCubeMap {
    levels: usize,
    width: usize,
    height: usize,
}

impl TextureLayout2D for TextureCubeMap {
    fn levels(&self) -> usize {
        self.levels
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn byte_length(&self, internal_format: &dyn TextureInternalFormat) -> usize {
        (0..self.levels)
            .map(|level| {
                let width = (self.width >> level).max(1);
                let height = (self.height >> level).max(1);
                internal_format.byte_length(width, height) * 6
            })
            .sum::<usize>()
    }
}

const TEXTURE_UNIT: TextureUnit = TextureUnit::Texture7;

#[derive(Debug)]
enum QueueItem {
    Uncompressed {
        data: TextureUncompressedData,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    },
    Compressed {
        data: TextureCompressedData,
        format: TextureCompressedFormat,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Texture<Layout, InternalFormat> {
    id: Uuid,
    layout: Layout,
    internal_format: InternalFormat,

    sampler_params: Rc<RefCell<Vec<SamplerParameter>>>,
    texture_params: Rc<RefCell<Vec<TextureParameter>>>,
    queue: Rc<RefCell<Vec<QueueItem>>>,

    registered: Rc<RefCell<Option<TextureRegistered>>>,
}

impl<Layout, InternalFormat> Texture<Layout, InternalFormat> {
    pub fn new(layout: Layout, internal_format: InternalFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            layout,
            internal_format,

            sampler_params: Rc::new(RefCell::new(Vec::new())),
            texture_params: Rc::new(RefCell::new(Vec::new())),
            queue: Rc::new(RefCell::new(Vec::new())),

            registered: Rc::new(RefCell::new(None)),
        }
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn internal_format(&self) -> &InternalFormat {
        &self.internal_format
    }
}

impl<Layout, InternalFormat> Texture<Layout, InternalFormat>
where
    Layout: TextureLayout2D,
    InternalFormat: TextureInternalFormat,
{
    fn tex_storage_2d(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        gl.tex_storage_2d(
            target.gl_enum(),
            self.layout.levels() as i32,
            self.internal_format.gl_enum(),
            self.layout.width() as i32,
            self.layout.height() as i32,
        )
    }
}

impl<Layout, InternalFormat> Texture<Layout, InternalFormat>
where
    Layout: TextureLayout3D,
    InternalFormat: TextureInternalFormat,
{
    fn tex_storage_3d(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        gl.tex_storage_3d(
            target.gl_enum(),
            self.layout.levels() as i32,
            self.internal_format.gl_enum(),
            self.layout.width() as i32,
            self.layout.height() as i32,
            self.layout.depth() as i32,
        )
    }
}

impl<Layout> Texture<Layout, TextureUncompressedInternalFormat>
where
    Layout: TextureLayout2D,
{
    fn write_2d(&self, data: TextureUncompressedData, level: usize, generate_mipmaps: bool) {
        self.write_2d_with_offset(data, level, 0, 0, generate_mipmaps)
    }

    fn write_2d_with_offset(
        &self,
        data: TextureUncompressedData,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) {
        self.queue.borrow_mut().push(QueueItem::Uncompressed {
            data,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
            generate_mipmaps,
        })
    }
}

impl<Layout> Texture<Layout, TextureUncompressedInternalFormat>
where
    Layout: TextureLayout2D,
{
    fn write_3d(
        &self,
        data: TextureUncompressedData,
        level: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) {
        self.write_3d_with_offset(data, level, depth, 0, 0, 0, generate_mipmaps)
    }

    fn write_3d_with_offset(
        &self,
        data: TextureUncompressedData,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    ) {
        self.queue.borrow_mut().push(QueueItem::Uncompressed {
            data,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
            generate_mipmaps,
        })
    }
}

impl<Layout> Texture<Layout, TextureCompressedFormat>
where
    Layout: TextureLayout2D,
{
    fn write_2d_compressed(&self, data: TextureCompressedData, level: usize) {
        self.write_2d_with_offset_compressed(data, level, 0, 0)
    }

    fn write_2d_with_offset_compressed(
        &self,
        data: TextureCompressedData,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) {
        self.queue.borrow_mut().push(QueueItem::Compressed {
            data,
            format: self.internal_format,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
        })
    }
}

impl<Layout> Texture<Layout, TextureCompressedFormat>
where
    Layout: TextureLayout2D,
{
    fn write_3d_compressed(&self, data: TextureCompressedData, level: usize, depth: usize) {
        self.write_3d_with_offset_compressed(data, level, depth, 0, 0, 0)
    }

    fn write_3d_with_offset_compressed(
        &self,
        data: TextureCompressedData,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) {
        self.queue.borrow_mut().push(QueueItem::Compressed {
            data,
            format: self.internal_format,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
        })
    }
}

#[derive(Debug)]
struct TextureRegistered {
    gl: WebGl2RenderingContext,
    gl_texture: WebGlTexture,
    gl_sampler: WebGlSampler,

    repo_id: Uuid,
    repo_items: Weak<RefCell<HashMap<Uuid, Weak<RefCell<Option<TextureRegistered>>>>>>,
    repo_used_memory: Weak<RefCell<usize>>,

    texture_id: Uuid,
    texture_memory: usize,
    texture_params: Weak<RefCell<Vec<TextureParameter>>>,
    sampler_params: Weak<RefCell<Vec<SamplerParameter>>>,
    texture_queue: Weak<RefCell<Vec<QueueItem>>>,
}

impl Drop for TextureRegistered {
    fn drop(&mut self) {
        todo!()
    }
}

impl TextureRegistered {
    fn upload(&self) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct TextureRepository {
    id: Uuid,
    gl: WebGl2RenderingContext,
    items: Rc<RefCell<HashMap<Uuid, Weak<RefCell<Option<TextureRegistered>>>>>>,
    used_memory: Rc<RefCell<usize>>,
}

impl TextureRepository {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            items: Rc::new(RefCell::new(HashMap::new())),
            used_memory: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn register(
        &self,
        texture: &Texture<Texture2D, TextureUncompressedInternalFormat>,
    ) -> Result<(), Error> {
        if let Some(registered) = &*texture.registered.borrow() {
            if registered.repo_id != self.id {
                return Err(Error::RegisterTextureToMultipleRepositoryUnsupported);
            } else {
                registered.upload()?;
                return Ok(());
            }
        }

        let gl_texture = self.gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let gl_sampler = self.gl.create_sampler().ok_or(Error::CreateSamplerFailure)?;
        // self.gl.active_texture(texture)

        todo!()
    }

    fn register_inner<Layout, InternalFormat>(
        &self,
        texture: &Texture<Layout, InternalFormat>,
    ) -> Result<(), Error> {
        todo!()
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum TextureCubeMapFace {
//     PositiveX,
//     NegativeX,
//     PositiveY,
//     NegativeY,
//     PositiveZ,
//     NegativeZ,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// struct TextureLayout {
//     target: TextureTarget,
//     internal_format: TextureInternalFormat,
//     levels: usize,
//     width: usize,
//     height: usize,
//     depth: usize,
// }

// impl TextureLayout {
//     fn target(&self) -> TextureTarget {
//         self.target
//     }

//     fn internal_format(&self) -> TextureInternalFormat {
//         self.internal_format
//     }

//     fn byte_length(&self) -> usize {
//         match self.target {
//             TextureTarget::Texture2d => (0..self.levels)
//                 .map(|level| {
//                     let width = (self.width >> level).max(1);
//                     let height = (self.height >> level).max(1);
//                     self.internal_format.byte_length(width, height)
//                 })
//                 .sum::<usize>(),
//             TextureTarget::Texture2dArray => (0..self.levels)
//                 .map(|level| {
//                     let width = (self.width >> level).max(1);
//                     let height = (self.height >> level).max(1);
//                     self.internal_format.byte_length(width, height) * self.depth
//                 })
//                 .sum::<usize>(),
//             TextureTarget::Texture3d => (0..self.levels)
//                 .map(|level| {
//                     let width = (self.width >> level).max(1);
//                     let height = (self.height >> level).max(1);
//                     let depth = (self.depth >> level).max(1);
//                     self.internal_format.byte_length(width, height) * depth
//                 })
//                 .sum::<usize>(),
//             TextureTarget::TextureCubeMap => (0..self.levels)
//                 .map(|level| {
//                     let width = (self.width >> level).max(1);
//                     let height = (self.height >> level).max(1);
//                     self.internal_format.byte_length(width, height) * 6
//                 })
//                 .sum::<usize>(),
//         }
//     }
// }

// struct QueueItem {
//     source: Box<dyn TextureSource>,
//     target: TextureTarget,
//     cube_map_face: Option<TextureCubeMapFace>,
//     generate_mipmaps: bool,
//     level: usize,
//     x_offset: Option<usize>,
//     y_offset: Option<usize>,
//     z_offset: Option<usize>,
//     width: Option<usize>,
//     height: Option<usize>,
//     depth: Option<usize>,
// }

// struct TextureRuntime {
//     gl: WebGl2RenderingContext,
//     capabilities: Capabilities,
//     byte_length: usize,
//     texture: Option<(WebGlTexture, WebGlSampler)>,
//     bindings: HashSet<TextureUnit>,
// }

// impl TextureRuntime {
//     fn get_or_create_texture(
//         &mut self,
//         layout: &TextureLayout,
//         texture_params: &[TextureParameter],
//         sampler_params: &[SamplerParameter],
//         registered: Option<&mut TextureRegistered>,
//     ) -> Result<(WebGlTexture, WebGlSampler), Error> {
//         match self.texture.as_ref() {
//             Some((texture, sampler)) => Ok((texture.clone(), sampler.clone())),
//             None => {
//                 if !self
//                     .capabilities
//                     .internal_format_supported(layout.internal_format)
//                 {
//                     return Err(Error::TextureInternalFormatUnsupported(
//                         layout.internal_format,
//                     ));
//                 }

//                 let texture = self
//                     .gl
//                     .create_texture()
//                     .ok_or(Error::CreateTextureFailure)?;
//                 let sampler = self
//                     .gl
//                     .create_sampler()
//                     .ok_or(Error::CreateSamplerFailure)?;

//                 let target = layout.target();
//                 let binding = if cfg!(feature = "rebind") {
//                     self.gl.texture_binding(target)
//                 } else {
//                     None
//                 };

//                 self.gl.bind_texture(target.gl_enum(), Some(&texture));

//                 // sets sampler parameters
//                 for param in sampler_params {
//                     param.set(&self.gl, &sampler);
//                 }
//                 // sets texture parameters
//                 for param in texture_params {
//                     param.set(&self.gl, target, &self.capabilities);
//                 }
//                 match target {
//                     TextureTarget::Texture2d | TextureTarget::TextureCubeMap => {
//                         self.gl.tex_storage_2d(
//                             target.gl_enum(),
//                             layout.levels as i32,
//                             layout.internal_format.gl_enum(),
//                             layout.width as i32,
//                             layout.height as i32,
//                         )
//                     }
//                     TextureTarget::Texture2dArray | TextureTarget::Texture3d => {
//                         self.gl.tex_storage_3d(
//                             target.gl_enum(),
//                             layout.levels as i32,
//                             layout.internal_format.gl_enum(),
//                             layout.width as i32,
//                             layout.height as i32,
//                             layout.depth as i32,
//                         )
//                     }
//                 }

//                 self.gl.bind_texture(target.gl_enum(), binding.as_ref());
//                 self.byte_length = layout.byte_length();

//                 let (texture, sampler) = self.texture.insert((texture, sampler));

//                 if let Some(registered) = registered {
//                     if let Some(store) = registered.store.upgrade() {
//                         store.borrow_mut().increase_used_memory(self.byte_length);
//                     }
//                 }

//                 Ok((texture.clone(), sampler.clone()))
//             }
//         }
//     }

//     fn upload(&self, layout: &TextureLayout, queue: &mut Vec<QueueItem>) -> Result<(), Error> {
//         let internal_format = layout.internal_format();
//         for QueueItem {
//             source,
//             target,
//             cube_map_face,
//             generate_mipmaps,
//             level,
//             x_offset,
//             y_offset,
//             z_offset,
//             width,
//             height,
//             depth,
//         } in queue.drain(..)
//         {
//             let is_3d = match target {
//                 TextureTarget::Texture2d | TextureTarget::TextureCubeMap => false,
//                 TextureTarget::Texture2dArray | TextureTarget::Texture3d => true,
//             };
//             let target = if target == TextureTarget::TextureCubeMap {
//                 // unwrap_or should never reach
//                 match cube_map_face.unwrap_or(TextureCubeMapFace::PositiveX) {
//                     TextureCubeMapFace::PositiveX => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X
//                     }
//                     TextureCubeMapFace::NegativeX => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X
//                     }
//                     TextureCubeMapFace::PositiveY => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y
//                     }
//                     TextureCubeMapFace::NegativeY => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y
//                     }
//                     TextureCubeMapFace::PositiveZ => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z
//                     }
//                     TextureCubeMapFace::NegativeZ => {
//                         WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z
//                     }
//                 }
//             } else {
//                 target.gl_enum()
//             };
//             let data = source.data();
//             let level = level as i32;
//             let x_offset = x_offset.unwrap_or(0) as i32;
//             let y_offset = y_offset.unwrap_or(0) as i32;
//             let z_offset = z_offset.unwrap_or(0) as i32;
//             let width = width.unwrap_or(data.width()) as i32;
//             let height = height.unwrap_or(data.height()) as i32;
//             let depth = depth.unwrap_or(0) as i32;

//             match (data, internal_format) {
//                 (
//                     TextureData::Uncompressed {
//                         data,
//                         pixel_format,
//                         pixel_storages,
//                         pixel_data_type,
//                     },
//                     TextureInternalFormat::Uncompressed(_),
//                 ) => {
//                     for storage in &pixel_storages {
//                         storage.set(&self.gl);
//                     }

//                     let result = match data {
//                         TextureUncompressedData::Bytes { .. }
//                         | TextureUncompressedData::BytesBorrowed { .. } => {
//                             enum Data<'a> {
//                                 Borrowed(&'a [u8]),
//                                 Owned(Box<dyn AsRef<[u8]>>),
//                             }

//                             impl<'a> Data<'a> {
//                                 fn as_bytes(&self) -> &[u8] {
//                                     match self {
//                                         Data::Borrowed(data) => *data,
//                                         Data::Owned(data) => data.as_ref().as_ref(),
//                                     }
//                                 }
//                             }

//                             let (data, data_type, src_element_offset) = match data {
//                                 TextureUncompressedData::Bytes {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Data::Owned(data), pixel_data_type, src_element_offset),
//                                 TextureUncompressedData::BytesBorrowed {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Data::Borrowed(data), pixel_data_type, src_element_offset),
//                                 _ => unreachable!(),
//                             };

//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_opt_u8_array_and_src_offset(
//                                     target,
//                                     level,
//                                     x_offset,
//                                     y_offset,
//                                     z_offset,
//                                     width,
//                                     height,
//                                     depth,
//                                     pixel_format.gl_enum(),
//                                     data_type.gl_enum(),
//                                     Some(data.as_bytes()),
//                                     src_element_offset.unwrap_or(0) as u32,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
//                                     target,
//                                     level,
//                                     x_offset,
//                                     y_offset,
//                                     width,
//                                     height,
//                                     pixel_format.gl_enum(),
//                                     data_type.gl_enum(),
//                                     data.as_bytes(),
//                                     src_element_offset.unwrap_or(0) as u32,
//                                 )
//                             }
//                         }
//                         TextureUncompressedData::PixelBufferObject {
//                             buffer, pbo_offset, ..
//                         } => {
//                             let binding = if cfg!(feature = "rebind") {
//                                 self.gl.pixel_unpack_buffer_binding()
//                             } else {
//                                 None
//                             };

//                             self.gl.bind_buffer(
//                                 WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
//                                 Some(&buffer),
//                             );
//                             let result = if is_3d {
//                                 self.gl.tex_sub_image_3d_with_i32(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     pbo_offset.unwrap_or(0) as i32,
//                                 )
//                             } else {
//                                 self.gl
//                                     .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
//                                         target,
//                                         level as i32,
//                                         x_offset as i32,
//                                         y_offset as i32,
//                                         width as i32,
//                                         height as i32,
//                                         pixel_format.gl_enum(),
//                                         pixel_data_type.gl_enum(),
//                                         pbo_offset.unwrap_or(0) as i32,
//                                     )
//                             };

//                             self.gl.bind_buffer(
//                                 WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
//                                 binding.as_ref(),
//                             );
//                             result
//                         }
//                         TextureUncompressedData::Int8Array { .. }
//                         | TextureUncompressedData::Uint8Array { .. }
//                         | TextureUncompressedData::Uint8ClampedArray { .. }
//                         | TextureUncompressedData::Int16Array { .. }
//                         | TextureUncompressedData::Uint16Array { .. }
//                         | TextureUncompressedData::Int32Array { .. }
//                         | TextureUncompressedData::Uint32Array { .. }
//                         | TextureUncompressedData::Float32Array { .. }
//                         | TextureUncompressedData::DataView { .. } => {
//                             let (data, src_element_offset) = match data {
//                                 TextureUncompressedData::Int8Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Uint8Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Uint8ClampedArray {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Int16Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Uint16Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Int32Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Uint32Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::Float32Array {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 TextureUncompressedData::DataView {
//                                     data,
//                                     src_element_offset,
//                                     ..
//                                 } => (Object::from(data), src_element_offset),
//                                 _ => unreachable!(),
//                             };

//                             if is_3d {
//                                 self.gl
//                                     .tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
//                                         target,
//                                         level as i32,
//                                         x_offset as i32,
//                                         y_offset as i32,
//                                         z_offset as i32,
//                                         width as i32,
//                                         height as i32,
//                                         depth as i32,
//                                         pixel_format.gl_enum(),
//                                         pixel_data_type.gl_enum(),
//                                         Some(&data),
//                                         src_element_offset.unwrap_or(0) as u32,
//                                     )
//                             } else {
//                                 self.gl
//                                     .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
//                                         target,
//                                         level as i32,
//                                         x_offset as i32,
//                                         y_offset as i32,
//                                         width as i32,
//                                         height as i32,
//                                         pixel_format.gl_enum(),
//                                         pixel_data_type.gl_enum(),
//                                         &data,
//                                         src_element_offset.unwrap_or(0) as u32,
//                                     )
//                             }
//                         }
//                         TextureUncompressedData::HtmlCanvasElement { data } => {
//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_html_canvas_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             }
//                         }
//                         TextureUncompressedData::HtmlImageElement { data } => {
//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_html_image_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             }
//                         }
//                         TextureUncompressedData::HtmlVideoElement { data } => {
//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_html_video_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             }
//                         }
//                         TextureUncompressedData::ImageData { data } => {
//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_image_data(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             }
//                         }
//                         TextureUncompressedData::ImageBitmap { data } => {
//                             if is_3d {
//                                 self.gl.tex_sub_image_3d_with_image_bitmap(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             } else {
//                                 self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     pixel_data_type.gl_enum(),
//                                     &data,
//                                 )
//                             }
//                         }
//                     };

//                     if let Err(err) = result {
//                         return Err(Error::TextureUploadImageFailure(err.as_string()));
//                     }

//                     if generate_mipmaps {
//                         self.gl.generate_mipmap(target);
//                     }

//                     for storage in &pixel_storages {
//                         storage.reset(&self.gl);
//                     }
//                 }
//                 (
//                     TextureData::Compressed { data, pixel_format },
//                     TextureInternalFormat::Compressed(internal_format),
//                 ) => {
//                     if pixel_format != internal_format {
//                         return Err(Error::TextureInternalFormatMismatched);
//                     }

//                     match data {
//                         TextureCompressedData::Bytes { .. }
//                         | TextureCompressedData::BytesBorrowed { .. } => {
//                             enum Data<'a> {
//                                 Borrowed(&'a mut [u8]),
//                                 Owned(Box<dyn AsMut<[u8]>>),
//                             }

//                             impl<'a> Data<'a> {
//                                 fn as_bytes(&mut self) -> &mut [u8] {
//                                     match self {
//                                         Data::Borrowed(data) => *data,
//                                         Data::Owned(data) => data.as_mut().as_mut(),
//                                     }
//                                 }
//                             }

//                             let (mut data, src_element_offset, src_element_length_override) =
//                                 match data {
//                                     TextureCompressedData::Bytes {
//                                         data,
//                                         src_element_offset,
//                                         src_element_length_override,
//                                         ..
//                                     } => (
//                                         Data::Owned(data),
//                                         src_element_offset,
//                                         src_element_length_override,
//                                     ),
//                                     TextureCompressedData::BytesBorrowed {
//                                         data,
//                                         src_element_offset,
//                                         src_element_length_override,
//                                         ..
//                                     } => (
//                                         Data::Borrowed(data),
//                                         src_element_offset,
//                                         src_element_length_override,
//                                     ),
//                                     _ => unreachable!(),
//                                 };

//                             if is_3d {
//                                 self.gl.compressed_tex_sub_image_3d_with_u8_array_and_u32_and_src_length_override(
//                                     target,
//                                     level,
//                                     x_offset,
//                                     y_offset,
//                                     z_offset,
//                                     width,
//                                     height,
//                                     depth,
//                                     pixel_format.gl_enum(),
//                                     data.as_bytes(),
//                                     src_element_offset.unwrap_or(0) as u32,
//                                     src_element_length_override.unwrap_or(0) as u32,
//                                 )
//                             } else {
//                                 self.gl.compressed_tex_sub_image_2d_with_u8_array_and_u32_and_src_length_override(
//                                     target,
//                                     level,
//                                     x_offset,
//                                     y_offset,
//                                     width,
//                                     height,
//                                     pixel_format.gl_enum(),
//                                     data.as_bytes(),
//                                     src_element_offset.unwrap_or(0) as u32,
//                                     src_element_length_override.unwrap_or(0) as u32,
//                                 )
//                             }
//                         }
//                         TextureCompressedData::PixelBufferObject {
//                             width,
//                             height,
//                             buffer,
//                             image_size,
//                             pbo_offset,
//                         } => {
//                             let binding = if cfg!(feature = "rebind") {
//                                 self.gl.pixel_unpack_buffer_binding()
//                             } else {
//                                 None
//                             };

//                             self.gl.bind_buffer(
//                                 WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
//                                 Some(&buffer),
//                             );
//                             if is_3d {
//                                 self.gl.compressed_tex_sub_image_3d_with_i32_and_i32(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     image_size as i32,
//                                     pbo_offset.unwrap_or(0) as i32,
//                                 )
//                             } else {
//                                 self.gl.compressed_tex_sub_image_2d_with_i32_and_i32(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     image_size as i32,
//                                     pbo_offset.unwrap_or(0) as i32,
//                                 )
//                             };

//                             self.gl.bind_buffer(
//                                 WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
//                                 binding.as_ref(),
//                             );
//                         }
//                         TextureCompressedData::Int8Array { .. }
//                         | TextureCompressedData::Uint8Array { .. }
//                         | TextureCompressedData::Uint8ClampedArray { .. }
//                         | TextureCompressedData::Int16Array { .. }
//                         | TextureCompressedData::Uint16Array { .. }
//                         | TextureCompressedData::Int32Array { .. }
//                         | TextureCompressedData::Uint32Array { .. }
//                         | TextureCompressedData::Float32Array { .. }
//                         | TextureCompressedData::DataView { .. } => {
//                             let (
//                                 width,
//                                 height,
//                                 data,
//                                 src_element_offset,
//                                 src_element_length_override,
//                             ) = match data {
//                                 TextureCompressedData::Int8Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Uint8Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Uint8ClampedArray {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Int16Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Uint16Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Int32Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Uint32Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::Float32Array {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 TextureCompressedData::DataView {
//                                     width,
//                                     height,
//                                     data,
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 } => (
//                                     width,
//                                     height,
//                                     Object::from(data),
//                                     src_element_offset,
//                                     src_element_length_override,
//                                 ),
//                                 _ => unreachable!(),
//                             };

//                             if is_3d {
//                                 self.gl.compressed_tex_sub_image_3d_with_array_buffer_view_and_u32_and_src_length_override(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     z_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     depth as i32,
//                                     pixel_format.gl_enum(),
//                                     &data,
//                                     src_element_offset.unwrap_or(0) as u32,
//                                     src_element_length_override.unwrap_or(0) as u32,
//                                 )
//                             } else {
//                                 self.gl.compressed_tex_sub_image_2d_with_array_buffer_view_and_u32_and_src_length_override(
//                                     target,
//                                     level as i32,
//                                     x_offset as i32,
//                                     y_offset as i32,
//                                     width as i32,
//                                     height as i32,
//                                     pixel_format.gl_enum(),
//                                     &data,
//                                     src_element_offset.unwrap_or(0) as u32,
//                                     src_element_length_override.unwrap_or(0) as u32,
//                                 )
//                             }
//                         }
//                     }
//                 }
//                 (TextureData::Uncompressed { .. }, TextureInternalFormat::Compressed(_))
//                 | (TextureData::Compressed { .. }, TextureInternalFormat::Uncompressed(_)) => {
//                     return Err(Error::TextureInternalFormatMismatched);
//                 }
//             };
//         }

//         Ok(())
//     }
// }

// struct TextureRegistered {
//     store: Weak<RefCell<StoreShared>>,
//     lru_node: *mut LruNode<Uuid>,
// }

// struct TextureShared {
//     id: Uuid,
//     name: Option<Cow<'static, str>>,
//     layout: TextureLayout,
//     sampler_params: Vec<SamplerParameter>,
//     texture_params: Vec<TextureParameter>,
//     queue: Vec<QueueItem>,
//     registered: Option<TextureRegistered>,
//     runtime: Option<TextureRuntime>,
// }

// impl Drop for TextureShared {
//     fn drop(&mut self) {
//         if let Some(mut runtime) = self.runtime.take() {
//             if let Some((texture, sampler)) = runtime.texture.take() {
//                 let target = self.layout.target;

//                 let activing = if cfg!(feature = "rebind") {
//                     Some(runtime.gl.texture_active_texture_unit())
//                 } else {
//                     None
//                 };

//                 for unit in runtime.bindings.iter() {
//                     runtime.gl.active_texture(unit.gl_enum());
//                     runtime.gl.bind_sampler(unit.unit_index(), None);
//                     runtime.gl.bind_texture(target.gl_enum(), None);
//                 }

//                 runtime.gl.delete_texture(Some(&texture));
//                 runtime.gl.delete_sampler(Some(&sampler));

//                 if let Some(activing) = activing {
//                     runtime.gl.active_texture(activing);
//                 }
//             }

//             if let Some(registered) = self.registered.as_mut() {
//                 if let Some(store) = registered.store.upgrade() {
//                     store.borrow_mut().unregister(
//                         &self.id,
//                         registered.lru_node,
//                         runtime.byte_length,
//                         self.layout.target,
//                         runtime.bindings.iter(),
//                     );
//                 }
//             }
//         }
//     }
// }

// impl Debug for TextureShared {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Texture")
//             .field("layout", &self.layout)
//             .field("id", &self.id)
//             .field("name", &self.name)
//             .field("texture_params", &self.texture_params)
//             .field("sampler_params", &self.sampler_params)
//             .finish()
//     }
// }

// impl TextureShared {
//     fn init(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
//         if let Some(runtime) = self.runtime.as_ref() {
//             if &runtime.gl != gl {
//                 return Err(Error::TextureAlreadyInitialized);
//             } else {
//                 return Ok(());
//             }
//         }

//         self.runtime = Some(TextureRuntime {
//             capabilities: Capabilities::new(gl.clone()),
//             gl: gl.clone(),
//             byte_length: 0,
//             texture: None,
//             bindings: HashSet::new(),
//         });
//         Ok(())
//     }

//     fn bind(&mut self, unit: TextureUnit) -> Result<(), Error> {
//         let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

//         let target = self.layout.target();

//         if let Some(registered) = self.registered.as_mut() {
//             if let Some(store) = registered.store.upgrade() {
//                 if store.borrow().is_occupied(unit, target, &self.id) {
//                     return Err(Error::TextureTargetOccupied(unit, target));
//                 }
//             }
//         }

//         if runtime.bindings.contains(&unit) {
//             runtime.upload(&self.layout, &mut self.queue)?;

//             if let Some(registered) = self.registered.as_mut() {
//                 if let Some(store) = registered.store.upgrade() {
//                     let mut store = store.borrow_mut();
//                     store.update_lru(registered.lru_node);
//                     store.free();
//                 }
//             }
//         } else {
//             let (texture, sampler) = runtime.get_or_create_texture(
//                 &self.layout,
//                 &self.texture_params,
//                 &self.sampler_params,
//                 self.registered.as_mut(),
//             )?;
//             let active_texture_unit = if cfg!(feature = "rebind") {
//                 Some(runtime.gl.texture_active_texture_unit())
//             } else {
//                 None
//             };

//             runtime.gl.active_texture(unit.gl_enum());
//             runtime.gl.bind_texture(target.gl_enum(), Some(&texture));
//             runtime.gl.bind_sampler(unit.unit_index(), Some(&sampler));
//             runtime.upload(&self.layout, &mut self.queue)?;
//             runtime.bindings.insert(unit);

//             if let Some(unit) = active_texture_unit {
//                 runtime.gl.active_texture(unit);
//             }

//             if let Some(registered) = self.registered.as_mut() {
//                 if let Some(store) = registered.store.upgrade() {
//                     let mut store = store.borrow_mut();
//                     store.add_binding(unit, target, self.id);
//                     store.update_lru(registered.lru_node);
//                     store.free();
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn unbind(&mut self, unit: TextureUnit) -> Result<(), Error> {
//         let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

//         let target = self.layout.target();
//         if runtime.bindings.remove(&unit) {
//             let active_texture_unit = if cfg!(feature = "rebind") {
//                 Some(runtime.gl.texture_active_texture_unit())
//             } else {
//                 None
//             };

//             runtime.gl.active_texture(unit.gl_enum());
//             runtime.gl.bind_texture(target.gl_enum(), None);
//             runtime.gl.bind_sampler(unit.unit_index(), None);

//             if let Some(unit) = active_texture_unit {
//                 runtime.gl.active_texture(unit);
//             }

//             if let Some(registered) = self.registered.as_mut() {
//                 if let Some(store) = registered.store.upgrade() {
//                     store.borrow_mut().remove_binding(unit, target);
//                 }
//             }
//         }

//         Ok(())
//     }

//     fn unbind_all(&mut self) -> Result<(), Error> {
//         let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

//         let active_texture_unit = if cfg!(feature = "rebind") {
//             Some(runtime.gl.texture_active_texture_unit())
//         } else {
//             None
//         };

//         let target = self.layout.target();
//         for unit in runtime.bindings.drain() {
//             runtime.gl.active_texture(unit.unit_index());
//             runtime.gl.bind_texture(target.gl_enum(), None);

//             if let Some(registered) = self.registered.as_mut() {
//                 if let Some(store) = registered.store.upgrade() {
//                     store.borrow_mut().remove_binding(unit, target);
//                 }
//             }
//         }

//         if let Some(unit) = active_texture_unit {
//             runtime.gl.active_texture(unit);
//         }

//         Ok(())
//     }

//     fn upload(&mut self) -> Result<(), Error> {
//         let runtime = self.runtime.as_mut().ok_or(Error::TextureUninitialized)?;

//         let (texture, _) = runtime.get_or_create_texture(
//             &self.layout,
//             &self.texture_params,
//             &self.sampler_params,
//             self.registered.as_mut(),
//         )?;
//         let target = self.layout.target();
//         let binding = if cfg!(feature = "rebind") {
//             runtime.gl.texture_binding(target)
//         } else {
//             None
//         };
//         runtime.gl.bind_texture(target.gl_enum(), Some(&texture));
//         runtime.upload(&self.layout, &mut self.queue)?;
//         runtime.gl.bind_texture(target.gl_enum(), binding.as_ref());

//         Ok(())
//     }

//     fn set_texture_parameter(&mut self, texture_param: TextureParameter) {
//         let index = self
//             .texture_params
//             .iter()
//             .position(|p| p.kind() == texture_param.kind());
//         match index {
//             Some(index) => {
//                 let _ = std::mem::replace::<TextureParameter>(
//                     &mut self.texture_params[index],
//                     texture_param,
//                 );
//             }
//             None => self.texture_params.push(texture_param),
//         };

//         if let Some(runtime) = self.runtime.as_ref() {
//             if let Some((texture, _)) = runtime.texture.as_ref() {
//                 let target = self.layout.target();
//                 let binding = if cfg!(feature = "rebind") {
//                     runtime.gl.texture_binding(target)
//                 } else {
//                     None
//                 };

//                 runtime.gl.bind_texture(target.gl_enum(), Some(texture));
//                 texture_param.set(&runtime.gl, target, &runtime.capabilities);

//                 runtime.gl.bind_texture(target.gl_enum(), binding.as_ref());
//             }
//         }
//     }

//     fn set_texture_parameters<I>(&mut self, texture_params: I)
//     where
//         I: IntoIterator<Item = TextureParameter>,
//     {
//         texture_params
//             .into_iter()
//             .for_each(|param| self.set_texture_parameter(param))
//     }

//     fn set_sampler_parameter(&mut self, sampler_param: SamplerParameter) {
//         let index = self
//             .sampler_params
//             .iter()
//             .position(|p| p.kind() == sampler_param.kind());
//         match index {
//             Some(index) => {
//                 let _ = std::mem::replace::<SamplerParameter>(
//                     &mut self.sampler_params[index],
//                     sampler_param,
//                 );
//             }
//             None => self.sampler_params.push(sampler_param),
//         };

//         if let Some(runtime) = self.runtime.as_ref() {
//             if let Some((_, sampler)) = runtime.texture.as_ref() {
//                 sampler_param.set(&runtime.gl, &sampler);
//             }
//         }
//     }

//     fn set_sampler_parameters<I>(&mut self, sampler_params: I)
//     where
//         I: IntoIterator<Item = SamplerParameter>,
//     {
//         sampler_params
//             .into_iter()
//             .for_each(|param| self.set_sampler_parameter(param))
//     }
// }

// #[derive(Debug, Clone)]
// pub struct TextureUnbinder {
//     unit: TextureUnit,
//     shared: Weak<RefCell<TextureShared>>,
// }

// impl TextureUnbinder {
//     /// Unbinds texture.
//     pub fn unbind(self) {
//         let Some(shared) = self.shared.upgrade() else {
//             return;
//         };
//         let _ = shared.borrow_mut().unbind(self.unit);
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Texture<L> {
//     layout: PhantomData<L>,
//     shared: Rc<RefCell<TextureShared>>,
// }

// impl<L> Texture<L> {
//     /// Returns id of this buffer.
//     pub fn id(&self) -> Uuid {
//         self.shared.borrow().id
//     }

//     /// Sets name.
//     pub fn set_name(&self, name: Option<Cow<'static, str>>) {
//         self.shared.borrow_mut().name = name;
//     }

//     /// Returns name.
//     pub fn name(&self) -> Option<String> {
//         match self.shared.borrow().name.as_ref() {
//             Some(name) => Some(name.to_string()),
//             None => None,
//         }
//     }

//     /// Initializes texture.
//     pub fn init(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
//         self.shared.borrow_mut().init(gl)
//     }

//     /// Binds texture to specified target in specified texture unit.
//     pub fn bind(&self, unit: TextureUnit) -> Result<TextureUnbinder, Error> {
//         self.shared.borrow_mut().bind(unit)?;
//         Ok(TextureUnbinder {
//             unit,
//             shared: Rc::downgrade(&self.shared),
//         })
//     }

//     /// Unbinds texture from specified target in specified texture unit.
//     pub fn unbind(&self, unit: TextureUnit) -> Result<(), Error> {
//         self.shared.borrow_mut().unbind(unit)
//     }

//     /// Unbinds texture from all bound texture unit.
//     pub fn unbind_all(&self) -> Result<(), Error> {
//         self.shared.borrow_mut().unbind_all()
//     }

//     /// Uploads texture data to WebGL runtime.
//     pub fn upload(&self) -> Result<(), Error> {
//         self.shared.borrow_mut().upload()
//     }

//     /// Returns a list of texture parameters.
//     pub fn texture_parameters(&self) -> Vec<TextureParameter> {
//         self.shared.borrow().texture_params.clone()
//     }

//     /// Sets texture parameter.
//     pub fn set_texture_parameter(&self, texture_param: TextureParameter) {
//         self.shared
//             .borrow_mut()
//             .set_texture_parameter(texture_param)
//     }

//     /// Sets texture parameters.
//     pub fn set_texture_parameters<I>(&self, texture_params: I)
//     where
//         I: IntoIterator<Item = TextureParameter>,
//     {
//         self.shared
//             .borrow_mut()
//             .set_texture_parameters(texture_params)
//     }

//     /// Returns a list of sampler parameters.
//     pub fn sampler_parameters(&self) -> Vec<SamplerParameter> {
//         self.shared.borrow().sampler_params.clone()
//     }

//     /// Sets sampler parameter.
//     pub fn set_sampler_parameter(&self, sampler_param: SamplerParameter) {
//         self.shared
//             .borrow_mut()
//             .set_sampler_parameter(sampler_param)
//     }

//     /// Sets sampler parameters.
//     pub fn set_sampler_parameters<I>(&self, sampler_params: I)
//     where
//         I: IntoIterator<Item = SamplerParameter>,
//     {
//         self.shared
//             .borrow_mut()
//             .set_sampler_parameters(sampler_params)
//     }
// }

// impl Texture<Texture2D> {
//     /// Creates a new 2d texture.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         texture_params: Vec<TextureParameter>,
//         sampler_params: Vec<SamplerParameter>,
//     ) -> Self {
//         let shared = Rc::new(RefCell::new(TextureShared {
//             id: Uuid::new_v4(),
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture2d,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: 0,
//             },
//             texture_params,
//             sampler_params,
//             queue: Vec::new(),
//             registered: None,
//             runtime: None,
//         }));

//         Self {
//             layout: PhantomData,
//             shared,
//         }
//     }

//     /// Returns texture target.
//     pub fn target(&self) -> TextureTarget {
//         self.shared.borrow().layout.target
//     }

//     /// Returns texture internal format.
//     pub fn internal_format(&self) -> TextureInternalFormat {
//         self.shared.borrow().layout.internal_format
//     }

//     /// Returns mipmap levels.
//     pub fn levels(&self) -> usize {
//         self.shared.borrow().layout.levels
//     }

//     /// Returns texture width at level 0.
//     pub fn width(&self) -> usize {
//         self.shared.borrow().layout.width
//     }

//     /// Returns texture height at level 0.
//     pub fn height(&self) -> usize {
//         self.shared.borrow().layout.height
//     }

//     /// Returns texture width at specified level.
//     pub fn width_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let width = (layout.width >> level).max(1);
//         Some(width)
//     }

//     /// Returns texture height at specified level.
//     pub fn height_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let height = (layout.height >> level).max(1);
//         Some(height)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         width: usize,
//         height: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: None,
//             width: Some(width),
//             height: Some(height),
//             depth: None,
//         });
//     }
// }

// impl Texture<Texture2DArray> {
//     /// Creates a new 2d array texture.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         len: usize,
//         texture_params: Vec<TextureParameter>,
//         sampler_params: Vec<SamplerParameter>,
//     ) -> Self {
//         let shared = Rc::new(RefCell::new(TextureShared {
//             id: Uuid::new_v4(),
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture2dArray,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: len,
//             },
//             texture_params,
//             sampler_params,
//             queue: Vec::new(),
//             registered: None,
//             runtime: None,
//         }));

//         Self {
//             layout: PhantomData,
//             shared,
//         }
//     }

//     /// Returns texture target.
//     pub fn target(&self) -> TextureTarget {
//         self.shared.borrow().layout.target
//     }

//     /// Returns texture internal format.
//     pub fn internal_format(&self) -> TextureInternalFormat {
//         self.shared.borrow().layout.internal_format
//     }

//     /// Returns texture mipmap levels.
//     pub fn levels(&self) -> usize {
//         self.shared.borrow().layout.levels
//     }

//     /// Returns texture width at level 0.
//     pub fn width(&self) -> usize {
//         self.shared.borrow().layout.width
//     }

//     /// Returns texture height at level 0.
//     pub fn height(&self) -> usize {
//         self.shared.borrow().layout.height
//     }

//     /// Returns texture array length.
//     pub fn len(&self) -> usize {
//         self.shared.borrow().layout.depth
//     }

//     /// Returns texture width at specified level.
//     pub fn width_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let width = (layout.width >> level).max(1);
//         Some(width)
//     }

//     /// Returns texture height at specified level.
//     pub fn height_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let height = (layout.height >> level).max(1);
//         Some(height)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2dArray,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         array_index_offset: usize,
//         width: usize,
//         height: usize,
//         array_index: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2dArray,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: Some(array_index_offset),
//             width: Some(width),
//             height: Some(height),
//             depth: Some(array_index),
//         });
//     }
// }

// impl Texture<Texture3D> {
//     /// Creates a new 3d texture.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         depth: usize,
//         texture_params: Vec<TextureParameter>,
//         sampler_params: Vec<SamplerParameter>,
//     ) -> Self {
//         let shared = Rc::new(RefCell::new(TextureShared {
//             id: Uuid::new_v4(),
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture3d,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth,
//             },
//             texture_params,
//             sampler_params,
//             queue: Vec::new(),
//             registered: None,
//             runtime: None,
//         }));

//         Self {
//             layout: PhantomData,
//             shared,
//         }
//     }

//     /// Returns texture target.
//     pub fn target(&self) -> TextureTarget {
//         self.shared.borrow().layout.target
//     }

//     /// Returns texture internal format.
//     pub fn internal_format(&self) -> TextureInternalFormat {
//         self.shared.borrow().layout.internal_format
//     }

//     /// Returns mipmap levels.
//     pub fn levels(&self) -> usize {
//         self.shared.borrow().layout.levels
//     }

//     /// Returns texture width at level 0.
//     pub fn width(&self) -> usize {
//         self.shared.borrow().layout.width
//     }

//     /// Returns texture height at level 0.
//     pub fn height(&self) -> usize {
//         self.shared.borrow().layout.height
//     }

//     /// Returns texture depth at level 0.
//     pub fn depth(&self) -> usize {
//         self.shared.borrow().layout.depth
//     }

//     /// Returns texture width at specified level.
//     pub fn width_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let width = (layout.width >> level).max(1);
//         Some(width)
//     }

//     /// Returns texture height at specified level.
//     pub fn height_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let height = (layout.height >> level).max(1);
//         Some(height)
//     }

//     /// Returns texture depth at specified level.
//     pub fn depth_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let depth = (layout.depth >> level).max(1);
//         Some(depth)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture3d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         z_offset: usize,
//         width: usize,
//         height: usize,
//         depth: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture3d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: Some(z_offset),
//             width: Some(width),
//             height: Some(height),
//             depth: Some(depth),
//         });
//     }
// }

// impl Texture<TextureCubeMap> {
//     /// Creates a new cube map texture.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         texture_params: Vec<TextureParameter>,
//         sampler_params: Vec<SamplerParameter>,
//     ) -> Self {
//         let shared = Rc::new(RefCell::new(TextureShared {
//             id: Uuid::new_v4(),
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::TextureCubeMap,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: 0,
//             },
//             texture_params,
//             sampler_params,
//             queue: Vec::new(),
//             registered: None,
//             runtime: None,
//         }));

//         Self {
//             layout: PhantomData,
//             shared,
//         }
//     }

//     /// Returns texture target.
//     pub fn target(&self) -> TextureTarget {
//         self.shared.borrow().layout.target
//     }

//     /// Returns texture internal format.
//     pub fn internal_format(&self) -> TextureInternalFormat {
//         self.shared.borrow().layout.internal_format
//     }

//     /// Returns texture mipmap levels.
//     pub fn levels(&self) -> usize {
//         self.shared.borrow().layout.levels
//     }

//     /// Returns texture width at level 0.
//     pub fn width(&self) -> usize {
//         self.shared.borrow().layout.width
//     }

//     /// Returns texture height at level 0.
//     pub fn height(&self) -> usize {
//         self.shared.borrow().layout.height
//     }

//     /// Returns texture width at specified level.
//     pub fn width_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let width = (layout.width >> level).max(1);
//         Some(width)
//     }

//     /// Returns texture height at specified level.
//     pub fn height_of_level(&self, level: usize) -> Option<usize> {
//         let layout = self.shared.borrow().layout;

//         if level > layout.levels {
//             return None;
//         }

//         let height = (layout.height >> level).max(1);
//         Some(height)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(
//         &self,
//         source: S,
//         face: TextureCubeMapFace,
//         level: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::TextureCubeMap,
//             cube_map_face: Some(face),
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &self,
//         source: S,
//         face: TextureCubeMapFace,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         width: usize,
//         height: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.shared.borrow_mut().queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::TextureCubeMap,
//             cube_map_face: Some(face),
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: None,
//             width: Some(width),
//             height: Some(height),
//             depth: None,
//         });
//     }
// }

// pub struct Builder<T> {
//     name: Option<Cow<'static, str>>,
//     layout: TextureLayout,
//     texture_params: Vec<TextureParameter>,
//     sampler_params: Vec<SamplerParameter>,
//     queue: Vec<QueueItem>,
// }

// impl<T> Builder<T> {
//     /// Sets texture name.
//     pub fn set_name<S>(&mut self, name: S)
//     where
//         S: Into<String>,
//     {
//         self.name = Some(Cow::Owned(name.into()));
//     }

//     /// Sets texture name by static str.
//     pub fn set_name_str(&mut self, name: &'static str) {
//         self.name = Some(Cow::Borrowed(name.into()));
//     }

//     /// Sets a single texture parameters.
//     pub fn set_texture_parameter(&mut self, texture_param: TextureParameter) {
//         let old = self
//             .texture_params
//             .iter()
//             .position(|p| p.kind() == texture_param.kind());
//         match old {
//             Some(index) => {
//                 let _ = std::mem::replace(&mut self.texture_params[index], texture_param);
//             }
//             None => self.texture_params.push(texture_param),
//         }
//     }

//     /// Sets texture parameters.
//     pub fn set_texture_parameters<I: IntoIterator<Item = TextureParameter>>(
//         &mut self,
//         texture_params: I,
//     ) {
//         self.texture_params.clear();
//         self.texture_params.extend(texture_params);
//     }

//     /// Sets a single sampler parameters.
//     pub fn set_sampler_parameter(&mut self, sampler_param: SamplerParameter) {
//         let old = self
//             .sampler_params
//             .iter()
//             .position(|p| p.kind() == sampler_param.kind());
//         match old {
//             Some(index) => {
//                 let _ = std::mem::replace(&mut self.sampler_params[index], sampler_param);
//             }
//             None => self.sampler_params.push(sampler_param),
//         }
//     }

//     /// Sets sampler parameters.
//     pub fn set_sampler_parameters<I: IntoIterator<Item = SamplerParameter>>(
//         &mut self,
//         sampler_params: I,
//     ) {
//         self.sampler_params.clear();
//         self.sampler_params.extend(sampler_params);
//     }
// }

// impl Builder<Texture2D> {
//     /// Creates a new 2d texture builder.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//     ) -> Self {
//         Self {
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture2d,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: 0,
//             },
//             texture_params: Vec::new(),
//             sampler_params: Vec::new(),
//             queue: Vec::new(),
//         }
//     }

//     /// Creates a new 2d texture builder with automatically calculated mipmaps levels.
//     pub fn with_auto_levels(
//         internal_format: TextureInternalFormat,
//         width: usize,
//         height: usize,
//     ) -> Self {
//         let levels = (width.max(height) as f64).log2().floor() as usize + 1;
//         Self::new(internal_format, levels, width, height)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &mut self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         width: usize,
//         height: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: None,
//             width: Some(width),
//             height: Some(height),
//             depth: None,
//         });
//     }

//     /// Builds 2d texture.
//     pub fn build(self) -> Texture<Texture2D> {
//         let shared = TextureShared {
//             id: Uuid::new_v4(),
//             name: self.name,
//             layout: self.layout,
//             sampler_params: self.sampler_params,
//             texture_params: self.texture_params,
//             queue: self.queue,
//             registered: None,
//             runtime: None,
//         };
//         Texture::<Texture2D> {
//             layout: PhantomData,
//             shared: Rc::new(RefCell::new(shared)),
//         }
//     }
// }

// impl Builder<Texture2DArray> {
//     /// Creates a new 2d array texture builder.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         len: usize,
//     ) -> Self {
//         Self {
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture2dArray,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: len,
//             },
//             texture_params: Vec::new(),
//             sampler_params: Vec::new(),
//             queue: Vec::new(),
//         }
//     }

//     /// Creates a new 2d array texture builder with automatically calculated mipmaps levels.
//     pub fn with_auto_levels(
//         internal_format: TextureInternalFormat,
//         width: usize,
//         height: usize,
//         len: usize,
//     ) -> Self {
//         let levels = (width.max(height) as f64).log2().floor() as usize + 1;
//         Self::new(internal_format, levels, width, height, len)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2dArray,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &mut self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         array_index_offset: usize,
//         width: usize,
//         height: usize,
//         array_index: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture2dArray,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: Some(array_index_offset),
//             width: Some(width),
//             height: Some(height),
//             depth: Some(array_index),
//         });
//     }

//     /// Builds 2d array texture.
//     pub fn build(self) -> Texture<Texture2DArray> {
//         let shared = TextureShared {
//             id: Uuid::new_v4(),
//             name: self.name,
//             layout: self.layout,
//             sampler_params: self.sampler_params,
//             texture_params: self.texture_params,
//             queue: self.queue,
//             registered: None,
//             runtime: None,
//         };
//         Texture::<Texture2DArray> {
//             layout: PhantomData,
//             shared: Rc::new(RefCell::new(shared)),
//         }
//     }
// }

// impl Builder<Texture3D> {
//     /// Creates a new 3d texture builder.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//         depth: usize,
//     ) -> Self {
//         Self {
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::Texture3d,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth,
//             },
//             texture_params: Vec::new(),
//             sampler_params: Vec::new(),
//             queue: Vec::new(),
//         }
//     }

//     /// Creates a new 3d texture builder with automatically calculated mipmaps levels.
//     pub fn with_auto_levels(
//         internal_format: TextureInternalFormat,
//         width: usize,
//         height: usize,
//         depth: usize,
//     ) -> Self {
//         let levels = (width.max(height).max(depth) as f64).log2().floor() as usize + 1;
//         Self::new(internal_format, levels, width, height, depth)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(&mut self, source: S, level: usize, generate_mipmaps: bool)
//     where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture3d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &mut self,
//         source: S,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         z_offset: usize,
//         width: usize,
//         height: usize,
//         depth: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::Texture3d,
//             cube_map_face: None,
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: Some(z_offset),
//             width: Some(width),
//             height: Some(height),
//             depth: Some(depth),
//         });
//     }

//     /// Builds 3d texture.
//     pub fn build(self) -> Texture<Texture3D> {
//         let shared = TextureShared {
//             id: Uuid::new_v4(),
//             name: self.name,
//             layout: self.layout,
//             sampler_params: self.sampler_params,
//             texture_params: self.texture_params,
//             queue: self.queue,
//             registered: None,
//             runtime: None,
//         };
//         Texture::<Texture3D> {
//             layout: PhantomData,
//             shared: Rc::new(RefCell::new(shared)),
//         }
//     }
// }

// impl Builder<TextureCubeMap> {
//     /// Creates a new cube map texture builder.
//     pub fn new(
//         internal_format: TextureInternalFormat,
//         levels: usize,
//         width: usize,
//         height: usize,
//     ) -> Self {
//         Self {
//             name: None,
//             layout: TextureLayout {
//                 target: TextureTarget::TextureCubeMap,
//                 internal_format,
//                 levels,
//                 width,
//                 height,
//                 depth: 0,
//             },
//             texture_params: Vec::new(),
//             sampler_params: Vec::new(),
//             queue: Vec::new(),
//         }
//     }

//     /// Creates a new cube map texture builder with automatically calculated mipmaps levels.
//     pub fn with_auto_levels(
//         internal_format: TextureInternalFormat,
//         width: usize,
//         height: usize,
//     ) -> Self {
//         let levels = (width.max(height) as f64).log2().floor() as usize + 1;
//         Self::new(internal_format, levels, width, height)
//     }

//     /// Uploads new texture image data.
//     pub fn tex_image<S>(
//         &mut self,
//         source: S,
//         face: TextureCubeMapFace,
//         level: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::TextureCubeMap,
//             cube_map_face: Some(face),
//             generate_mipmaps,
//             level,
//             x_offset: None,
//             y_offset: None,
//             z_offset: None,
//             width: None,
//             height: None,
//             depth: None,
//         });
//     }

//     /// Uploads new texture sub image data.
//     pub fn tex_sub_image<S>(
//         &mut self,
//         source: S,
//         face: TextureCubeMapFace,
//         level: usize,
//         x_offset: usize,
//         y_offset: usize,
//         width: usize,
//         height: usize,
//         generate_mipmaps: bool,
//     ) where
//         S: TextureSource + 'static,
//     {
//         self.queue.push(QueueItem {
//             source: Box::new(source),
//             target: TextureTarget::TextureCubeMap,
//             cube_map_face: Some(face),
//             generate_mipmaps,
//             level,
//             x_offset: Some(x_offset),
//             y_offset: Some(y_offset),
//             z_offset: None,
//             width: Some(width),
//             height: Some(height),
//             depth: None,
//         });
//     }

//     /// Builds 3d texture.
//     pub fn build(self) -> Texture<TextureCubeMap> {
//         let shared = TextureShared {
//             id: Uuid::new_v4(),
//             name: self.name,
//             layout: self.layout,
//             sampler_params: self.sampler_params,
//             texture_params: self.texture_params,
//             queue: self.queue,
//             registered: None,
//             runtime: None,
//         };
//         Texture::<TextureCubeMap> {
//             layout: PhantomData,
//             shared: Rc::new(RefCell::new(shared)),
//         }
//     }
// }

// struct StoreShared {
//     gl: WebGl2RenderingContext,
//     id: Uuid,

//     available_memory: usize,
//     used_memory: usize,

//     lru: Lru<Uuid>,
//     textures: HashMap<Uuid, Weak<RefCell<TextureShared>>>,
//     bindings: HashMap<(TextureUnit, TextureTarget), Uuid>,
// }

// impl Drop for StoreShared {
//     fn drop(&mut self) {
//         for texture in self.textures.values_mut() {
//             let Some(texture) = texture.upgrade() else {
//                 continue;
//             };
//             texture.borrow_mut().registered = None;
//         }
//     }
// }

// impl StoreShared {
//     fn update_lru(&mut self, lru_node: *mut LruNode<Uuid>) {
//         unsafe {
//             self.lru.cache(lru_node);
//         }
//     }

//     fn increase_used_memory(&mut self, byte_length: usize) {
//         self.used_memory += byte_length;
//     }

//     fn decrease_used_memory(&mut self, byte_length: usize) {
//         self.used_memory -= byte_length;
//     }

//     fn add_binding(&mut self, unit: TextureUnit, target: TextureTarget, id: Uuid) {
//         self.bindings.insert((unit, target), id);
//     }

//     fn remove_binding(&mut self, unit: TextureUnit, target: TextureTarget) {
//         self.bindings.remove(&(unit, target));
//     }

//     fn remove(&mut self, byte_length: usize, lru_node: *mut LruNode<Uuid>) {
//         self.decrease_used_memory(byte_length);
//         unsafe {
//             self.lru.remove(lru_node);
//         }
//     }

//     fn is_occupied(&self, unit: TextureUnit, target: TextureTarget, id: &Uuid) -> bool {
//         self.bindings
//             .get(&(unit, target))
//             .map(|v| v != id)
//             .unwrap_or(false)
//     }

//     fn free(&mut self) {
//         unsafe {
//             if self.used_memory <= self.available_memory {
//                 return;
//             }

//             let mut next_node = self.lru.least_recently();
//             while self.used_memory > self.available_memory {
//                 let Some(current_node) = next_node.take() else {
//                     break;
//                 };
//                 let id = (*current_node).data();
//                 let Entry::Occupied(occupied) = self.textures.entry(*id) else {
//                     next_node = (*current_node).more_recently();
//                     continue;
//                 };

//                 let texture = occupied.get();
//                 let Some(texture) = texture.upgrade() else {
//                     occupied.remove();
//                     next_node = (*current_node).more_recently();
//                     continue;
//                 };

//                 if let Ok(mut texture) = texture.try_borrow_mut() {
//                     if !texture.free() {
//                         next_node = (*current_node).more_recently();
//                         continue;
//                     }
//                 }

//                 occupied.remove();
//                 next_node = (*current_node).more_recently();
//             }
//         }
//     }

//     fn unregister<'a, B>(
//         &mut self,
//         id: &Uuid,
//         lru_node: *mut LruNode<Uuid>,
//         byte_length: usize,
//         target: TextureTarget,
//         bindings: B,
//     ) where
//         B: IntoIterator<Item = &'a TextureUnit>,
//     {
//         bindings.into_iter().for_each(|unit| {
//             self.bindings.remove(&(*unit, target));
//         });
//         self.used_memory -= byte_length;
//         self.textures.remove(id);
//         unsafe {
//             self.lru.remove(lru_node);
//         }
//     }
// }

// pub struct TextureStore {
//     shared: Rc<RefCell<StoreShared>>,
// }

// impl TextureStore {
//     /// Constructs a new texture store with [`i32::MAX`] bytes memory limitation.
//     pub fn new(gl: WebGl2RenderingContext) -> Self {
//         Self::with_available_memory(gl, i32::MAX as usize)
//     }

//     /// Constructs a new texture store with a maximum available memory.
//     /// Maximum available memory is clamped to [`i32::MAX`] if larger than [`i32::MAX`];
//     pub fn with_available_memory(gl: WebGl2RenderingContext, available_memory: usize) -> Self {
//         let shared = StoreShared {
//             gl,
//             id: Uuid::new_v4(),

//             available_memory,
//             used_memory: 0,

//             lru: Lru::new(),
//             textures: HashMap::new(),
//             bindings: HashMap::new(),
//         };

//         Self {
//             shared: Rc::new(RefCell::new(shared)),
//         }
//     }

//     /// Returns store id.
//     pub fn id(&self) -> Uuid {
//         self.shared.borrow().id
//     }

//     /// Returns the maximum available memory in bytes.
//     /// Returns [`i32::MAX`] if not specified.
//     pub fn available_memory(&self) -> usize {
//         self.shared.borrow().available_memory
//     }

//     /// Returns current used memory in bytes.
//     pub fn used_memory(&self) -> usize {
//         self.shared.borrow().used_memory
//     }

//     /// Registers a texture to store, and initializes the texture.
//     pub fn register<L>(&self, texture: &Texture<L>) -> Result<(), Error> {
//         unsafe {
//             let mut store_shared = self.shared.borrow_mut();
//             let mut texture_shared = texture.shared.borrow_mut();

//             if let Some(store) = texture_shared
//                 .registered
//                 .as_ref()
//                 .and_then(|registered| registered.store.upgrade())
//             {
//                 if let Ok(store) = store.try_borrow() {
//                     if &store.id != &store_shared.id {
//                         return Err(Error::RegisterTextureToMultipleStore);
//                     } else {
//                         return Ok(());
//                     }
//                 } else {
//                     // if store is borrowed, it means that store of registered is the same store as self.
//                     return Ok(());
//                 }
//             }

//             texture_shared.init(&store_shared.gl)?;

//             let runtime = texture_shared.runtime.as_ref().unwrap();
//             store_shared.used_memory += runtime.byte_length;
//             let target = texture_shared.layout.target;
//             for unit in &runtime.bindings {
//                 let key = (*unit, target);
//                 if store_shared.bindings.contains_key(&key) {
//                     return Err(Error::TextureTargetOccupied(*unit, target));
//                 }
//                 store_shared.bindings.insert(key, texture_shared.id);
//             }

//             texture_shared.registered = Some(TextureRegistered {
//                 store: Rc::downgrade(&self.shared),
//                 lru_node: LruNode::new(texture_shared.id),
//             });

//             store_shared
//                 .textures
//                 .insert(texture_shared.id, Rc::downgrade(&texture.shared));

//             Ok(())
//         }
//     }

//     /// Unregisters a texture from store.
//     pub fn unregister<L>(&self, texture: &Texture<L>) {
//         unsafe {
//             let mut store_shared = self.shared.borrow_mut();
//             let mut texture_shared = texture.shared.borrow_mut();

//             if store_shared.textures.remove(&texture_shared.id).is_none() {
//                 return;
//             }

//             let runtime = texture_shared.runtime.as_ref().unwrap();
//             store_shared.used_memory -= runtime.byte_length;
//             let target = texture_shared.layout.target;
//             for unit in &runtime.bindings {
//                 let key = (*unit, target);
//                 if let Entry::Occupied(entry) = store_shared.bindings.entry(key) {
//                     if &texture_shared.id == entry.get() {
//                         entry.remove();
//                     }
//                 }
//             }

//             if let Some(registered) = texture_shared.registered.take() {
//                 store_shared.lru.remove(registered.lru_node);
//             }
//         }
//     }
// }
