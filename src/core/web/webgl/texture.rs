use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};

use hashbrown::{HashMap, HashSet};
use js_sys::{
    DataView, Float32Array, Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
use uuid::Uuid;
use web_sys::{
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use super::{buffer::BufferTarget, conversion::ToGlEnum, error::Error};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureTarget {
    Texture2D,
    TextureCubeMap,
    Texture2DArray,
    Texture3D,
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
pub enum TexturePixelDataType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TexturePixelAlignment {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

/// Available texture pixel storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePackPixelStorage {
    PackAlignment(TexturePixelAlignment),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
}

// impl TexturePackPixelStorage {
//     fn pixel_store(&self, gl: &WebGl2RenderingContext) -> TexturePackPixelStorage {
//         match self {
//             TexturePackPixelStorage::PackAlignment(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, *v as i32);
//                 TexturePackPixelStorage::PackAlignment(TexturePixelAlignment::Four)
//             }
//             TexturePackPixelStorage::PackRowLength(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, *v);
//                 TexturePackPixelStorage::PackRowLength(0)
//             }
//             TexturePackPixelStorage::PackSkipPixels(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, *v);
//                 TexturePackPixelStorage::PackSkipPixels(0)
//             }
//             TexturePackPixelStorage::PackSkipRows(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, *v);
//                 TexturePackPixelStorage::PackSkipRows(0)
//             }
//         }
//     }
// }

/// Available texture pixel storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackPixelStorage {
    UnpackAlignment(TexturePixelAlignment),
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
    fn pixel_store(&self, gl: &WebGl2RenderingContext) -> TextureUnpackPixelStorage {
        match self {
            TextureUnpackPixelStorage::UnpackAlignment(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, *v as i32);
                TextureUnpackPixelStorage::UnpackAlignment(TexturePixelAlignment::Four)
            }
            TextureUnpackPixelStorage::UnpackFlipY(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
                    if *v { 1 } else { 0 },
                );
                TextureUnpackPixelStorage::UnpackFlipY(false)
            }
            TextureUnpackPixelStorage::UnpackPremultiplyAlpha(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                    if *v { 1 } else { 0 },
                );
                TextureUnpackPixelStorage::UnpackPremultiplyAlpha(false)
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
                TextureUnpackPixelStorage::UnpackColorSpaceConversion(
                    TextureUnpackColorSpaceConversion::BrowserDefault,
                )
            }
            TextureUnpackPixelStorage::UnpackRowLength(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, *v);
                TextureUnpackPixelStorage::UnpackRowLength(0)
            }
            TextureUnpackPixelStorage::UnpackImageHeight(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, *v);
                TextureUnpackPixelStorage::UnpackImageHeight(0)
            }
            TextureUnpackPixelStorage::UnpackSkipPixels(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, *v);
                TextureUnpackPixelStorage::UnpackSkipPixels(0)
            }
            TextureUnpackPixelStorage::UnpackSkipRows(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, *v);
                TextureUnpackPixelStorage::UnpackSkipRows(0)
            }
            TextureUnpackPixelStorage::UnpackSkipImages(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, *v);
                TextureUnpackPixelStorage::UnpackSkipImages(0)
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
    fn texture_parameter(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        match self {
            TextureParameter::BaseLevel(v) => {
                gl.tex_parameteri(target.gl_enum(), self.gl_enum(), *v);
            }
            TextureParameter::MaxLevel(v) => {
                gl.tex_parameteri(target.gl_enum(), self.gl_enum(), *v);
            }
            TextureParameter::MaxAnisotropy(v) => {
                gl.tex_parameterf(target.gl_enum(), self.gl_enum(), *v);
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

    fn sampler_parameter(&self, gl: &WebGl2RenderingContext, sampler: &WebGlSampler) {
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
pub enum TexturePixelFormat {
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
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        pbo_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int8Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Int8Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint8Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Uint8Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint8ClampedArray {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int16Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Int16Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint16Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Uint16Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Int32Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Int32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Uint32Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Uint32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    Float32Array {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: Float32Array,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    DataView {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        width: usize,
        height: usize,
        data: DataView,
        src_element_offset: Option<usize>,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlCanvasElement {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        data: HtmlCanvasElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlImageElement {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        data: HtmlImageElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    HtmlVideoElement {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        data: HtmlVideoElement,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    ImageData {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        data: ImageData,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
    ImageBitmap {
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        data: ImageBitmap,
        pixel_storages: Option<Vec<TextureUnpackPixelStorage>>,
    },
}

impl TextureUncompressedData {
    fn pixel_storages(&self) -> Option<&[TextureUnpackPixelStorage]> {
        match self {
            TextureUncompressedData::PixelBufferObject { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::Int8Array { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::Uint8Array { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::Uint8ClampedArray { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::Int16Array { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::Uint16Array { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::Int32Array { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::Uint32Array { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::Float32Array { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::DataView { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::HtmlCanvasElement { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::HtmlImageElement { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::HtmlVideoElement { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
            TextureUncompressedData::ImageData { pixel_storages, .. } => pixel_storages.as_deref(),
            TextureUncompressedData::ImageBitmap { pixel_storages, .. } => {
                pixel_storages.as_deref()
            }
        }
    }

    fn upload(
        &self,
        gl: &WebGl2RenderingContext,
        buffer_bounds: &Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
        target: TextureTarget,
        face: TextureCubeMapFace,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        let is_3d = match target {
            TextureTarget::Texture2D | TextureTarget::TextureCubeMap => false,
            TextureTarget::Texture2DArray | TextureTarget::Texture3D => true,
        };
        let target = match target {
            TextureTarget::Texture2D => TextureTarget::Texture2D.gl_enum(),
            TextureTarget::TextureCubeMap => match face {
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
            },
            TextureTarget::Texture2DArray => TextureTarget::Texture2DArray.gl_enum(),
            TextureTarget::Texture3D => TextureTarget::Texture3D.gl_enum(),
        };

        let result = match self {
            TextureUncompressedData::PixelBufferObject {
                buffer,
                pbo_offset,
                pixel_format,
                pixel_data_type,
                width,
                height,
                ..
            } => {
                gl.bind_buffer(BufferTarget::PixelUnpackBuffer.gl_enum(), Some(&buffer));
                let result = if is_3d {
                    gl.tex_sub_image_3d_with_i32(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        *width as i32,
                        *height as i32,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        pbo_offset.unwrap_or(0) as i32,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        pbo_offset.unwrap_or(0) as i32,
                    )
                };

                gl.bind_buffer(
                    BufferTarget::PixelUnpackBuffer.gl_enum(),
                    buffer_bounds.borrow().get(&BufferTarget::PixelUnpackBuffer),
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
                let (pixel_data_type, pixel_format, data, width, height, src_element_offset) =
                    match self {
                        TextureUncompressedData::Int8Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Uint8Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Uint8ClampedArray {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Int16Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Uint16Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Int32Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Uint32Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::Float32Array {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        TextureUncompressedData::DataView {
                            pixel_data_type,
                            pixel_format,
                            data,
                            width,
                            height,
                            src_element_offset,
                            ..
                        } => (
                            pixel_data_type,
                            pixel_format,
                            Object::as_ref(data),
                            width,
                            height,
                            src_element_offset,
                        ),
                        _ => unreachable!(),
                    };

                if is_3d {
                    gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        *width as i32,
                        *height as i32,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        Some(&data),
                        src_element_offset.unwrap_or(0) as u32,
                    )
                } else {
                    gl
                        .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                            target,
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            *width as i32,
                            *height as i32,
                            pixel_format.gl_enum(),
                            pixel_data_type.gl_enum(),
                            &data,
                            src_element_offset.unwrap_or(0) as u32,
                        )
                }
            }
            TextureUncompressedData::HtmlCanvasElement {
                pixel_data_type,
                pixel_format,
                data,
                ..
            } => {
                let width = data.width() as i32;
                let height = data.height() as i32;

                if is_3d {
                    gl.tex_sub_image_3d_with_html_canvas_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width,
                        height,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width,
                        height,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                }
            }
            TextureUncompressedData::HtmlImageElement {
                pixel_data_type,
                pixel_format,
                data,
                ..
            } => {
                let width = data.natural_width() as i32;
                let height = data.natural_height() as i32;

                if is_3d {
                    gl.tex_sub_image_3d_with_html_image_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width,
                        height,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width,
                        height,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                }
            }
            TextureUncompressedData::HtmlVideoElement {
                pixel_data_type,
                pixel_format,
                data,
                ..
            } => {
                let width = data.video_width() as i32;
                let height = data.video_height() as i32;

                if is_3d {
                    gl.tex_sub_image_3d_with_html_video_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width,
                        height,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width,
                        height,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                }
            }
            TextureUncompressedData::ImageData {
                pixel_data_type,
                pixel_format,
                data,
                ..
            } => {
                let width = data.width() as i32;
                let height = data.height() as i32;

                if is_3d {
                    gl.tex_sub_image_3d_with_image_data(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width,
                        height,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width,
                        height,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                }
            }
            TextureUncompressedData::ImageBitmap {
                pixel_data_type,
                pixel_format,
                data,
                ..
            } => {
                let width = data.width() as i32;
                let height = data.height() as i32;

                if is_3d {
                    gl.tex_sub_image_3d_with_image_bitmap(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width,
                        height,
                        depth as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                } else {
                    gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width,
                        height,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        &data,
                    )
                }
            }
        };

        result.or(Err(Error::UploadTextureDataFailure))
    }
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

impl TextureCompressedData {
    fn upload(
        &self,
        gl: &WebGl2RenderingContext,
        buffer_bounds: &Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
        target: TextureTarget,
        face: TextureCubeMapFace,
        format: TextureCompressedFormat,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) {
        let is_3d = match target {
            TextureTarget::Texture2D | TextureTarget::TextureCubeMap => false,
            TextureTarget::Texture2DArray | TextureTarget::Texture3D => true,
        };
        let target = match target {
            TextureTarget::Texture2D => TextureTarget::Texture2D.gl_enum(),
            TextureTarget::TextureCubeMap => match face {
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
            },
            TextureTarget::Texture2DArray => TextureTarget::Texture2DArray.gl_enum(),
            TextureTarget::Texture3D => TextureTarget::Texture3D.gl_enum(),
        };

        match self {
            TextureCompressedData::PixelBufferObject {
                width,
                height,
                buffer,
                image_size,
                pbo_offset,
            } => {
                gl.bind_buffer(BufferTarget::PixelUnpackBuffer.gl_enum(), Some(&buffer));
                if is_3d {
                    gl.compressed_tex_sub_image_3d_with_i32_and_i32(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        *width as i32,
                        *height as i32,
                        depth as i32,
                        format.gl_enum(),
                        *image_size as i32,
                        pbo_offset.unwrap_or(0) as i32,
                    )
                } else {
                    gl.compressed_tex_sub_image_2d_with_i32_and_i32(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        *image_size as i32,
                        pbo_offset.unwrap_or(0) as i32,
                    )
                };

                gl.bind_buffer(
                    BufferTarget::PixelUnpackBuffer.gl_enum(),
                    buffer_bounds.borrow().get(&BufferTarget::PixelUnpackBuffer),
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
                let (width, height, data, src_element_offset, src_element_length_override) =
                    match self {
                        TextureCompressedData::Int8Array {
                            width,
                            height,
                            data,
                            src_element_offset,
                            src_element_length_override,
                        } => (
                            width,
                            height,
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
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
                            Object::as_ref(data),
                            src_element_offset,
                            src_element_length_override,
                        ),
                        _ => unreachable!(),
                    };

                if is_3d {
                    gl.compressed_tex_sub_image_3d_with_array_buffer_view_and_u32_and_src_length_override(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        *width as i32,
                        *height as i32,
                        depth as i32,
                        format.gl_enum(),
                        &data,
                        src_element_offset.unwrap_or(0) as u32,
                        src_element_length_override.unwrap_or(0) as u32,
                    )
                } else {
                    gl.compressed_tex_sub_image_2d_with_array_buffer_view_and_u32_and_src_length_override(
                        target,
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        &data,
                        src_element_offset.unwrap_or(0) as u32,
                        src_element_length_override.unwrap_or(0) as u32,
                    )
                }
            }
        }
    }
}

pub trait TextureLayout2D {
    fn levels(&self) -> usize;

    fn width(&self) -> usize;

    fn height(&self) -> usize;
}

pub trait TextureLayout3D {
    fn levels(&self) -> usize;

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn depth(&self) -> usize;
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
}

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
}

#[derive(Debug)]
enum QueueItem {
    Uncompressed {
        data: TextureUncompressedData,
        face: TextureCubeMapFace,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    },
    Compressed {
        data: TextureCompressedData,
        face: TextureCubeMapFace,
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

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    pub fn internal_format(&self) -> &InternalFormat {
        &self.internal_format
    }

    pub fn bind(&self, unit: TextureUnit) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .bind(unit)
    }

    pub fn unbind(&self, unit: TextureUnit) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .unbind(unit);
        Ok(())
    }

    pub fn unbind_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .unbind_all();
        Ok(())
    }

    pub fn upload(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .upload()
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

impl<InternalFormat> Texture<Texture2D, InternalFormat>
where
    InternalFormat: TextureInternalFormat,
{
    pub fn byte_length(&self) -> usize {
        (0..self.layout.levels)
            .map(|level| {
                let width = (self.layout.width >> level).max(1);
                let height = (self.layout.height >> level).max(1);
                self.internal_format.byte_length(width, height)
            })
            .sum::<usize>()
    }
}

impl<InternalFormat> Texture<Texture2DArray, InternalFormat>
where
    InternalFormat: TextureInternalFormat,
{
    pub fn byte_length(&self) -> usize {
        (0..self.layout.levels)
            .map(|level| {
                let width = (self.layout.width >> level).max(1);
                let height = (self.layout.height >> level).max(1);
                self.internal_format.byte_length(width, height) * self.layout.length
            })
            .sum::<usize>()
    }
}

impl<InternalFormat> Texture<Texture3D, InternalFormat>
where
    InternalFormat: TextureInternalFormat,
{
    pub fn byte_length(&self) -> usize {
        (0..self.layout.levels)
            .map(|level| {
                let width = (self.layout.width >> level).max(1);
                let height = (self.layout.height >> level).max(1);
                let depth = (self.layout.depth >> level).max(1);
                self.internal_format.byte_length(width, height) * depth
            })
            .sum::<usize>()
    }
}

impl<InternalFormat> Texture<TextureCubeMap, InternalFormat>
where
    InternalFormat: TextureInternalFormat,
{
    pub fn byte_length(&self) -> usize {
        (0..self.layout.levels)
            .map(|level| {
                let width = (self.layout.width >> level).max(1);
                let height = (self.layout.height >> level).max(1);
                self.internal_format.byte_length(width, height) * 6
            })
            .sum::<usize>()
    }
}

impl Texture<Texture2D, TextureUncompressedInternalFormat> {
    pub fn write(&self, data: TextureUncompressedData, level: usize, generate_mipmaps: bool) {
        self.write_with_offset(data, level, 0, 0, generate_mipmaps)
    }

    pub fn write_with_offset(
        &self,
        data: TextureUncompressedData,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) {
        self.queue.borrow_mut().push(QueueItem::Uncompressed {
            data,
            face: TextureCubeMapFace::NegativeX,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
            generate_mipmaps,
        })
    }
}

impl Texture<Texture2D, TextureCompressedFormat> {
    pub fn write(&self, data: TextureCompressedData, level: usize) {
        self.write_with_offset(data, level, 0, 0)
    }

    pub fn write_with_offset(
        &self,
        data: TextureCompressedData,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) {
        self.queue.borrow_mut().push(QueueItem::Compressed {
            data,
            face: TextureCubeMapFace::NegativeX,
            format: self.internal_format,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
        })
    }
}

impl Texture<TextureCubeMap, TextureUncompressedInternalFormat> {
    pub fn write(
        &self,
        data: TextureUncompressedData,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) {
        self.write_with_offset(data, face, level, 0, 0, generate_mipmaps)
    }

    pub fn write_with_offset(
        &self,
        data: TextureUncompressedData,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) {
        self.queue.borrow_mut().push(QueueItem::Uncompressed {
            data,
            face,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
            generate_mipmaps,
        })
    }
}

impl Texture<TextureCubeMap, TextureCompressedFormat> {
    pub fn write(&self, data: TextureCompressedData, face: TextureCubeMapFace, level: usize) {
        self.write_with_offset(data, face, level, 0, 0)
    }

    pub fn write_with_offset(
        &self,
        data: TextureCompressedData,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) {
        self.queue.borrow_mut().push(QueueItem::Compressed {
            data,
            face,
            format: self.internal_format,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
        })
    }
}

impl Texture<Texture2DArray, TextureUncompressedInternalFormat> {
    pub fn write(
        &self,
        data: TextureUncompressedData,
        level: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) {
        self.write_with_offset(data, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_with_offset(
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
            face: TextureCubeMapFace::NegativeX,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
            generate_mipmaps,
        })
    }
}

impl Texture<Texture2DArray, TextureCompressedFormat> {
    pub fn write(&self, data: TextureCompressedData, level: usize, depth: usize) {
        self.write_with_offset(data, level, depth, 0, 0, 0)
    }

    pub fn write_with_offset(
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
            face: TextureCubeMapFace::NegativeX,
            format: self.internal_format,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
        })
    }
}

impl Texture<Texture3D, TextureUncompressedInternalFormat> {
    pub fn write(
        &self,
        data: TextureUncompressedData,
        level: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) {
        self.write_with_offset(data, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_with_offset(
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
            face: TextureCubeMapFace::NegativeX,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
            generate_mipmaps,
        })
    }
}

impl Texture<Texture3D, TextureCompressedFormat> {
    pub fn write(&self, data: TextureCompressedData, level: usize, depth: usize) {
        self.write_with_offset(data, level, depth, 0, 0, 0)
    }

    pub fn write_with_offset(
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
            face: TextureCubeMapFace::NegativeX,
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
    gl_active_unit: HashSet<TextureUnit>,

    reg_id: Uuid,
    reg_active_unit: Rc<RefCell<TextureUnit>>,
    reg_buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    reg_texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    reg_used_memory: Weak<RefCell<usize>>,

    texture_target: TextureTarget,
    texture_memory: usize,
    texture_params: Rc<RefCell<Vec<TextureParameter>>>,
    sampler_params: Rc<RefCell<Vec<SamplerParameter>>>,
    texture_queue: Weak<RefCell<Vec<QueueItem>>>,
}

impl Drop for TextureRegistered {
    fn drop(&mut self) {
        self.unbind_all();
        self.gl.delete_texture(Some(&self.gl_texture));
        self.gl.delete_sampler(Some(&self.gl_sampler));
        self.reg_used_memory
            .upgrade()
            .map(|used_memory| *used_memory.borrow_mut() -= self.texture_memory);
    }
}

impl TextureRegistered {
    fn bind(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if let Some(bound) = self
            .reg_texture_bounds
            .borrow()
            .get(&(unit, self.texture_target))
        {
            if bound == &self.gl_texture {
                self.upload()?;
                return Ok(());
            } else {
                return Err(Error::TextureTargetOccupied(unit, self.texture_target));
            }
        }

        self.upload()?;

        self.gl.active_texture(unit.gl_enum());
        self.gl
            .bind_texture(self.texture_target.gl_enum(), Some(&self.gl_texture));
        self.gl.bind_sampler(unit.gl_enum(), Some(&self.gl_sampler));
        self.gl_active_unit.insert(unit);
        self.reg_texture_bounds
            .borrow_mut()
            .insert_unique_unchecked((unit, self.texture_target), self.gl_texture.clone());
        self.gl
            .active_texture(self.reg_active_unit.borrow().gl_enum());

        Ok(())
    }

    fn unbind(&mut self, unit: TextureUnit) {
        if self.gl_active_unit.remove(&unit) {
            self.gl.active_texture(unit.gl_enum());
            self.gl.bind_texture(self.texture_target.gl_enum(), None);
            self.gl.bind_sampler(unit.gl_enum(), None);
            self.gl
                .active_texture(self.reg_active_unit.borrow().gl_enum());
            self.reg_texture_bounds
                .borrow_mut()
                .remove(&(unit, self.texture_target));
        }
    }

    fn unbind_all(&mut self) {
        for unit in self.gl_active_unit.drain() {
            self.gl.active_texture(unit.gl_enum());
            self.gl.bind_texture(self.texture_target.gl_enum(), None);
            self.gl.bind_sampler(unit.gl_enum(), None);
            self.reg_texture_bounds
                .borrow_mut()
                .remove(&(unit, self.texture_target));
        }
        self.gl
            .active_texture(self.reg_active_unit.borrow().gl_enum());
    }

    fn upload(&self) -> Result<(), Error> {
        let Some(texture_queue) = self.texture_queue.upgrade() else {
            return Err(Error::TextureUnexpectedDropped);
        };

        // update sampler parameters
        for sampler_param in self.sampler_params.borrow().iter() {
            sampler_param.sampler_parameter(&self.gl, &self.gl_sampler);
        }

        self.gl
            .bind_texture(self.texture_target.gl_enum(), Some(&self.gl_texture));

        // update texture parameters
        for texture_param in self.texture_params.borrow().iter() {
            texture_param.texture_parameter(&self.gl, self.texture_target);
        }

        let mut initial_pixel_storages = HashSet::new();
        for item in texture_queue.borrow_mut().drain(..) {
            match item {
                QueueItem::Uncompressed {
                    data,
                    face,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                    generate_mipmaps,
                } => {
                    if let Some(pixel_storages) = data.pixel_storages() {
                        for pixel_storage in pixel_storages {
                            let init = pixel_storage.pixel_store(&self.gl);
                            initial_pixel_storages.insert(init);
                        }
                    }

                    data.upload(
                        &self.gl,
                        &self.reg_buffer_bounds,
                        self.texture_target,
                        face,
                        level,
                        depth,
                        x_offset,
                        y_offset,
                        z_offset,
                    )?;

                    if generate_mipmaps {
                        self.gl.generate_mipmap(self.texture_target.gl_enum());
                    }

                    for pixel_storage in initial_pixel_storages.drain() {
                        pixel_storage.pixel_store(&self.gl);
                    }
                }
                QueueItem::Compressed {
                    data,
                    face,
                    format,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                } => {
                    data.upload(
                        &self.gl,
                        &self.reg_buffer_bounds,
                        self.texture_target,
                        face,
                        format,
                        level,
                        depth,
                        x_offset,
                        y_offset,
                        z_offset,
                    );
                }
            }
        }

        self.gl.bind_texture(
            self.texture_target.gl_enum(),
            self.reg_texture_bounds
                .borrow()
                .get(&(*self.reg_active_unit.borrow(), self.texture_target)),
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct TextureRegistry {
    id: Uuid,
    gl: WebGl2RenderingContext,
    active_unit: Rc<RefCell<TextureUnit>>,
    texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    used_memory: Rc<RefCell<usize>>,
}

impl TextureRegistry {
    pub fn new(
        gl: WebGl2RenderingContext,
        buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    ) -> Self {
        gl.active_texture(TextureUnit::Texture0.gl_enum());
        Self {
            id: Uuid::new_v4(),
            gl,
            active_unit: Rc::new(RefCell::new(TextureUnit::Texture0)),
            texture_bounds: Rc::new(RefCell::new(HashMap::new())),
            buffer_bounds,
            used_memory: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn used_memory(&self) -> usize {
        *self.used_memory.borrow()
    }

    pub fn active_unit(&self) -> Rc<RefCell<TextureUnit>> {
        Rc::clone(&self.active_unit)
    }

    pub fn bounds(&self) -> Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>> {
        Rc::clone(&self.texture_bounds)
    }
}

macro_rules! register_functions {
    ($(($name: ident, $name_compressed: ident, $tex_storage: ident, $target: expr, $layout: ident))+) => {
        impl TextureRegistry {
            $(
                pub fn $name(
                    &self,
                    texture: &Texture<$layout, TextureUncompressedInternalFormat>,
                ) -> Result<(), Error> {
                    if let Some(registered) = &*texture.registered.borrow() {
                        if &registered.reg_id != &self.id {
                            return Err(Error::RegisterTextureToMultipleRepositoryUnsupported);
                        } else {
                            registered.upload()?;
                            return Ok(());
                        }
                    }

                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    let gl_sampler = self
                        .gl
                        .create_sampler()
                        .ok_or(Error::CreateSamplerFailure)?;

                    self.gl
                        .bind_texture($target.gl_enum(), Some(&gl_texture));
                    texture.$tex_storage(&self.gl, $target);
                    let texture_memory = texture.byte_length();
                    *self.used_memory.borrow_mut() += texture_memory;

                    let registered = TextureRegistered {
                        gl: self.gl.clone(),
                        gl_texture,
                        gl_sampler,
                        gl_active_unit: HashSet::new(),

                        reg_id: self.id,
                        reg_active_unit: Rc::clone(&self.active_unit),
                        reg_texture_bounds: Rc::clone(&self.texture_bounds),
                        reg_buffer_bounds: Rc::clone(&self.buffer_bounds),
                        reg_used_memory: Rc::downgrade(&self.used_memory),

                        texture_target: $target,
                        texture_memory,
                        texture_params: Rc::clone(&texture.texture_params),
                        sampler_params: Rc::clone(&texture.sampler_params),
                        texture_queue: Rc::downgrade(&texture.queue),
                    };
                    registered.upload()?; // texture unbind after uploading

                    *texture.registered.borrow_mut() = Some(registered);

                    Ok(())
                }

                pub fn $name_compressed(
                    &self,
                    texture: &Texture<$layout, TextureCompressedFormat>,
                ) -> Result<(), Error> {
                    if let Some(registered) = &*texture.registered.borrow() {
                        if registered.reg_id != self.id {
                            return Err(Error::RegisterTextureToMultipleRepositoryUnsupported);
                        } else {
                            registered.upload()?;
                            return Ok(());
                        }
                    }

                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    let gl_sampler = self
                        .gl
                        .create_sampler()
                        .ok_or(Error::CreateSamplerFailure)?;

                    self.gl
                        .bind_texture($target.gl_enum(), Some(&gl_texture));
                    texture.$tex_storage(&self.gl, $target);
                    let texture_memory = texture.byte_length();
                    *self.used_memory.borrow_mut() += texture_memory;

                    let registered = TextureRegistered {
                        gl: self.gl.clone(),
                        gl_texture,
                        gl_sampler,
                        gl_active_unit: HashSet::new(),

                        reg_id: self.id,
                        reg_active_unit: Rc::clone(&self.active_unit),
                        reg_texture_bounds: Rc::clone(&self.texture_bounds),
                        reg_buffer_bounds: Rc::clone(&self.buffer_bounds),
                        reg_used_memory: Rc::downgrade(&self.used_memory),

                        texture_target: $target,
                        texture_memory,
                        texture_params: Rc::clone(&texture.texture_params),
                        sampler_params: Rc::clone(&texture.sampler_params),
                        texture_queue: Rc::downgrade(&texture.queue),
                    };
                    registered.upload()?; // texture unbind after uploading

                    *texture.registered.borrow_mut() = Some(registered);

                    Ok(())
                }
            )+
        }
    };
}

register_functions! {
    (register_2d, register_2d_compressed, tex_storage_2d, TextureTarget::Texture2D, Texture2D)
    (register_2d_array, register_2d_array_compressed, tex_storage_3d, TextureTarget::Texture2DArray, Texture2DArray)
    (register_3d, register_3d_compressed, tex_storage_3d, TextureTarget::Texture3D, Texture3D)
    (register_cube_map, register_cube_map_compressed, tex_storage_2d, TextureTarget::TextureCubeMap, TextureCubeMap)
}
