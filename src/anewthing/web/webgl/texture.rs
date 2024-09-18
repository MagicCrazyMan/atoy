use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use js_sys::{
    DataView, Float32Array, Int16Array, Int32Array, Int8Array, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
use log::warn;
use ordered_float::OrderedFloat;
use proc::GlEnum;
use uuid::Uuid;
use web_sys::{
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlSampler, WebGlTexture,
};

use crate::anewthing::{
    channel::{Channel, Event, Handler},
    texturing::{Texturing, TexturingDropped, TexturingItem},
};

use super::{buffer::WebGlBuffering, capabilities::WebGlCapabilities, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebGlTextureOptions {
    /// texture layout with size.
    pub layout: WebGlTextureLayoutWithSize,
    /// texture internal format.
    pub internal_format: WebGlTextureInternalFormat,
    /// Samplers parameters.
    pub sampler_parameters: Option<Vec<WebGlSamplerParamWithValue>>,
}

/// A wrapped [`Texturing`] with [`WebGlTextureOptions`].
///
/// Do not use different [`WebGlTextureOptions`] for a same [`Texturing`].
/// [`WebGlTextureOptions`] is ignored once a texturing is synced by [`WebGlTextureManager::sync_texture`].
#[derive(Debug, Clone)]
pub struct WebGlTexturing {
    texturing: Texturing,
    options: WebGlTextureOptions,
}

impl WebGlTexturing {
    /// Constructs a new WebGl texturing container.
    pub fn new(texturing: Texturing, options: WebGlTextureOptions) -> Self {
        Self { texturing, options }
    }

    /// Returns WebGl texture options.
    pub fn options(&self) -> &WebGlTextureOptions {
        &self.options
    }
}

impl Deref for WebGlTexturing {
    type Target = Texturing;

    fn deref(&self) -> &Self::Target {
        &self.texturing
    }
}

impl DerefMut for WebGlTexturing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.texturing
    }
}

/// Available texture layouts mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureLayout {
    #[gl_enum(TEXTURE_2D)]
    Texture2D,
    TextureCubeMap,
    #[gl_enum(TEXTURE_2D_ARRAY)]
    Texture2DArray,
    #[gl_enum(TEXTURE_3D)]
    Texture3D,
}

/// Available texture layouts with texture size mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlTextureLayoutWithSize {
    Texture2D {
        /// Texture levels.
        /// Caculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
    },
    TextureCubeMap {
        /// Texture levels.
        /// Caculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
    },
    Texture2DArray {
        /// Texture levels.
        /// Caculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
        /// Texture array length.
        len: usize,
    },
    Texture3D {
        /// Texture levels.
        /// Caculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
        /// Texture depth.
        depth: usize,
    },
}

impl WebGlTextureLayoutWithSize {
    #[inline]
    fn get_or_auto_levels(&self) -> usize {
        match self {
            WebGlTextureLayoutWithSize::Texture2D { width, height, .. } => {
                (*width.max(height) as f64).log2().floor() as usize + 1
            }
            WebGlTextureLayoutWithSize::TextureCubeMap { width, height, .. } => {
                (*width.max(height) as f64).log2().floor() as usize + 1
            }
            WebGlTextureLayoutWithSize::Texture2DArray { width, height, .. } => {
                (*width.max(height) as f64).log2().floor() as usize + 1
            }
            WebGlTextureLayoutWithSize::Texture3D {
                width,
                height,
                depth,
                ..
            } => (*width.max(height).max(depth) as f64).log2().floor() as usize + 1,
        }
    }
}

impl From<WebGlTextureLayoutWithSize> for WebGlTextureLayout {
    #[inline]
    fn from(value: WebGlTextureLayoutWithSize) -> Self {
        match value {
            WebGlTextureLayoutWithSize::Texture2D { .. } => WebGlTextureLayout::Texture2D,
            WebGlTextureLayoutWithSize::TextureCubeMap { .. } => WebGlTextureLayout::TextureCubeMap,
            WebGlTextureLayoutWithSize::Texture2DArray { .. } => WebGlTextureLayout::Texture2DArray,
            WebGlTextureLayoutWithSize::Texture3D { .. } => WebGlTextureLayout::Texture3D,
        }
    }
}

impl WebGlTextureLayoutWithSize {
    /// Returns as [`WebGlTextureLayout`].
    #[inline]
    pub fn as_layout(&self) -> WebGlTextureLayout {
        WebGlTextureLayout::from(*self)
    }

    #[inline]
    pub fn to_gl_enum(&self) -> u32 {
        WebGlTextureLayout::from(*self).to_gl_enum()
    }
}

/// Available cube map texture faces mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureCubeMapFace {
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
pub enum WebGlTextureUnit {
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

/// Available texture sample magnification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSampleMagnificationFilter {
    Linear,
    Nearest,
}

/// Available texture sample minification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSampleMinificationFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

/// Available texture sample wrap methods for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSampleWrapMethod {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

/// Available texture sample compare function for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSampleCompareFunction {
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

/// Available texture sample compare modes for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSampleCompareMode {
    None,
    CompareRefToTexture,
}

/// Available texture color internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureInternalFormat {
    #[gl_enum(RGB)]
    RGB,
    #[gl_enum(RGBA)]
    RGBA,
    #[gl_enum(LUMINANCE)]
    LUMINANCE,
    #[gl_enum(LUMINANCE_ALPHA)]
    LUMINANCE_ALPHA,
    #[gl_enum(ALPHA)]
    ALPHA,
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

impl WebGlTextureInternalFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            Self::RGB => width * height * 3,
            Self::RGBA => width * height * 4,
            Self::LUMINANCE => width * height * 3, // luminance is stored as RGB where all components represent the same value
            Self::LUMINANCE_ALPHA => width * height * 4, // luminance alpha is stored as RGBA where RGB components represent the same value
            Self::ALPHA => width * height * 1,
            Self::RGBA32I => width * height * 16,
            Self::RGBA32UI => width * height * 16,
            Self::RGBA16I => width * height * 4,
            Self::RGBA16UI => width * height * 4,
            Self::RGBA8 => width * height * 4,
            Self::RGBA8I => width * height * 4,
            Self::RGBA8UI => width * height * 4,
            Self::SRGB8_ALPHA8 => width * height * 4,
            Self::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
            Self::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
            Self::RGBA4 => width * height * 2,
            Self::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
            Self::RGB8 => width * height * 3,
            Self::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
            Self::RG32I => width * height * 4,
            Self::RG32UI => width * height * 4,
            Self::RG16I => width * height * 4,
            Self::RG16UI => width * height * 4,
            Self::RG8 => width * height * 2,
            Self::RG8I => width * height * 2,
            Self::RG8UI => width * height * 2,
            Self::R32I => width * height * 4,
            Self::R32UI => width * height * 4,
            Self::R16I => width * height * 2,
            Self::R16UI => width * height * 2,
            Self::R8 => width * height * 1,
            Self::R8I => width * height * 1,
            Self::R8UI => width * height * 1,
            Self::RGBA32F => width * height * 16,
            Self::RGBA16F => width * height * 4,
            Self::RGBA8_SNORM => width * height * 4,
            Self::RGB32F => width * height * 12,
            Self::RGB32I => width * height * 12,
            Self::RGB32UI => width * height * 12,
            Self::RGB16F => width * height * 6,
            Self::RGB16I => width * height * 6,
            Self::RGB16UI => width * height * 6,
            Self::RGB8_SNORM => width * height * 3,
            Self::RGB8I => width * height * 3,
            Self::RGB8UI => width * height * 3,
            Self::SRGB8 => width * height * 3,
            Self::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
            Self::RGB9_E5 => width * height * 4,        // 9 + 9 + 9 + 5 in bits
            Self::RG32F => width * height * 4,
            Self::RG16F => width * height * 4,
            Self::RG8_SNORM => width * height * 2,
            Self::R32F => width * height * 4,
            Self::R16F => width * height * 2,
            Self::R8_SNORM => width * height * 1,
            Self::DEPTH_COMPONENT32F => width * height * 4,
            Self::DEPTH_COMPONENT24 => width * height * 3,
            Self::DEPTH_COMPONENT16 => width * height * 2,
            Self::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
            Self::DEPTH24_STENCIL8 => width * height * 4,
            // for S3TC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc/ for more details
            Self::RGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::RGBA_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::RGBA_S3TC_DXT3 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGBA_S3TC_DXT5 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            // for S3TC RGBA, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc_srgb/ for more details
            Self::SRGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SRGB_ALPHA_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SRGB_ALPHA_S3TC_DXT3 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SRGB_ALPHA_S3TC_DXT5 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            // for ETC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc/ for more details
            Self::R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SIGNED_R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SIGNED_RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SRGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::RGBA8_ETC2_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SRGB8_ALPHA8_ETC2_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGB8_PUNCHTHROUGH_ALPHA1_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            // for PVRTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_pvrtc/ for more details
            Self::RGB_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            Self::RGBA_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            Self::RGB_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            Self::RGBA_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            // for ETC1, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc1/ for more details
            Self::RGB_ETC1_WEBGL => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            // for ASTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_astc/ for more details
            Self::RGBA_ASTC_4x4 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SRGB8_ALPHA8_ASTC_4x4 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGBA_ASTC_5x4 => ((width + 4) / 5) * ((height + 3) / 4) * 16,
            Self::SRGB8_ALPHA8_ASTC_5x4 => ((width + 4) / 5) * ((height + 3) / 4) * 16,
            Self::RGBA_ASTC_5x5 => ((width + 4) / 5) * ((height + 4) / 5) * 16,
            Self::SRGB8_ALPHA8_ASTC_5x5 => ((width + 4) / 5) * ((height + 4) / 5) * 16,
            Self::RGBA_ASTC_6x5 => ((width + 5) / 6) * ((height + 4) / 5) * 16,
            Self::SRGB8_ALPHA8_ASTC_6x5 => ((width + 5) / 6) * ((height + 4) / 5) * 16,
            Self::RGBA_ASTC_6x6 => ((width + 5) / 6) * ((height + 5) / 6) * 16,
            Self::SRGB8_ALPHA8_ASTC_6x6 => ((width + 5) / 6) * ((height + 5) / 6) * 16,
            Self::RGBA_ASTC_8x5 => ((width + 7) / 8) * ((height + 4) / 5) * 16,
            Self::SRGB8_ALPHA8_ASTC_8x5 => ((width + 7) / 8) * ((height + 4) / 5) * 16,
            Self::RGBA_ASTC_8x6 => ((width + 7) / 8) * ((height + 5) / 6) * 16,
            Self::SRGB8_ALPHA8_ASTC_8x6 => ((width + 7) / 8) * ((height + 5) / 6) * 16,
            Self::RGBA_ASTC_8x8 => ((width + 7) / 8) * ((height + 7) / 8) * 16,
            Self::SRGB8_ALPHA8_ASTC_8x8 => ((width + 7) / 8) * ((height + 7) / 8) * 16,
            Self::RGBA_ASTC_10x5 => ((width + 9) / 10) * ((height + 4) / 5) * 16,
            Self::SRGB8_ALPHA8_ASTC_10x5 => ((width + 9) / 10) * ((height + 4) / 5) * 16,
            Self::RGBA_ASTC_10x6 => ((width + 9) / 10) * ((height + 5) / 6) * 16,
            Self::SRGB8_ALPHA8_ASTC_10x6 => ((width + 9) / 10) * ((height + 5) / 6) * 16,
            Self::RGBA_ASTC_10x10 => ((width + 9) / 10) * ((height + 9) / 10) * 16,
            Self::SRGB8_ALPHA8_ASTC_10x10 => ((width + 9) / 10) * ((height + 9) / 10) * 16,
            Self::RGBA_ASTC_12x10 => ((width + 11) / 12) * ((height + 9) / 10) * 16,
            Self::SRGB8_ALPHA8_ASTC_12x10 => ((width + 11) / 12) * ((height + 9) / 10) * 16,
            Self::RGBA_ASTC_12x12 => ((width + 11) / 12) * ((height + 11) / 12) * 16,
            Self::SRGB8_ALPHA8_ASTC_12x12 => ((width + 11) / 12) * ((height + 11) / 12) * 16,
            // for BPTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_bptc/ for more details
            Self::RGBA_BPTC_UNORM => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SRGB_ALPHA_BPTC_UNORM => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGB_BPTC_SIGNED_FLOAT => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::RGB_BPTC_UNSIGNED_FLOAT => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            // for RGTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_rgtc/ for more details
            Self::RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::SIGNED_RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
            Self::RED_GREEN_RGTC2 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
            Self::SIGNED_RED_GREEN_RGTC2 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
        }
    }

    /// Checks whether the pixel data type is compatible with the internal format.
    ///
    /// References [https://registry.khronos.org/webgl/specs/latest/2.0/#3.7.6] for more details.
    fn check_pixel_data_type(&self, data_type: WebGlImagePixelDataType) -> bool {
        match self {
            WebGlTextureInternalFormat::RGB => match data_type {
                WebGlImagePixelDataType::UnsignedByte
                | WebGlImagePixelDataType::UnsignedShort_5_6_5 => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGBA => match data_type {
                WebGlImagePixelDataType::UnsignedByte
                | WebGlImagePixelDataType::UnsignedShort_5_5_5_1
                | WebGlImagePixelDataType::UnsignedShort_4_4_4_4 => true,
                _ => false,
            },
            WebGlTextureInternalFormat::LUMINANCE
            | WebGlTextureInternalFormat::LUMINANCE_ALPHA
            | WebGlTextureInternalFormat::ALPHA
            | WebGlTextureInternalFormat::RGBA8
            | WebGlTextureInternalFormat::RGBA8UI
            | WebGlTextureInternalFormat::SRGB8_ALPHA8
            | WebGlTextureInternalFormat::RGB8
            | WebGlTextureInternalFormat::RG8
            | WebGlTextureInternalFormat::RG8UI
            | WebGlTextureInternalFormat::R8
            | WebGlTextureInternalFormat::R8UI
            | WebGlTextureInternalFormat::RGB8UI
            | WebGlTextureInternalFormat::SRGB8 => match data_type {
                WebGlImagePixelDataType::UnsignedByte => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGB10_A2 => match data_type {
                WebGlImagePixelDataType::UnsignedInt_2_10_10_10Rev => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGBA4 => match data_type {
                WebGlImagePixelDataType::UnsignedByte
                | WebGlImagePixelDataType::UnsignedShort_4_4_4_4 => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGB5_A1 => match data_type {
                WebGlImagePixelDataType::UnsignedByte
                | WebGlImagePixelDataType::UnsignedShort_5_5_5_1 => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGB565 => match data_type {
                WebGlImagePixelDataType::UnsignedByte
                | WebGlImagePixelDataType::UnsignedShort_5_6_5 => true,
                _ => false,
            },
            WebGlTextureInternalFormat::R16F
            | WebGlTextureInternalFormat::RG16F
            | WebGlTextureInternalFormat::RGBA16F
            | WebGlTextureInternalFormat::RGB16F
            | WebGlTextureInternalFormat::RGB9_E5 => match data_type {
                WebGlImagePixelDataType::HalfFloat | WebGlImagePixelDataType::Float => true,
                _ => false,
            },
            WebGlTextureInternalFormat::R32F
            | WebGlTextureInternalFormat::RG32F
            | WebGlTextureInternalFormat::RGBA32F
            | WebGlTextureInternalFormat::RGB32F => match data_type {
                WebGlImagePixelDataType::Float => true,
                _ => false,
            },
            WebGlTextureInternalFormat::R11F_G11F_B10F => match data_type {
                WebGlImagePixelDataType::HalfFloat
                | WebGlImagePixelDataType::Float
                | WebGlImagePixelDataType::UnsignedInt_10F_11F_11F_Rev => true,
                _ => false,
            },
            _ => false,
        }
    }

    /// Checks whether the pixel format is compatible with the internal format.
    ///
    /// References [https://registry.khronos.org/webgl/specs/latest/2.0/#3.7.6] for more details.
    fn check_pixel_format(&self, format: WebGlImagePixelFormat) -> bool {
        match self {
            WebGlTextureInternalFormat::RGB
            | WebGlTextureInternalFormat::RGB16F
            | WebGlTextureInternalFormat::RGB32F
            | WebGlTextureInternalFormat::SRGB8
            | WebGlTextureInternalFormat::RGB9_E5
            | WebGlTextureInternalFormat::RGB8
            | WebGlTextureInternalFormat::RGB565
            | WebGlTextureInternalFormat::R11F_G11F_B10F => match format {
                WebGlImagePixelFormat::Rgb => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGBA
            | WebGlTextureInternalFormat::RGBA8
            | WebGlTextureInternalFormat::RGBA8UI
            | WebGlTextureInternalFormat::SRGB8_ALPHA8
            | WebGlTextureInternalFormat::RGB10_A2
            | WebGlTextureInternalFormat::RGBA4
            | WebGlTextureInternalFormat::RGB5_A1
            | WebGlTextureInternalFormat::RGBA16F
            | WebGlTextureInternalFormat::RGBA32F => match format {
                WebGlImagePixelFormat::Rgba => true,
                _ => false,
            },
            WebGlTextureInternalFormat::LUMINANCE => match format {
                WebGlImagePixelFormat::Luminance => true,
                _ => false,
            },
            WebGlTextureInternalFormat::LUMINANCE_ALPHA => match format {
                WebGlImagePixelFormat::LuminanceAlpha => true,
                _ => false,
            },
            WebGlTextureInternalFormat::ALPHA => match format {
                WebGlImagePixelFormat::Alpha => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RG8
            | WebGlTextureInternalFormat::RG16F
            | WebGlTextureInternalFormat::RG32F => match format {
                WebGlImagePixelFormat::Rg => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RG8UI => match format {
                WebGlImagePixelFormat::RgInteger => true,
                _ => false,
            },
            WebGlTextureInternalFormat::R8
            | WebGlTextureInternalFormat::R16F
            | WebGlTextureInternalFormat::R32F => match format {
                WebGlImagePixelFormat::Red => true,
                _ => false,
            },
            WebGlTextureInternalFormat::R8UI => match format {
                WebGlImagePixelFormat::RedInteger => true,
                _ => false,
            },
            WebGlTextureInternalFormat::RGB8UI => match format {
                WebGlImagePixelFormat::RgbInteger => true,
                _ => false,
            },
            _ => false,
        }
    }
}

/// Available image pixel formats mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlImagePixelFormat {
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

/// Available image pixel data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlImagePixelDataType {
    Float,
    HalfFloat,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    #[gl_enum(UNSIGNED_SHORT_5_6_5)]
    UnsignedShort_5_6_5,
    #[gl_enum(UNSIGNED_SHORT_4_4_4_4)]
    UnsignedShort_4_4_4_4,
    #[gl_enum(UNSIGNED_SHORT_5_5_5_1)]
    UnsignedShort_5_5_5_1,
    #[gl_enum(UNSIGNED_INT_2_10_10_10_REV)]
    UnsignedInt_2_10_10_10Rev,
    #[gl_enum(UNSIGNED_INT_10F_11F_11F_REV)]
    UnsignedInt_10F_11F_11F_Rev,
    #[gl_enum(UNSIGNED_INT_5_9_9_9_REV)]
    UnsignedInt_5_9_9_9Rev,
    #[gl_enum(UNSIGNED_INT_24_8)]
    UnsignedInt_24_8,
    #[gl_enum(FLOAT_32_UNSIGNED_INT_24_8_REV)]
    Float_32_UnsignedInt_24_8_Rev,
}

/// Available texture unpack color space conversions for [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelUnpackColorSpaceConversion {
    NONE,
    BROWSER_DEFAULT_WEBGL,
}

/// Available texture pixel store for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelStore {
    PackAlignment,
    PackRowLength,
    PackSkipPixels,
    PackSkipRows,
    UnpackAlignment,
    #[gl_enum(UNPACK_FLIP_Y_WEBGL)]
    UnpackFlipY,
    #[gl_enum(UNPACK_PREMULTIPLY_ALPHA_WEBGL)]
    UnpackPremultiplyAlpha,
    #[gl_enum(UNPACK_COLORSPACE_CONVERSION_WEBGL)]
    UnpackColorspaceConversion,
    UnpackRowLength,
    UnpackImageHeight,
    UnpackSkipPixels,
    UnpackSkipRows,
    UnpackSkipImages,
}

/// Available texture pixel stores with value for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelStoreWithValue {
    PackAlignment(i32),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
    UnpackAlignment(i32),
    UnpackFlipY(bool),
    UnpackPremultiplyAlpha(bool),
    UnpackColorspaceConversion(WebGlPixelUnpackColorSpaceConversion),
    UnpackRowLength(i32),
    UnpackImageHeight(i32),
    UnpackSkipPixels(i32),
    UnpackSkipRows(i32),
    UnpackSkipImages(i32),
}

impl From<WebGlPixelStoreWithValue> for WebGlPixelStore {
    #[inline]
    fn from(value: WebGlPixelStoreWithValue) -> Self {
        match value {
            WebGlPixelStoreWithValue::PackAlignment(_) => WebGlPixelStore::PackAlignment,
            WebGlPixelStoreWithValue::PackRowLength(_) => WebGlPixelStore::PackRowLength,
            WebGlPixelStoreWithValue::PackSkipPixels(_) => WebGlPixelStore::PackSkipPixels,
            WebGlPixelStoreWithValue::PackSkipRows(_) => WebGlPixelStore::PackSkipRows,
            WebGlPixelStoreWithValue::UnpackAlignment(_) => WebGlPixelStore::UnpackAlignment,
            WebGlPixelStoreWithValue::UnpackFlipY(_) => WebGlPixelStore::UnpackFlipY,
            WebGlPixelStoreWithValue::UnpackPremultiplyAlpha(_) => {
                WebGlPixelStore::UnpackPremultiplyAlpha
            }
            WebGlPixelStoreWithValue::UnpackColorspaceConversion(_) => {
                WebGlPixelStore::UnpackColorspaceConversion
            }
            WebGlPixelStoreWithValue::UnpackRowLength(_) => WebGlPixelStore::UnpackRowLength,
            WebGlPixelStoreWithValue::UnpackImageHeight(_) => WebGlPixelStore::UnpackImageHeight,
            WebGlPixelStoreWithValue::UnpackSkipPixels(_) => WebGlPixelStore::UnpackSkipPixels,
            WebGlPixelStoreWithValue::UnpackSkipRows(_) => WebGlPixelStore::UnpackSkipRows,
            WebGlPixelStoreWithValue::UnpackSkipImages(_) => WebGlPixelStore::UnpackSkipImages,
        }
    }
}

impl WebGlPixelStoreWithValue {
    /// Returns as [`WebGlTexturePixelStore`].
    #[inline]
    pub fn as_pixel_store(&self) -> WebGlPixelStore {
        WebGlPixelStore::from(*self)
    }

    #[inline]
    pub fn to_gl_enum(&self) -> u32 {
        WebGlPixelStore::from(*self).to_gl_enum()
    }
}

/// Available texture sample parameters for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSamplerParam {
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

/// Available texture sample parameter with values for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebGlSamplerParamWithValue {
    MagnificationFilter(WebGlSampleMagnificationFilter),
    MinificationFilter(WebGlSampleMinificationFilter),
    WrapS(WebGlSampleWrapMethod),
    WrapT(WebGlSampleWrapMethod),
    WrapR(WebGlSampleWrapMethod),
    CompareFunction(WebGlSampleCompareFunction),
    CompareMode(WebGlSampleCompareMode),
    MaxLod(f32),
    MinLod(f32),
}

impl Eq for WebGlSamplerParamWithValue {}

impl Hash for WebGlSamplerParamWithValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl From<WebGlSamplerParamWithValue> for WebGlSamplerParam {
    #[inline]
    fn from(value: WebGlSamplerParamWithValue) -> Self {
        match value {
            WebGlSamplerParamWithValue::MagnificationFilter(_) => {
                WebGlSamplerParam::MagnificationFilter
            }
            WebGlSamplerParamWithValue::MinificationFilter(_) => {
                WebGlSamplerParam::MinificationFilter
            }
            WebGlSamplerParamWithValue::WrapS(_) => WebGlSamplerParam::WrapS,
            WebGlSamplerParamWithValue::WrapT(_) => WebGlSamplerParam::WrapT,
            WebGlSamplerParamWithValue::WrapR(_) => WebGlSamplerParam::WrapR,
            WebGlSamplerParamWithValue::CompareFunction(_) => WebGlSamplerParam::CompareFunction,
            WebGlSamplerParamWithValue::CompareMode(_) => WebGlSamplerParam::CompareMode,
            WebGlSamplerParamWithValue::MaxLod(_) => WebGlSamplerParam::MaxLod,
            WebGlSamplerParamWithValue::MinLod(_) => WebGlSamplerParam::MinLod,
        }
    }
}

impl WebGlSamplerParamWithValue {
    /// Returns as [`WebGlSamplerParam`].
    #[inline]
    pub fn as_sample_parameter(&self) -> WebGlSamplerParam {
        WebGlSamplerParam::from(*self)
    }

    #[inline]
    fn priority(&self) -> u8 {
        match self {
            WebGlSamplerParamWithValue::MagnificationFilter(_) => 0,
            WebGlSamplerParamWithValue::MinificationFilter(_) => 1,
            WebGlSamplerParamWithValue::WrapS(_) => 2,
            WebGlSamplerParamWithValue::WrapT(_) => 3,
            WebGlSamplerParamWithValue::WrapR(_) => 4,
            WebGlSamplerParamWithValue::CompareFunction(_) => 5,
            WebGlSamplerParamWithValue::CompareMode(_) => 6,
            WebGlSamplerParamWithValue::MaxLod(_) => 7,
            WebGlSamplerParamWithValue::MinLod(_) => 8,
        }
    }
}

/// Available uncompressed texture data types.
pub enum WebGlUncompressedTextureData<'a> {
    Binary {
        width: usize,
        height: usize,
        data: &'a [u8],
        element_offset: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffering: &'a WebGlBuffering,
        pbo_offset: Option<usize>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        element_offset: Option<usize>,
    },
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        element_offset: Option<usize>,
    },
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        element_offset: Option<usize>,
    },
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        element_offset: Option<usize>,
    },
    Uint16Array {
        width: usize,
        height: usize,
        data: Uint16Array,
        element_offset: Option<usize>,
    },
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        element_offset: Option<usize>,
    },
    Uint32Array {
        width: usize,
        height: usize,
        data: Uint32Array,
        element_offset: Option<usize>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        element_offset: Option<usize>,
    },
    DataView {
        width: usize,
        height: usize,
        data: DataView,
        element_offset: Option<usize>,
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

/// Available compressed texture data types.
pub enum WebGlCompressedTextureData<'a> {
    Binary {
        width: usize,
        height: usize,
        data: &'a [u8],
        element_offset: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffering: &'a WebGlBuffering,
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
pub enum WebGlTextureData<'a> {
    Uncompressed {
        pixel_format: WebGlImagePixelFormat,
        pixel_data_type: WebGlImagePixelDataType,
        pixel_storages: &'a [WebGlPixelStoreWithValue],
        generate_mipmaps: bool,
        flip_y: bool,
        data: WebGlUncompressedTextureData<'a>,
    },
    Compressed {
        data: WebGlCompressedTextureData<'a>,
    },
}

impl<'a> WebGlTextureData<'a> {
    fn width(&self) -> usize {
        match self {
            WebGlTextureData::Uncompressed { data, .. } => match data {
                WebGlUncompressedTextureData::Binary { width, .. }
                | WebGlUncompressedTextureData::PixelBufferObject { width, .. }
                | WebGlUncompressedTextureData::Int8Array { width, .. }
                | WebGlUncompressedTextureData::Uint8Array { width, .. }
                | WebGlUncompressedTextureData::Uint8ClampedArray { width, .. }
                | WebGlUncompressedTextureData::Int16Array { width, .. }
                | WebGlUncompressedTextureData::Uint16Array { width, .. }
                | WebGlUncompressedTextureData::Int32Array { width, .. }
                | WebGlUncompressedTextureData::Uint32Array { width, .. }
                | WebGlUncompressedTextureData::Float32Array { width, .. }
                | WebGlUncompressedTextureData::DataView { width, .. } => *width,
                WebGlUncompressedTextureData::HtmlCanvasElement { data, .. } => {
                    data.width() as usize
                }
                WebGlUncompressedTextureData::HtmlImageElement { data, .. } => {
                    data.natural_width() as usize
                }
                WebGlUncompressedTextureData::HtmlVideoElement { data, .. } => {
                    data.video_width() as usize
                }
                WebGlUncompressedTextureData::ImageData { data, .. } => data.width() as usize,
                WebGlUncompressedTextureData::ImageBitmap { data, .. } => data.width() as usize,
            },
            WebGlTextureData::Compressed { data, .. } => match data {
                WebGlCompressedTextureData::Binary { width, .. }
                | WebGlCompressedTextureData::PixelBufferObject { width, .. }
                | WebGlCompressedTextureData::Int8Array { width, .. }
                | WebGlCompressedTextureData::Uint8Array { width, .. }
                | WebGlCompressedTextureData::Uint8ClampedArray { width, .. }
                | WebGlCompressedTextureData::Int16Array { width, .. }
                | WebGlCompressedTextureData::Uint16Array { width, .. }
                | WebGlCompressedTextureData::Int32Array { width, .. }
                | WebGlCompressedTextureData::Uint32Array { width, .. }
                | WebGlCompressedTextureData::Float32Array { width, .. }
                | WebGlCompressedTextureData::DataView { width, .. } => *width,
            },
        }
    }

    fn height(&self) -> usize {
        match self {
            WebGlTextureData::Uncompressed { data, .. } => match data {
                WebGlUncompressedTextureData::Binary { height, .. }
                | WebGlUncompressedTextureData::PixelBufferObject { height, .. }
                | WebGlUncompressedTextureData::Int8Array { height, .. }
                | WebGlUncompressedTextureData::Uint8Array { height, .. }
                | WebGlUncompressedTextureData::Uint8ClampedArray { height, .. }
                | WebGlUncompressedTextureData::Int16Array { height, .. }
                | WebGlUncompressedTextureData::Uint16Array { height, .. }
                | WebGlUncompressedTextureData::Int32Array { height, .. }
                | WebGlUncompressedTextureData::Uint32Array { height, .. }
                | WebGlUncompressedTextureData::Float32Array { height, .. }
                | WebGlUncompressedTextureData::DataView { height, .. } => *height,
                WebGlUncompressedTextureData::HtmlCanvasElement { data, .. } => {
                    data.height() as usize
                }
                WebGlUncompressedTextureData::HtmlImageElement { data, .. } => {
                    data.natural_height() as usize
                }
                WebGlUncompressedTextureData::HtmlVideoElement { data, .. } => {
                    data.video_height() as usize
                }
                WebGlUncompressedTextureData::ImageData { data, .. } => data.height() as usize,
                WebGlUncompressedTextureData::ImageBitmap { data, .. } => data.height() as usize,
            },
            WebGlTextureData::Compressed { data, .. } => match data {
                WebGlCompressedTextureData::Binary { height, .. }
                | WebGlCompressedTextureData::PixelBufferObject { height, .. }
                | WebGlCompressedTextureData::Int8Array { height, .. }
                | WebGlCompressedTextureData::Uint8Array { height, .. }
                | WebGlCompressedTextureData::Uint8ClampedArray { height, .. }
                | WebGlCompressedTextureData::Int16Array { height, .. }
                | WebGlCompressedTextureData::Uint16Array { height, .. }
                | WebGlCompressedTextureData::Int32Array { height, .. }
                | WebGlCompressedTextureData::Uint32Array { height, .. }
                | WebGlCompressedTextureData::Float32Array { height, .. }
                | WebGlCompressedTextureData::DataView { height, .. } => *height,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SamplerParameters {
    magnification_filter: WebGlSampleMagnificationFilter,
    minification_filter: WebGlSampleMinificationFilter,
    wrap_s: WebGlSampleWrapMethod,
    wrap_t: WebGlSampleWrapMethod,
    wrap_r: WebGlSampleWrapMethod,
    compare_function: WebGlSampleCompareFunction,
    compare_mode: WebGlSampleCompareMode,
    max_lod: OrderedFloat<f32>,
    min_lod: OrderedFloat<f32>,
}

impl Default for SamplerParameters {
    fn default() -> Self {
        Self {
            magnification_filter: WebGlSampleMagnificationFilter::Linear,
            minification_filter: WebGlSampleMinificationFilter::NearestMipmapLinear,
            wrap_s: WebGlSampleWrapMethod::Repeat,
            wrap_t: WebGlSampleWrapMethod::Repeat,
            wrap_r: WebGlSampleWrapMethod::Repeat,
            compare_function: WebGlSampleCompareFunction::LessEqual,
            compare_mode: WebGlSampleCompareMode::None,
            max_lod: OrderedFloat(1000.0),
            min_lod: OrderedFloat(-1000.0),
        }
    }
}

struct WebGlSamplerManager {
    gl: WebGl2RenderingContext,
    samplers: HashMap<SamplerParameters, WebGlSampler>,
}

impl WebGlSamplerManager {
    fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            samplers: HashMap::new(),
        }
    }

    fn get_or_create_default_sampler(&mut self) -> Result<WebGlSampler, Error> {
        self.get_or_create_sampler(SamplerParameters::default())
    }

    fn get_or_create_sampler_by_iter<I>(&mut self, params: I) -> Result<WebGlSampler, Error>
    where
        I: IntoIterator<Item = WebGlSamplerParamWithValue>,
    {
        let mut sampler_params = SamplerParameters::default();
        params.into_iter().for_each(|param| match param {
            WebGlSamplerParamWithValue::MagnificationFilter(value) => {
                sampler_params.magnification_filter = value
            }
            WebGlSamplerParamWithValue::MinificationFilter(value) => {
                sampler_params.minification_filter = value
            }
            WebGlSamplerParamWithValue::WrapS(value) => sampler_params.wrap_s = value,
            WebGlSamplerParamWithValue::WrapT(value) => sampler_params.wrap_t = value,
            WebGlSamplerParamWithValue::WrapR(value) => sampler_params.wrap_r = value,
            WebGlSamplerParamWithValue::CompareFunction(value) => {
                sampler_params.compare_function = value
            }
            WebGlSamplerParamWithValue::CompareMode(value) => sampler_params.compare_mode = value,
            WebGlSamplerParamWithValue::MaxLod(value) => {
                sampler_params.max_lod = OrderedFloat(value)
            }
            WebGlSamplerParamWithValue::MinLod(value) => {
                sampler_params.min_lod = OrderedFloat(value)
            }
        });

        self.get_or_create_sampler(sampler_params)
    }

    fn get_or_create_sampler(&mut self, params: SamplerParameters) -> Result<WebGlSampler, Error> {
        if let Some(sampler) = self.samplers.get(&params) {
            return Ok(sampler.clone());
        }

        let sampler = self
            .gl
            .create_sampler()
            .ok_or(Error::CreateSamplerFailure)?;
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::MagnificationFilter.to_gl_enum(),
            params.magnification_filter.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::MinificationFilter.to_gl_enum(),
            params.minification_filter.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::WrapS.to_gl_enum(),
            params.wrap_s.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::WrapT.to_gl_enum(),
            params.wrap_t.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::WrapR.to_gl_enum(),
            params.wrap_r.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::CompareFunction.to_gl_enum(),
            params.compare_function.to_gl_enum() as i32,
        );
        self.gl.sampler_parameteri(
            &sampler,
            WebGlSamplerParam::CompareMode.to_gl_enum(),
            params.compare_mode.to_gl_enum() as i32,
        );
        self.gl.sampler_parameterf(
            &sampler,
            WebGlSamplerParam::MaxLod.to_gl_enum(),
            params.max_lod.0,
        );
        self.gl.sampler_parameterf(
            &sampler,
            WebGlSamplerParam::MinLod.to_gl_enum(),
            params.min_lod.0,
        );

        self.samplers.insert(params, sampler.clone());
        Ok(sampler)
    }
}

#[derive(Clone)]
pub struct WebGlTextureItem {
    gl_texture: WebGlTexture,
    gl_sampler: WebGlSampler,
    /// `levels` of texture layout here is safe to unwrap.
    layout: WebGlTextureLayoutWithSize,
    internal_format: WebGlTextureInternalFormat,
}

impl WebGlTextureItem {
    /// Returns native [`WebGlTexture`].
    pub fn gl_texture(&self) -> &WebGlTexture {
        &self.gl_texture
    }

    /// Returns native [`WebGlSampler`].
    pub fn gl_sampler(&self) -> &WebGlSampler {
        &self.gl_sampler
    }

    /// Returns [`WebGlTextureLayoutWithSize`].
    /// `levels` of texture layout here is safe to unwrap.
    pub fn layout(&self) -> WebGlTextureLayoutWithSize {
        self.layout
    }

    /// Returns [`WebGlTextureInternalFormat`].
    pub fn internal_format(&self) -> WebGlTextureInternalFormat {
        self.internal_format
    }
}

pub struct WebGlTextureManager {
    id: Uuid,
    gl: WebGl2RenderingContext,
    capabilities: WebGlCapabilities,
    channel: Channel,
    sampler_manager: WebGlSamplerManager,
    textures: Rc<RefCell<HashMap<Uuid, WebGlTextureItem>>>,
}

impl WebGlTextureManager {
    pub fn new(
        gl: WebGl2RenderingContext,
        capabilities: WebGlCapabilities,
        channel: Channel,
    ) -> Self {
        let textures = Rc::new(RefCell::new(HashMap::new()));
        channel.on(TextureDroppedHandler::new(Rc::clone(&textures)));

        Self {
            id: Uuid::new_v4(),
            capabilities,
            channel,
            sampler_manager: WebGlSamplerManager::new(gl.clone()),
            textures,
            gl,
        }
    }

    /// Returns texture manager id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Manages a [`WebGlTexturing`] and syncs its queueing [`TextureData`] into WebGl context.
    pub fn sync_texture(&mut self, texturing: &WebGlTexturing) -> Result<WebGlTextureItem, Error> {
        self.verify_manager(texturing)?;

        let mut textures = self.textures.borrow_mut();
        let item = match textures.entry(*texturing.id()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let gl_texture = self
                    .gl
                    .create_texture()
                    .ok_or(Error::CreateTextureFailure)?;
                let gl_sampler = match &texturing.options.sampler_parameters {
                    Some(params) => self
                        .sampler_manager
                        .get_or_create_sampler_by_iter(params.iter().cloned())?,
                    None => self.sampler_manager.get_or_create_default_sampler()?,
                };
                let layout = texturing.options.layout;
                let levels = layout.get_or_auto_levels();
                let internal_format = texturing.options.internal_format;

                self.gl.bind_texture(layout.to_gl_enum(), Some(&gl_texture));
                match layout {
                    WebGlTextureLayoutWithSize::Texture2D { width, height, .. }
                    | WebGlTextureLayoutWithSize::TextureCubeMap { width, height, .. } => {
                        self.gl.tex_storage_2d(
                            layout.to_gl_enum(),
                            levels as i32,
                            internal_format.to_gl_enum(),
                            width as i32,
                            height as i32,
                        )
                    }
                    WebGlTextureLayoutWithSize::Texture2DArray {
                        width,
                        height,
                        len: depth,
                        ..
                    }
                    | WebGlTextureLayoutWithSize::Texture3D {
                        width,
                        height,
                        depth,
                        ..
                    } => self.gl.tex_storage_3d(
                        layout.to_gl_enum(),
                        levels as i32,
                        internal_format.to_gl_enum(),
                        width as i32,
                        height as i32,
                        depth as i32,
                    ),
                };

                let item = WebGlTextureItem {
                    gl_texture,
                    gl_sampler,
                    layout,
                    internal_format,
                };
                texturing.set_managed(self.id, self.channel.clone());

                entry.insert(item)
            }
        };

        let WebGlTextureItem {
            gl_texture,
            gl_sampler,
            layout,
            internal_format,
        } = item;
        self.gl.bind_texture(layout.to_gl_enum(), Some(&gl_texture));
        for level in 0..layout.get_or_auto_levels() {
            for item in texturing.queue_of_level(level).drain() {
                let TexturingItem {
                    data,
                    dst_origin_x,
                    dst_origin_y,
                } = item;
                let Some(data) = data.as_webgl_texture_data() else {
                    warn!("texture data is not supported for WebGL, skipped");
                    continue;
                };
            }
        }
        self.gl.bind_texture(layout.to_gl_enum(), None);

        Ok(item.clone())
    }

    fn verify_manager(&self, texturing: &WebGlTexturing) -> Result<(), Error> {
        if let Some(manager_id) = texturing.manager_id() {
            if manager_id != self.id {
                return Err(Error::TextureManagedByOtherManager);
            }
        }

        Ok(())
    }
}

impl Drop for WebGlTextureManager {
    fn drop(&mut self) {
        self.channel
            .off::<TexturingDropped, TextureDroppedHandler>();
    }
}

/// A handler removes [`WebGlBufferItem`] from manager when a [`Buffer`] dropped.
/// This handler only removes items from [`WebGlBufferManager::buffers`], without unbinding them from WebGL context.
struct TextureDroppedHandler {
    textures: Rc<RefCell<HashMap<Uuid, WebGlTextureItem>>>,
}

impl TextureDroppedHandler {
    fn new(textures: Rc<RefCell<HashMap<Uuid, WebGlTextureItem>>>) -> Self {
        Self { textures }
    }
}

impl Handler<TexturingDropped> for TextureDroppedHandler {
    fn handle(&mut self, evt: &mut Event<'_, TexturingDropped>) {
        self.textures.borrow_mut().remove(evt.id());
    }
}
