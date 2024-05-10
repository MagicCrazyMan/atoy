use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};

use async_trait::async_trait;
use hashbrown::{HashMap, HashSet};
use js_sys::{
    DataView, Float32Array, Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
use log::error;
use proc::GlEnum;
use uuid::Uuid;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::future_to_promise;
use web_sys::{
    DomException, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use super::{
    buffer::BufferTarget,
    error::Error,
    pixel::{PixelDataType, PixelFormat, PixelUnpackStorage},
};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureTarget {
    #[gl_enum(TEXTURE_2D)]
    Texture2D,
    TextureCubeMap,
    #[gl_enum(TEXTURE_2D_ARRAY)]
    Texture2DArray,
    #[gl_enum(TEXTURE_3D)]
    Texture3D,
}

/// Available cube map texture faces mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureCubeMapFace {
    #[gl_enum(TEXTURE_CUBE_MAP_POSITIVE_X)]
    PositiveX,
    #[gl_enum(TEXTURE_CUBE_MAP_NEGATIVE_X)]
    NegativeX,
    #[gl_enum(TEXTURE_CUBE_MAP_POSITIVE_Y)]
    PositiveY,
    #[gl_enum(TEXTURE_CUBE_MAP_NEGATIVE_Y)]
    NegativeY,
    #[gl_enum(TEXTURE_CUBE_MAP_POSITIVE_Z)]
    PositiveZ,
    #[gl_enum(TEXTURE_CUBE_MAP_NEGATIVE_Z)]
    NegativeZ,
}

/// Available texture units mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureUnit {
    #[gl_enum(TEXTURE0)]
    Texture0,
    #[gl_enum(TEXTURE1)]
    Texture1,
    #[gl_enum(TEXTURE2)]
    Texture2,
    #[gl_enum(TEXTURE3)]
    Texture3,
    #[gl_enum(TEXTURE4)]
    Texture4,
    #[gl_enum(TEXTURE5)]
    Texture5,
    #[gl_enum(TEXTURE6)]
    Texture6,
    #[gl_enum(TEXTURE7)]
    Texture7,
    #[gl_enum(TEXTURE8)]
    Texture8,
    #[gl_enum(TEXTURE9)]
    Texture9,
    #[gl_enum(TEXTURE10)]
    Texture10,
    #[gl_enum(TEXTURE11)]
    Texture11,
    #[gl_enum(TEXTURE12)]
    Texture12,
    #[gl_enum(TEXTURE13)]
    Texture13,
    #[gl_enum(TEXTURE14)]
    Texture14,
    #[gl_enum(TEXTURE15)]
    Texture15,
    #[gl_enum(TEXTURE16)]
    Texture16,
    #[gl_enum(TEXTURE17)]
    Texture17,
    #[gl_enum(TEXTURE18)]
    Texture18,
    #[gl_enum(TEXTURE19)]
    Texture19,
    #[gl_enum(TEXTURE20)]
    Texture20,
    #[gl_enum(TEXTURE21)]
    Texture21,
    #[gl_enum(TEXTURE22)]
    Texture22,
    #[gl_enum(TEXTURE23)]
    Texture23,
    #[gl_enum(TEXTURE24)]
    Texture24,
    #[gl_enum(TEXTURE25)]
    Texture25,
    #[gl_enum(TEXTURE26)]
    Texture26,
    #[gl_enum(TEXTURE27)]
    Texture27,
    #[gl_enum(TEXTURE28)]
    Texture28,
    #[gl_enum(TEXTURE29)]
    Texture29,
    #[gl_enum(TEXTURE30)]
    Texture30,
    #[gl_enum(TEXTURE31)]
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

/// Available texture magnification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureMagnificationFilter {
    Linear,
    Nearest,
}

/// Available texture minification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureMinificationFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

/// Available texture wrap methods for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureWrapMethod {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

/// Available texture compare function for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureCompareFunction {
    #[gl_enum(LEQUAL)]
    LessEqual,
    #[gl_enum(GEQUAL)]
    GreaterEqual,
    Less,
    Greater,
    Equal,
    #[gl_enum(NOTEQUAL)]
    NotEqual,
    Always,
    Never,
}

/// Available texture compare modes for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureCompareMode {
    None,
    CompareRefToTexture,
}

/// Available texture parameter kinds mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureParameterKind {
    #[gl_enum(TEXTURE_BASE_LEVEL)]
    BaseLevel,
    #[gl_enum(TEXTURE_MAX_LEVEL)]
    MaxLevel,
    /// Available when extension `EXT_texture_filter_anisotropic` enabled.
    #[gl_enum(TEXTURE_MAX_ANISOTROPY_EXT)]
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
    pub fn kind(&self) -> TextureParameterKind {
        match self {
            TextureParameter::BaseLevel(_) => TextureParameterKind::BaseLevel,
            TextureParameter::MaxLevel(_) => TextureParameterKind::MaxLevel,
            TextureParameter::MaxAnisotropy(_) => TextureParameterKind::MaxAnisotropy,
        }
    }

    fn tex_parameter(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        let pname = self.kind().to_gl_enum();
        match self {
            TextureParameter::BaseLevel(v) => {
                gl.tex_parameteri(target.to_gl_enum(), pname, *v);
            }
            TextureParameter::MaxLevel(v) => {
                gl.tex_parameteri(target.to_gl_enum(), pname, *v);
            }
            TextureParameter::MaxAnisotropy(v) => {
                gl.tex_parameterf(target.to_gl_enum(), pname, *v);
            }
        };
    }
}

/// Available sampling kinds for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum SamplerParameterKind {
    #[gl_enum(TEXTURE_MAG_FILTER)]
    MagnificationFilter,
    #[gl_enum(TEXTURE_MIN_FILTER)]
    MinificationFilter,
    #[gl_enum(TEXTURE_WRAP_S)]
    WrapS,
    #[gl_enum(TEXTURE_WRAP_T)]
    WrapT,
    #[gl_enum(TEXTURE_WRAP_R)]
    WrapR,
    #[gl_enum(TEXTURE_COMPARE_FUNC)]
    CompareFunction,
    #[gl_enum(TEXTURE_COMPARE_MODE)]
    CompareMode,
    #[gl_enum(TEXTURE_MAX_LOD)]
    MaxLod,
    #[gl_enum(TEXTURE_MIN_LOD)]
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
        let pname = self.kind().to_gl_enum();
        match self {
            SamplerParameter::MagnificationFilter(v) => {
                gl.sampler_parameteri(&sampler, pname, v.to_gl_enum() as i32)
            }
            SamplerParameter::MinificationFilter(v) => {
                gl.sampler_parameteri(&sampler, pname, v.to_gl_enum() as i32)
            }
            SamplerParameter::WrapS(v)
            | SamplerParameter::WrapT(v)
            | SamplerParameter::WrapR(v) => {
                gl.sampler_parameteri(&sampler, pname, v.to_gl_enum() as i32)
            }
            SamplerParameter::CompareFunction(v) => {
                gl.sampler_parameteri(&sampler, pname, v.to_gl_enum() as i32)
            }
            SamplerParameter::CompareMode(v) => {
                gl.sampler_parameteri(&sampler, pname, v.to_gl_enum() as i32)
            }
            SamplerParameter::MaxLod(v) | SamplerParameter::MinLod(v) => {
                gl.sampler_parameterf(&sampler, pname, *v)
            }
        }
    }
}

pub trait TextureInternalFormat {
    /// Returns byte length of this internal format in specified size.
    fn byte_length(&self, width: usize, height: usize) -> usize;

    /// Returns WebGL enum associated with this internal format.
    fn to_gl_enum(&self) -> u32;
}

/// Available texture color internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureUncompressedInternalFormat {
    #[gl_enum(RGBA32I)]
    RGBA32I,
    #[gl_enum(RGBA32UI)]
    RGBA32UI,
    #[gl_enum(RGBA16I)]
    RGBA16I,
    #[gl_enum(RGBA16UI)]
    RGBA16UI,
    #[gl_enum(RGBA8)]
    RGBA8,
    #[gl_enum(RGBA8I)]
    RGBA8I,
    #[gl_enum(RGBA8UI)]
    RGBA8UI,
    #[gl_enum(SRGB8_ALPHA8)]
    SRGB8_ALPHA8,
    #[gl_enum(RGB10_A2)]
    RGB10_A2,
    #[gl_enum(RGB10_A2UI)]
    RGB10_A2UI,
    #[gl_enum(RGBA4)]
    RGBA4,
    #[gl_enum(RGB5_A1)]
    RGB5_A1,
    #[gl_enum(RGB8)]
    RGB8,
    #[gl_enum(RGB565)]
    RGB565,
    #[gl_enum(RG32I)]
    RG32I,
    #[gl_enum(RG32UI)]
    RG32UI,
    #[gl_enum(RG16I)]
    RG16I,
    #[gl_enum(RG16UI)]
    RG16UI,
    #[gl_enum(RG8)]
    RG8,
    #[gl_enum(RG8I)]
    RG8I,
    #[gl_enum(RG8UI)]
    RG8UI,
    #[gl_enum(R32I)]
    R32I,
    #[gl_enum(R32UI)]
    R32UI,
    #[gl_enum(R16I)]
    R16I,
    #[gl_enum(R16UI)]
    R16UI,
    #[gl_enum(R8)]
    R8,
    #[gl_enum(R8I)]
    R8I,
    #[gl_enum(R8UI)]
    R8UI,
    #[gl_enum(DEPTH_COMPONENT32F)]
    DEPTH_COMPONENT32F,
    #[gl_enum(DEPTH_COMPONENT24)]
    DEPTH_COMPONENT24,
    #[gl_enum(DEPTH_COMPONENT16)]
    DEPTH_COMPONENT16,
    #[gl_enum(DEPTH32F_STENCIL8)]
    DEPTH32F_STENCIL8,
    #[gl_enum(DEPTH24_STENCIL8)]
    DEPTH24_STENCIL8,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(R16F)]
    R16F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(RG16F)]
    RG16F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(RGBA16F)]
    RGBA16F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(R32F)]
    R32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(RG32F)]
    RG32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(RGBA32F)]
    RGBA32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    #[gl_enum(R11F_G11F_B10F)]
    R11F_G11F_B10F,
    #[gl_enum(RGB8I)]
    RGB8I,
    #[gl_enum(RGB8UI)]
    RGB8UI,
    #[gl_enum(RGB16I)]
    RGB16I,
    #[gl_enum(RGB16UI)]
    RGB16UI,
    #[gl_enum(RGB16F)]
    RGB16F,
    #[gl_enum(RGB32I)]
    RGB32I,
    #[gl_enum(RGB32UI)]
    RGB32UI,
    #[gl_enum(RGB32F)]
    RGB32F,
    #[gl_enum(R8_SNORM)]
    R8_SNORM,
    #[gl_enum(RG8_SNORM)]
    RG8_SNORM,
    #[gl_enum(RGB8_SNORM)]
    RGB8_SNORM,
    #[gl_enum(RGBA8_SNORM)]
    RGBA8_SNORM,
    #[gl_enum(SRGB8)]
    SRGB8,
    #[gl_enum(RGB9_E5)]
    RGB9_E5,
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
    
    fn to_gl_enum(&self) -> u32 {
        self.to_gl_enum()
    }    
}

/// Available texture compressed internal and upload formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum TextureCompressedFormat {
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGB_S3TC_DXT1_EXT)]
    RGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT1_EXT)]
    RGBA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT3_EXT)]
    RGBA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT5_EXT)]
    RGBA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_S3TC_DXT1_EXT)]
    SRGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT)]
    SRGB_ALPHA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT)]
    SRGB_ALPHA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT)]
    SRGB_ALPHA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_R11_EAC)]
    R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_R11_EAC)]
    SIGNED_R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RG11_EAC)]
    RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RG11_EAC)]
    SIGNED_RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RGB8_ETC2)]
    RGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ETC2)]
    RGBA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ETC2)]
    SRGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ETC2_EAC)]
    SRGB8_ALPHA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2)]
    RGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2)]
    SRGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGB_PVRTC_2BPPV1_IMG)]
    RGB_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGBA_PVRTC_2BPPV1_IMG)]
    RGBA_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGB_PVRTC_4BPPV1_IMG)]
    RGB_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGBA_PVRTC_4BPPV1_IMG)]
    RGBA_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_etc1` enabled.
    #[gl_enum(COMPRESSED_RGB_ETC1_WEBGL)]
    RGB_ETC1_WEBGL,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_4X4_KHR)]
    RGBA_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR)]
    SRGB8_ALPHA8_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_5X4_KHR)]
    RGBA_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR)]
    SRGB8_ALPHA8_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_5X5_KHR)]
    RGBA_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR)]
    SRGB8_ALPHA8_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_6X5_KHR)]
    RGBA_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR)]
    SRGB8_ALPHA8_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_6X6_KHR)]
    RGBA_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR)]
    SRGB8_ALPHA8_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X5_KHR)]
    RGBA_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR)]
    SRGB8_ALPHA8_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X6_KHR)]
    RGBA_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR)]
    SRGB8_ALPHA8_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X8_KHR)]
    RGBA_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR)]
    SRGB8_ALPHA8_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X5_KHR)]
    RGBA_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR)]
    SRGB8_ALPHA8_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X6_KHR)]
    RGBA_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR)]
    SRGB8_ALPHA8_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X8_KHR)]
    RGBA_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X8_KHR)]
    SRGB8_ALPHA8_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_12X10_KHR)]
    RGBA_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR)]
    SRGB8_ALPHA8_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_12X12_KHR)]
    RGBA_ASTC_12x12,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR)]
    SRGB8_ALPHA8_ASTC_12x12,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGBA_BPTC_UNORM_EXT)]
    RGBA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT)]
    SRGB_ALPHA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT)]
    RGB_BPTC_SIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT)]
    RGB_BPTC_UNSIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_RED_RGTC1_EXT)]
    RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RED_RGTC1_EXT)]
    SIGNED_RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_RED_GREEN_RGTC2_EXT)]
    RED_GREEN_RGTC2,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT)]
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
    
    fn to_gl_enum(&self) -> u32 {
        self.to_gl_enum()
    }
}

#[derive(Debug, Clone)]
pub enum TextureDataUncompressed {
    PixelBufferObject {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        pbo_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Int8Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Int8Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Uint8Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Uint8Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Uint8ClampedArray {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Int16Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Int16Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Uint16Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Uint16Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Int32Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Int32Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Uint32Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Uint32Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    Float32Array {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: Float32Array,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    DataView {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        width: usize,
        height: usize,
        data: DataView,
        src_element_offset: Option<usize>,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    HtmlCanvasElement {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        data: HtmlCanvasElement,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    HtmlImageElement {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        data: HtmlImageElement,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    HtmlVideoElement {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        data: HtmlVideoElement,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    ImageData {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        data: ImageData,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
    ImageBitmap {
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        data: ImageBitmap,
        pixel_unpack_storages: Option<Vec<PixelUnpackStorage>>,
    },
}

impl TextureDataUncompressed {
    fn pixel_unpack_storages(&self) -> Option<&[PixelUnpackStorage]> {
        match self {
            TextureDataUncompressed::PixelBufferObject {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Int8Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Uint8Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Uint8ClampedArray {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Int16Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Uint16Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Int32Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Uint32Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::Float32Array {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::DataView {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::HtmlCanvasElement {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::HtmlImageElement {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::HtmlVideoElement {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::ImageData {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
            TextureDataUncompressed::ImageBitmap {
                pixel_unpack_storages,
                ..
            } => pixel_unpack_storages.as_deref(),
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
            TextureTarget::Texture2D => TextureTarget::Texture2D.to_gl_enum(),
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
            TextureTarget::Texture2DArray => TextureTarget::Texture2DArray.to_gl_enum(),
            TextureTarget::Texture3D => TextureTarget::Texture3D.to_gl_enum(),
        };

        let result = match self {
            TextureDataUncompressed::PixelBufferObject {
                buffer,
                pbo_offset,
                pixel_format,
                pixel_data_type,
                width,
                height,
                ..
            } => {
                gl.bind_buffer(BufferTarget::PixelUnpackBuffer.to_gl_enum(), Some(&buffer));
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        pbo_offset.unwrap_or(0) as i32,
                    )
                };

                gl.bind_buffer(
                    BufferTarget::PixelUnpackBuffer.to_gl_enum(),
                    buffer_bounds.borrow().get(&BufferTarget::PixelUnpackBuffer),
                );

                result
            }
            TextureDataUncompressed::Int8Array { .. }
            | TextureDataUncompressed::Uint8Array { .. }
            | TextureDataUncompressed::Uint8ClampedArray { .. }
            | TextureDataUncompressed::Int16Array { .. }
            | TextureDataUncompressed::Uint16Array { .. }
            | TextureDataUncompressed::Int32Array { .. }
            | TextureDataUncompressed::Uint32Array { .. }
            | TextureDataUncompressed::Float32Array { .. }
            | TextureDataUncompressed::DataView { .. } => {
                let (pixel_data_type, pixel_format, data, width, height, src_element_offset) =
                    match self {
                        TextureDataUncompressed::Int8Array {
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
                        TextureDataUncompressed::Uint8Array {
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
                        TextureDataUncompressed::Uint8ClampedArray {
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
                        TextureDataUncompressed::Int16Array {
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
                        TextureDataUncompressed::Uint16Array {
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
                        TextureDataUncompressed::Int32Array {
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
                        TextureDataUncompressed::Uint32Array {
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
                        TextureDataUncompressed::Float32Array {
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
                        TextureDataUncompressed::DataView {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                            pixel_format.to_gl_enum(),
                            pixel_data_type.to_gl_enum(),
                            &data,
                            src_element_offset.unwrap_or(0) as u32,
                        )
                }
            }
            TextureDataUncompressed::HtmlCanvasElement {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                }
            }
            TextureDataUncompressed::HtmlImageElement {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                }
            }
            TextureDataUncompressed::HtmlVideoElement {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                }
            }
            TextureDataUncompressed::ImageData {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                }
            }
            TextureDataUncompressed::ImageBitmap {
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
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
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                }
            }
        };

        result.map_err(|err| {
            Error::LoadTextureSourceFailure(Some(err.dyn_into::<DomException>().unwrap().message()))
        })
    }
}

impl TextureSourceUncompressed for TextureDataUncompressed {
    fn load(&mut self) -> Result<TextureDataUncompressed, String> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone)]
pub enum TextureDataCompressed {
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

impl TextureDataCompressed {
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
            TextureTarget::Texture2D => TextureTarget::Texture2D.to_gl_enum(),
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
            TextureTarget::Texture2DArray => TextureTarget::Texture2DArray.to_gl_enum(),
            TextureTarget::Texture3D => TextureTarget::Texture3D.to_gl_enum(),
        };

        match self {
            TextureDataCompressed::PixelBufferObject {
                width,
                height,
                buffer,
                image_size,
                pbo_offset,
            } => {
                gl.bind_buffer(BufferTarget::PixelUnpackBuffer.to_gl_enum(), Some(&buffer));
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
                        format.to_gl_enum(),
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
                        format.to_gl_enum(),
                        *image_size as i32,
                        pbo_offset.unwrap_or(0) as i32,
                    )
                };

                gl.bind_buffer(
                    BufferTarget::PixelUnpackBuffer.to_gl_enum(),
                    buffer_bounds.borrow().get(&BufferTarget::PixelUnpackBuffer),
                );
            }
            TextureDataCompressed::Int8Array { .. }
            | TextureDataCompressed::Uint8Array { .. }
            | TextureDataCompressed::Uint8ClampedArray { .. }
            | TextureDataCompressed::Int16Array { .. }
            | TextureDataCompressed::Uint16Array { .. }
            | TextureDataCompressed::Int32Array { .. }
            | TextureDataCompressed::Uint32Array { .. }
            | TextureDataCompressed::Float32Array { .. }
            | TextureDataCompressed::DataView { .. } => {
                let (width, height, data, src_element_offset, src_element_length_override) =
                    match self {
                        TextureDataCompressed::Int8Array {
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
                        TextureDataCompressed::Uint8Array {
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
                        TextureDataCompressed::Uint8ClampedArray {
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
                        TextureDataCompressed::Int16Array {
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
                        TextureDataCompressed::Uint16Array {
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
                        TextureDataCompressed::Int32Array {
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
                        TextureDataCompressed::Uint32Array {
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
                        TextureDataCompressed::Float32Array {
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
                        TextureDataCompressed::DataView {
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
                        format.to_gl_enum(),
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
                        format.to_gl_enum(),
                        &data,
                        src_element_offset.unwrap_or(0) as u32,
                        src_element_length_override.unwrap_or(0) as u32,
                    )
                }
            }
        }
    }
}

impl TextureSourceCompressed for TextureDataCompressed {
    fn load(&mut self) -> Result<TextureDataCompressed, String> {
        Ok(self.clone())
    }
}

pub trait TextureSourceUncompressed {
    fn load(&mut self) -> Result<TextureDataUncompressed, String>;
}

#[async_trait(?Send)]
pub trait TextureSourceUncompressedAsync {
    async fn load(&mut self) -> Result<TextureDataUncompressed, String>;
}

pub trait TextureSourceCompressed {
    fn load(&mut self) -> Result<TextureDataCompressed, String>;
}

#[async_trait(?Send)]
pub trait TextureSourceCompressedAsync {
    async fn load(&mut self) -> Result<TextureDataCompressed, String>;
}

pub(super) enum TextureSourceUncompressedInner {
    Sync(Box<dyn TextureSourceUncompressed>),
    Async(Box<dyn TextureSourceUncompressedAsync>),
}

impl Debug for TextureSourceUncompressedInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(_) => f.debug_tuple("Sync").finish(),
            Self::Async(_) => f.debug_tuple("Async").finish(),
        }
    }
}

pub(super) enum TextureSourceCompressedInner {
    Sync(Box<dyn TextureSourceCompressed>),
    Async(Box<dyn TextureSourceCompressedAsync>),
}

impl Debug for TextureSourceCompressedInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(_) => f.debug_tuple("Sync").finish(),
            Self::Async(_) => f.debug_tuple("Async").finish(),
        }
    }
}

#[derive(Debug)]
pub(super) enum QueueItem {
    Uncompressed {
        source: TextureSourceUncompressedInner,
        face: TextureCubeMapFace,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    },
    Compressed {
        source: TextureSourceCompressedInner,
        face: TextureCubeMapFace,
        format: TextureCompressedFormat,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    },
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

impl Texture2D {
    pub fn new(levels: usize, width: usize, height: usize) -> Self {
        Self {
            levels,
            width,
            height,
        }
    }
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

impl Texture2DArray {
    pub fn new(levels: usize, width: usize, height: usize, length: usize) -> Self {
        Self {
            levels,
            width,
            height,
            length,
        }
    }
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

impl Texture3D {
    pub fn new(levels: usize, width: usize, height: usize, depth: usize) -> Self {
        Self {
            levels,
            width,
            height,
            depth,
        }
    }
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
pub struct TextureCubeMap {
    levels: usize,
    width: usize,
    height: usize,
}

impl TextureCubeMap {
    pub fn new(levels: usize, width: usize, height: usize) -> Self {
        Self {
            levels,
            width,
            height,
        }
    }
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

#[derive(Debug, Clone)]
pub struct Texture<Layout, InternalFormat> {
    pub(super) id: Uuid,
    pub(super) layout: Layout,
    pub(super) internal_format: InternalFormat,

    pub(super) sampler_params: Rc<RefCell<HashMap<SamplerParameterKind, SamplerParameter>>>,
    pub(super) texture_params: Rc<RefCell<HashMap<TextureParameterKind, TextureParameter>>>,
    pub(super) queue: Rc<RefCell<VecDeque<QueueItem>>>,

    pub(super) registered: Rc<RefCell<Option<TextureRegistered>>>,
}

impl<Layout, InternalFormat> Texture<Layout, InternalFormat> {
    pub fn new(layout: Layout, internal_format: InternalFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            layout,
            internal_format,

            sampler_params: Rc::new(RefCell::new(HashMap::new())),
            texture_params: Rc::new(RefCell::new(HashMap::new())),
            queue: Rc::new(RefCell::new(VecDeque::new())),

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

    pub fn texture_parameter(&self, kind: TextureParameterKind) -> Option<TextureParameter> {
        self.texture_params.borrow().get(&kind).copied()
    }

    pub fn set_texture_parameter(&self, param: TextureParameter) -> Option<TextureParameter> {
        self.texture_params.borrow_mut().insert(param.kind(), param)
    }

    pub fn sampler_parameter(&self, kind: SamplerParameterKind) -> Option<SamplerParameter> {
        self.sampler_params.borrow().get(&kind).copied()
    }

    pub fn set_sampler_parameter(&self, param: SamplerParameter) -> Option<SamplerParameter> {
        self.sampler_params.borrow_mut().insert(param.kind(), param)
    }

    pub fn gl_texture(&self) -> Option<WebGlTexture> {
        self.registered
            .borrow()
            .as_ref()
            .map(|registered| registered.0.gl_texture.clone())
    }

    pub fn gl_sampler(&self) -> Option<WebGlSampler> {
        self.registered
            .borrow()
            .as_ref()
            .map(|registered| registered.0.gl_sampler.clone())
    }

    pub fn flushing(&self) -> bool {
        self.registered
            .borrow()
            .as_ref()
            .map_or(false, |registered| {
                registered.0.texture_async_upload.borrow().is_some()
            })
    }

    pub fn ready(&self) -> bool {
        self.registered.borrow().as_ref().is_some()
            && self.queue.borrow().is_empty()
            && !self.flushing()
    }

    pub fn flush(&self, continue_when_failed: bool) -> Result<bool, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .0
            .flush(continue_when_failed)
    }

    pub async fn flush_async(&self, continue_when_failed: bool) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .0
            .flush_async(continue_when_failed)
            .await
    }

    pub fn bind(&self, unit: TextureUnit) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .0
            .bind(unit)
    }

    pub fn unbind(&self, unit: TextureUnit) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .0
            .unbind(unit);
        Ok(())
    }

    pub fn unbind_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::TextureUnregistered)?
            .0
            .unbind_all();
        Ok(())
    }
}

impl<Layout, InternalFormat> Texture<Layout, InternalFormat>
where
    Layout: TextureLayout2D,
    InternalFormat: TextureInternalFormat,
{
    fn tex_storage_2d(&self, gl: &WebGl2RenderingContext, target: TextureTarget) {
        gl.tex_storage_2d(
            target.to_gl_enum(),
            self.layout.levels() as i32,
            self.internal_format.to_gl_enum(),
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
            target.to_gl_enum(),
            self.layout.levels() as i32,
            self.internal_format.to_gl_enum(),
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
    pub fn write_source<S>(&self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSourceUncompressed + 'static,
    {
        self.write_source_with_offset(source, level, 0, 0, generate_mipmaps)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
            generate_mipmaps,
        })
    }

    pub fn write_async_source<S>(&self, source: S, level: usize, generate_mipmaps: bool)
    where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, 0, 0, generate_mipmaps)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, level: usize)
    where
        S: TextureSourceCompressed + 'static,
    {
        self.write_source_with_offset(source, level, 0, 0)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) where
        S: TextureSourceCompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            format: self.internal_format,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
        })
    }

    pub fn write_async_source<S>(&self, source: S, level: usize)
    where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, 0, 0)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressed + 'static,
    {
        self.write_source_with_offset(source, face, level, 0, 0, generate_mipmaps)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Sync(Box::new(source)),
            face,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
            generate_mipmaps,
        })
    }

    pub fn write_async_source<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, face, level, 0, 0, generate_mipmaps)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, face: TextureCubeMapFace, level: usize)
    where
        S: TextureSourceCompressed + 'static,
    {
        self.write_source_with_offset(source, face, level, 0, 0)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) where
        S: TextureSourceCompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Sync(Box::new(source)),
            face,
            format: self.internal_format,
            level,
            depth: 0,
            x_offset,
            y_offset,
            z_offset: 0,
        })
    }

    pub fn write_async_source<S>(&self, source: S, face: TextureCubeMapFace, level: usize)
    where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, face, level, 0, 0)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        face: TextureCubeMapFace,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, level: usize, depth: usize, generate_mipmaps: bool)
    where
        S: TextureSourceUncompressed + 'static,
    {
        self.write_source_with_offset(source, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
            generate_mipmaps,
        })
    }

    pub fn write_async_source<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, level: usize, depth: usize)
    where
        S: TextureSourceCompressed + 'static,
    {
        self.write_source_with_offset(source, level, depth, 0, 0, 0)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) where
        S: TextureSourceCompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            format: self.internal_format,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
        })
    }

    pub fn write_async_source<S>(&self, source: S, level: usize, depth: usize)
    where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, depth, 0, 0, 0)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, level: usize, depth: usize, generate_mipmaps: bool)
    where
        S: TextureSourceUncompressed + 'static,
    {
        self.write_source_with_offset(source, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
            generate_mipmaps,
        })
    }

    pub fn write_async_source<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, depth, 0, 0, 0, generate_mipmaps)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
        generate_mipmaps: bool,
    ) where
        S: TextureSourceUncompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Uncompressed {
            source: TextureSourceUncompressedInner::Async(Box::new(source)),
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
    pub fn write_source<S>(&self, source: S, level: usize, depth: usize)
    where
        S: TextureSourceCompressed + 'static,
    {
        self.write_source_with_offset(source, level, depth, 0, 0, 0)
    }

    pub fn write_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) where
        S: TextureSourceCompressed + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Sync(Box::new(source)),
            face: TextureCubeMapFace::NegativeX,
            format: self.internal_format,
            level,
            depth,
            x_offset,
            y_offset,
            z_offset,
        })
    }

    pub fn write_async_source<S>(&self, source: S, level: usize, depth: usize)
    where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.write_async_source_with_offset(source, level, depth, 0, 0, 0)
    }

    pub fn write_async_source_with_offset<S>(
        &self,
        source: S,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) where
        S: TextureSourceCompressedAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::Compressed {
            source: TextureSourceCompressedInner::Async(Box::new(source)),
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
pub(super) struct TextureRegistered(pub(super) TextureRegisteredUndrop);

impl Drop for TextureRegistered {
    fn drop(&mut self) {
        self.0.unbind_all();
        self.0.gl.delete_texture(Some(&self.0.gl_texture));
        self.0.gl.delete_sampler(Some(&self.0.gl_sampler));
        self.0
            .reg_used_memory
            .upgrade()
            .map(|used_memory| *used_memory.borrow_mut() -= self.0.texture_memory);
    }
}

#[derive(Debug, Clone)]
pub(super) struct TextureRegisteredUndrop {
    pub(super) gl: WebGl2RenderingContext,
    pub(super) gl_texture: WebGlTexture,
    pub(super) gl_sampler: WebGlSampler,
    pub(super) gl_active_unit: HashSet<TextureUnit>,

    pub(super) reg_id: Uuid,
    pub(super) reg_texture_active_unit: Rc<RefCell<TextureUnit>>,
    pub(super) reg_texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    pub(super) reg_used_memory: Weak<RefCell<usize>>,

    pub(super) reg_buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,

    pub(super) texture_target: TextureTarget,
    pub(super) texture_memory: usize,
    pub(super) texture_params: Rc<RefCell<HashMap<TextureParameterKind, TextureParameter>>>,
    pub(super) sampler_params: Rc<RefCell<HashMap<SamplerParameterKind, SamplerParameter>>>,
    pub(super) texture_queue: Weak<RefCell<VecDeque<QueueItem>>>,
    pub(super) texture_async_upload:
        Rc<RefCell<Option<(Closure<dyn FnMut(JsValue)>, Closure<dyn FnMut(JsValue)>)>>>,
}

impl TextureRegisteredUndrop {
    fn bind(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if let Some(bound) = self
            .reg_texture_bounds
            .borrow()
            .get(&(unit, self.texture_target))
        {
            if bound == &self.gl_texture {
                return Ok(());
            } else {
                return Err(Error::TextureTargetOccupied(unit, self.texture_target));
            }
        }

        self.gl.active_texture(unit.to_gl_enum());
        self.gl
            .bind_texture(self.texture_target.to_gl_enum(), Some(&self.gl_texture));
        self.gl
            .bind_sampler(unit.to_gl_enum(), Some(&self.gl_sampler));
        self.gl_active_unit.insert(unit);
        self.reg_texture_bounds
            .borrow_mut()
            .insert_unique_unchecked((unit, self.texture_target), self.gl_texture.clone());
        self.gl
            .active_texture(self.reg_texture_active_unit.borrow().to_gl_enum());

        Ok(())
    }

    fn unbind(&mut self, unit: TextureUnit) {
        if self.gl_active_unit.remove(&unit) {
            self.gl.active_texture(unit.to_gl_enum());
            self.gl.bind_texture(self.texture_target.to_gl_enum(), None);
            self.gl.bind_sampler(unit.to_gl_enum(), None);
            self.gl
                .active_texture(self.reg_texture_active_unit.borrow().to_gl_enum());
            self.reg_texture_bounds
                .borrow_mut()
                .remove(&(unit, self.texture_target));
        }
    }

    fn unbind_all(&mut self) {
        for unit in self.gl_active_unit.drain() {
            self.gl.active_texture(unit.to_gl_enum());
            self.gl.bind_texture(self.texture_target.to_gl_enum(), None);
            self.gl.bind_sampler(unit.to_gl_enum(), None);
            self.reg_texture_bounds
                .borrow_mut()
                .remove(&(unit, self.texture_target));
        }
        self.gl
            .active_texture(self.reg_texture_active_unit.borrow().to_gl_enum());
    }

    fn flush(&self, continue_when_failed: bool) -> Result<bool, Error> {
        // if there is an ongoing async upload, skips this flush
        if self.texture_async_upload.borrow().is_some() {
            return Ok(false);
        }

        // update sampler parameters
        for (_, sampler_param) in self.sampler_params.borrow().iter() {
            sampler_param.sampler_parameter(&self.gl, &self.gl_sampler);
        }

        self.gl
            .bind_texture(self.texture_target.to_gl_enum(), Some(&self.gl_texture));

        // update texture parameters
        for (_, texture_param) in self.texture_params.borrow().iter() {
            texture_param.tex_parameter(&self.gl, self.texture_target);
        }

        let Some(texture_queue) = self.texture_queue.upgrade() else {
            self.gl.bind_texture(
                self.texture_target.to_gl_enum(),
                self.reg_texture_bounds
                    .borrow()
                    .get(&(*self.reg_texture_active_unit.borrow(), self.texture_target)),
            );
            return Ok(true);
        };

        let mut initial_pixel_unpack_storages = HashSet::new();
        let mut queue = texture_queue.borrow_mut();
        while let Some(item) = queue.pop_front() {
            match item {
                QueueItem::Uncompressed {
                    source,
                    face,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                    generate_mipmaps,
                } => {
                    let data = match source {
                        TextureSourceUncompressedInner::Sync(mut source) => source.load(),
                        TextureSourceUncompressedInner::Async(mut source) => {
                            let promise = future_to_promise(async move {
                                let data = source.load().await;
                                match data {
                                    Ok(data) => {
                                        let data_ptr =
                                            Box::leak(Box::new(data)) as *const _ as usize;
                                        Ok(JsValue::from(data_ptr))
                                    }
                                    Err(msg) => {
                                        let msg_ptr = Box::leak(Box::new(msg)) as *const _ as usize;
                                        Err(JsValue::from(msg_ptr))
                                    }
                                }
                            });

                            let me = self.clone();
                            let resolve = Closure::once(move |value: JsValue| unsafe {
                                let texture_data = Box::from_raw(value.as_f64().unwrap() as usize
                                    as *mut TextureDataUncompressed);
                                let Some(queue) = me.texture_queue.upgrade() else {
                                    return;
                                };

                                // adds buffer data as the first value to the queue and then continues uploading
                                queue.borrow_mut().push_front(QueueItem::Uncompressed {
                                    source: TextureSourceUncompressedInner::Sync(texture_data),
                                    face,
                                    level,
                                    depth,
                                    x_offset,
                                    y_offset,
                                    z_offset,
                                    generate_mipmaps,
                                });
                                me.texture_async_upload.borrow_mut().as_mut().take();
                                let _ = me.flush(continue_when_failed);
                            });

                            let me = self.clone();
                            let reject = Closure::once(move |value: JsValue| unsafe {
                                // if reject, prints error message, sends error message to channel and skips this source
                                let msg =
                                    Box::from_raw(value.as_f64().unwrap() as usize as *mut String);
                                error!("failed to load async buffer source: {}", msg);

                                me.texture_async_upload.borrow_mut().as_mut().take();
                                if continue_when_failed {
                                    // continues uploading
                                    let _ = me.flush(continue_when_failed);
                                }
                            });

                            *self.texture_async_upload.borrow_mut() = Some((resolve, reject));
                            let _ = promise
                                .then(
                                    self.texture_async_upload
                                        .borrow()
                                        .as_ref()
                                        .map(|(resolve, _)| resolve)
                                        .unwrap(),
                                )
                                .catch(
                                    self.texture_async_upload
                                        .borrow()
                                        .as_ref()
                                        .map(|(_, reject)| reject)
                                        .unwrap(),
                                );

                            break;
                        }
                    };
                    let data = match data {
                        Ok(data) => data,
                        Err(msg) => {
                            if continue_when_failed {
                                error!("failed to load texture source: {msg}");
                                continue;
                            } else {
                                return Err(Error::LoadTextureSourceFailure(Some(msg)));
                            }
                        }
                    };

                    if let Some(pixel_unpack_storages) = data.pixel_unpack_storages() {
                        for pixel_unpack_storage in pixel_unpack_storages {
                            let default = pixel_unpack_storage.pixel_store(&self.gl);
                            initial_pixel_unpack_storages.insert(default);
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
                        self.gl.generate_mipmap(self.texture_target.to_gl_enum());
                    }

                    for pixel_unpack_storage in initial_pixel_unpack_storages.drain() {
                        pixel_unpack_storage.pixel_store(&self.gl);
                    }
                }
                QueueItem::Compressed {
                    source,
                    face,
                    format,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                } => {
                    let data = match source {
                        TextureSourceCompressedInner::Sync(mut source) => source.load(),
                        TextureSourceCompressedInner::Async(mut source) => {
                            let promise = future_to_promise(async move {
                                let data = source.load().await;
                                match data {
                                    Ok(data) => {
                                        let data_ptr =
                                            Box::leak(Box::new(data)) as *const _ as usize;
                                        Ok(JsValue::from(data_ptr))
                                    }
                                    Err(msg) => {
                                        let msg_ptr = Box::leak(Box::new(msg)) as *const _ as usize;
                                        Err(JsValue::from(msg_ptr))
                                    }
                                }
                            });

                            let me = self.clone();
                            let resolve = Closure::once(move |value: JsValue| unsafe {
                                let texture_data =
                                    Box::from_raw(value.as_f64().unwrap() as usize
                                        as *mut TextureDataCompressed);
                                let Some(queue) = me.texture_queue.upgrade() else {
                                    return;
                                };

                                // adds buffer data as the first value to the queue and then continues uploading
                                queue.borrow_mut().push_front(QueueItem::Compressed {
                                    source: TextureSourceCompressedInner::Sync(texture_data),
                                    face,
                                    format,
                                    level,
                                    depth,
                                    x_offset,
                                    y_offset,
                                    z_offset,
                                });
                                me.texture_async_upload.borrow_mut().as_mut().take();
                                let _ = me.flush(continue_when_failed);
                            });

                            let me = self.clone();
                            let reject = Closure::once(move |value: JsValue| unsafe {
                                // if reject, prints error message, sends error message to channel and skips this source
                                let msg =
                                    Box::from_raw(value.as_f64().unwrap() as usize as *mut String);
                                error!("failed to load async buffer source: {}", msg);

                                me.texture_async_upload.borrow_mut().as_mut().take();
                                if continue_when_failed {
                                    // continues uploading
                                    let _ = me.flush(continue_when_failed);
                                }
                            });

                            *self.texture_async_upload.borrow_mut() = Some((resolve, reject));
                            let _ = promise
                                .then(
                                    self.texture_async_upload
                                        .borrow()
                                        .as_ref()
                                        .map(|(resolve, _)| resolve)
                                        .unwrap(),
                                )
                                .catch(
                                    self.texture_async_upload
                                        .borrow()
                                        .as_ref()
                                        .map(|(_, reject)| reject)
                                        .unwrap(),
                                );

                            break;
                        }
                    };
                    let data = match data {
                        Ok(data) => data,
                        Err(msg) => {
                            if continue_when_failed {
                                error!("failed to load texture source: {msg}");
                                continue;
                            } else {
                                return Err(Error::LoadTextureSourceFailure(Some(msg)));
                            }
                        }
                    };

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
            self.texture_target.to_gl_enum(),
            self.reg_texture_bounds
                .borrow()
                .get(&(*self.reg_texture_active_unit.borrow(), self.texture_target)),
        );

        Ok(self.texture_async_upload.borrow().is_none())
    }

    async fn flush_async(&self, continue_when_failed: bool) -> Result<(), Error> {
        // update sampler parameters
        for (_, sampler_param) in self.sampler_params.borrow().iter() {
            sampler_param.sampler_parameter(&self.gl, &self.gl_sampler);
        }

        self.gl
            .bind_texture(self.texture_target.to_gl_enum(), Some(&self.gl_texture));

        // update texture parameters
        for (_, texture_param) in self.texture_params.borrow().iter() {
            texture_param.tex_parameter(&self.gl, self.texture_target);
        }

        let Some(texture_queue) = self.texture_queue.upgrade() else {
            self.gl.bind_texture(
                self.texture_target.to_gl_enum(),
                self.reg_texture_bounds
                    .borrow()
                    .get(&(*self.reg_texture_active_unit.borrow(), self.texture_target)),
            );
            return Ok(());
        };

        let mut initial_pixel_unpack_storages = HashSet::new();
        let mut queue = texture_queue.borrow_mut();
        while let Some(item) = queue.pop_front() {
            match item {
                QueueItem::Uncompressed {
                    source,
                    face,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                    generate_mipmaps,
                } => {
                    let data = match source {
                        TextureSourceUncompressedInner::Sync(mut source) => source.load(),
                        TextureSourceUncompressedInner::Async(mut source) => source.load().await,
                    };
                    let data = match data {
                        Ok(data) => data,
                        Err(msg) => {
                            if continue_when_failed {
                                error!("failed to load texture source: {msg}");
                                continue;
                            } else {
                                return Err(Error::LoadTextureSourceFailure(Some(msg)));
                            }
                        }
                    };

                    if let Some(pixel_unpack_storages) = data.pixel_unpack_storages() {
                        for pixel_unpack_storage in pixel_unpack_storages {
                            let default = pixel_unpack_storage.pixel_store(&self.gl);
                            initial_pixel_unpack_storages.insert(default);
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
                        self.gl.generate_mipmap(self.texture_target.to_gl_enum());
                    }

                    for pixel_unpack_storage in initial_pixel_unpack_storages.drain() {
                        pixel_unpack_storage.pixel_store(&self.gl);
                    }
                }
                QueueItem::Compressed {
                    source,
                    face,
                    format,
                    level,
                    depth,
                    x_offset,
                    y_offset,
                    z_offset,
                } => {
                    let data = match source {
                        TextureSourceCompressedInner::Sync(mut source) => source.load(),
                        TextureSourceCompressedInner::Async(mut source) => source.load().await,
                    };
                    let data = match data {
                        Ok(data) => data,
                        Err(msg) => {
                            if continue_when_failed {
                                error!("failed to load texture source: {msg}");
                                continue;
                            } else {
                                return Err(Error::LoadTextureSourceFailure(Some(msg)));
                            }
                        }
                    };

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
            self.texture_target.to_gl_enum(),
            self.reg_texture_bounds
                .borrow()
                .get(&(*self.reg_texture_active_unit.borrow(), self.texture_target)),
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct TextureRegistry {
    pub(super) id: Uuid,
    pub(super) gl: WebGl2RenderingContext,
    pub(super) texture_active_unit: Rc<RefCell<TextureUnit>>,
    pub(super) texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    pub(super) buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    pub(super) used_memory: Rc<RefCell<usize>>,
}

impl TextureRegistry {
    pub fn new(
        gl: WebGl2RenderingContext,
        buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    ) -> Self {
        gl.active_texture(TextureUnit::Texture0.to_gl_enum());
        Self {
            id: Uuid::new_v4(),
            gl,
            texture_active_unit: Rc::new(RefCell::new(TextureUnit::Texture0)),
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
        Rc::clone(&self.texture_active_unit)
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
                        if &registered.0.reg_id != &self.id {
                            return Err(Error::RegisterTextureToMultipleRepositoryUnsupported);
                        } else {
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
                        .bind_texture($target.to_gl_enum(), Some(&gl_texture));

                    texture.$tex_storage(&self.gl, $target);
                    let texture_memory = texture.byte_length();
                    *self.used_memory.borrow_mut() += texture_memory;

                    let registered = TextureRegistered(TextureRegisteredUndrop {
                        gl: self.gl.clone(),
                        gl_texture,
                        gl_sampler,
                        gl_active_unit: HashSet::new(),

                        reg_id: self.id,
                        reg_texture_active_unit: Rc::clone(&self.texture_active_unit),
                        reg_texture_bounds: Rc::clone(&self.texture_bounds),
                        reg_buffer_bounds: Rc::clone(&self.buffer_bounds),
                        reg_used_memory: Rc::downgrade(&self.used_memory),

                        texture_target: $target,
                        texture_memory,
                        texture_params: Rc::clone(&texture.texture_params),
                        sampler_params: Rc::clone(&texture.sampler_params),
                        texture_queue: Rc::downgrade(&texture.queue),
                        texture_async_upload: Rc::new(RefCell::new(None)),
                    });

                    self.gl
                        .bind_texture($target.to_gl_enum(), self.texture_bounds.borrow().get(&(self.texture_active_unit.borrow().clone(), $target)));

                    *texture.registered.borrow_mut() = Some(registered);

                    Ok(())
                }

                pub fn $name_compressed(
                    &self,
                    texture: &Texture<$layout, TextureCompressedFormat>,
                ) -> Result<(), Error> {
                    if let Some(registered) = &*texture.registered.borrow() {
                        if &registered.0.reg_id != &self.id {
                            return Err(Error::RegisterTextureToMultipleRepositoryUnsupported);
                        } else {
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
                        .bind_texture($target.to_gl_enum(), Some(&gl_texture));

                    texture.$tex_storage(&self.gl, $target);
                    let texture_memory = texture.byte_length();
                    *self.used_memory.borrow_mut() += texture_memory;

                    let registered = TextureRegistered(TextureRegisteredUndrop {
                        gl: self.gl.clone(),
                        gl_texture,
                        gl_sampler,
                        gl_active_unit: HashSet::new(),

                        reg_id: self.id,
                        reg_texture_active_unit: Rc::clone(&self.texture_active_unit),
                        reg_texture_bounds: Rc::clone(&self.texture_bounds),
                        reg_buffer_bounds: Rc::clone(&self.buffer_bounds),
                        reg_used_memory: Rc::downgrade(&self.used_memory),

                        texture_target: $target,
                        texture_memory,
                        texture_params: Rc::clone(&texture.texture_params),
                        sampler_params: Rc::clone(&texture.sampler_params),
                        texture_queue: Rc::downgrade(&texture.queue),
                        texture_async_upload: Rc::new(RefCell::new(None)),
                    });

                    self.gl
                        .bind_texture($target.to_gl_enum(), self.texture_bounds.borrow().get(&(self.texture_active_unit.borrow().clone(), $target)));

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
