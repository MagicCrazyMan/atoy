use std::{
    cell::RefCell,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use js_sys::{
    Float32Array, Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array,
    Uint8ClampedArray,
};
use log::warn;
use ordered_float::OrderedFloat;
use proc::GlEnum;
use tokio::{
    select,
    sync::broadcast::{self, error::RecvError},
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{
    DomException, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use crate::anewthing::texturing::{TextureCubeMapFace, Texturing, TexturingItem, TexturingMessage};

use super::{
    buffer::{WebGlBufferManager, WebGlBufferTarget, WebGlBuffering},
    capabilities::WebGlCapabilities,
    error::Error,
    pixel::{WebGlPixelDataType, WebGlPixelFormat, WebGlPixelUnpackStores},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WebGlTextureOptions {
    /// texture layout with size.
    pub layout: WebGlTextureLayoutWithSize,
    /// texture internal format.
    pub internal_format: WebGlTextureInternalFormat,
}

/// A wrapped [`Texturing`] with [`WebGlTextureOptions`].
#[derive(Debug, Clone)]
pub struct WebGlTexturing {
    pub texturing: Texturing,
    /// Create options of a texture.
    /// This field only works once, changing this does not influence anything.
    pub create_options: WebGlTextureOptions,
    /// Textures parameters.
    /// Manager will set a different sampler to the texture if this field changed.
    pub texture_parameters: WebGlTextureParameters,
    /// Sampler parameters to a texture sampler.
    /// Manager will set a different sampler to the texture if this field changed.
    pub sampler_parameters: WebGlSamplerParameters,
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
        /// Calculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
    },
    TextureCubeMap {
        /// Texture levels.
        /// Calculates automatically if `None`.
        levels: Option<usize>,
        /// Texture width.
        width: usize,
        /// Texture height.
        height: usize,
    },
    Texture2DArray {
        /// Texture levels.
        /// Calculates automatically if `None`.
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
        /// Calculates automatically if `None`.
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
    /// Returns as [`WebGlTextureLayout`].
    #[inline]
    pub fn as_layout(&self) -> WebGlTextureLayout {
        WebGlTextureLayout::from(*self)
    }

    #[inline]
    pub fn to_gl_enum(&self) -> u32 {
        WebGlTextureLayout::from(*self).to_gl_enum()
    }

    #[inline]
    fn get_or_auto_levels(&self) -> usize {
        match self {
            WebGlTextureLayoutWithSize::Texture2D {
                levels,
                width,
                height,
                ..
            }
            | WebGlTextureLayoutWithSize::TextureCubeMap {
                levels,
                width,
                height,
                ..
            }
            | WebGlTextureLayoutWithSize::Texture2DArray {
                levels,
                width,
                height,
                ..
            } => levels.unwrap_or_else(|| (*width.max(height) as f64).log2().floor() as usize + 1),
            WebGlTextureLayoutWithSize::Texture3D {
                levels,
                width,
                height,
                depth,
                ..
            } => levels.unwrap_or_else(|| {
                (*width.max(height).max(depth) as f64).log2().floor() as usize + 1
            }),
        }
    }

    fn tex_store(&self, gl: &WebGl2RenderingContext, internal_format: WebGlTextureInternalFormat) {
        let levels = self.get_or_auto_levels();
        match self {
            WebGlTextureLayoutWithSize::Texture2D { width, height, .. }
            | WebGlTextureLayoutWithSize::TextureCubeMap { width, height, .. } => gl
                .tex_storage_2d(
                    self.to_gl_enum(),
                    levels as i32,
                    internal_format.to_gl_enum(),
                    *width as i32,
                    *height as i32,
                ),
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
            } => gl.tex_storage_3d(
                self.to_gl_enum(),
                levels as i32,
                internal_format.to_gl_enum(),
                *width as i32,
                *height as i32,
                *depth as i32,
            ),
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

/// Available texture 2d targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTexture2DTarget {
    #[gl_enum(TEXTURE_2D)]
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
}

/// Available texture 3d targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTexture3DTarget {
    #[gl_enum(TEXTURE_2D_ARRAY)]
    Texture2DArray,
    #[gl_enum(TEXTURE_3D)]
    Texture3D,
}

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureTarget {
    #[gl_enum(TEXTURE_2D)]
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
    #[gl_enum(TEXTURE_2D_ARRAY)]
    Texture2DArray,
    #[gl_enum(TEXTURE_3D)]
    Texture3D,
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

impl WebGlTextureUnit {
    /// Returns the sequence index of texture unit.
    pub fn as_index(&self) -> i32 {
        match self {
            WebGlTextureUnit::Texture0 => 0,
            WebGlTextureUnit::Texture1 => 1,
            WebGlTextureUnit::Texture2 => 2,
            WebGlTextureUnit::Texture3 => 3,
            WebGlTextureUnit::Texture4 => 4,
            WebGlTextureUnit::Texture5 => 5,
            WebGlTextureUnit::Texture6 => 6,
            WebGlTextureUnit::Texture7 => 7,
            WebGlTextureUnit::Texture8 => 8,
            WebGlTextureUnit::Texture9 => 9,
            WebGlTextureUnit::Texture10 => 10,
            WebGlTextureUnit::Texture11 => 11,
            WebGlTextureUnit::Texture12 => 12,
            WebGlTextureUnit::Texture13 => 13,
            WebGlTextureUnit::Texture14 => 14,
            WebGlTextureUnit::Texture15 => 15,
            WebGlTextureUnit::Texture16 => 16,
            WebGlTextureUnit::Texture17 => 17,
            WebGlTextureUnit::Texture18 => 18,
            WebGlTextureUnit::Texture19 => 19,
            WebGlTextureUnit::Texture20 => 20,
            WebGlTextureUnit::Texture21 => 21,
            WebGlTextureUnit::Texture22 => 22,
            WebGlTextureUnit::Texture23 => 23,
            WebGlTextureUnit::Texture24 => 24,
            WebGlTextureUnit::Texture25 => 25,
            WebGlTextureUnit::Texture26 => 26,
            WebGlTextureUnit::Texture27 => 27,
            WebGlTextureUnit::Texture28 => 28,
            WebGlTextureUnit::Texture29 => 29,
            WebGlTextureUnit::Texture30 => 30,
            WebGlTextureUnit::Texture31 => 31,
        }
    }
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

/// Available texture plain internal formats mapped from [`WebGl2RenderingContext`].
/// Unsized internal formats are removed.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTexturePlainInternalFormat {
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

/// Available texture compressed formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureCompressedFormat {
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGB_S3TC_DXT1_EXT)]
    COMPRESSED_RGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT1_EXT)]
    COMPRESSED_RGBA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT3_EXT)]
    COMPRESSED_RGBA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    #[gl_enum(COMPRESSED_RGBA_S3TC_DXT5_EXT)]
    COMPRESSED_RGBA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_S3TC_DXT1_EXT)]
    COMPRESSED_SRGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT)]
    COMPRESSED_SRGB_ALPHA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT)]
    COMPRESSED_SRGB_ALPHA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT)]
    COMPRESSED_SRGB_ALPHA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_R11_EAC)]
    COMPRESSED_R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_R11_EAC)]
    COMPRESSED_SIGNED_R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RG11_EAC)]
    COMPRESSED_RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RG11_EAC)]
    COMPRESSED_SIGNED_RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RGB8_ETC2)]
    COMPRESSED_RGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ETC2)]
    COMPRESSED_RGBA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ETC2)]
    COMPRESSED_SRGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ETC2_EAC)]
    COMPRESSED_SRGB8_ALPHA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2)]
    COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2)]
    COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGB_PVRTC_2BPPV1_IMG)]
    COMPRESSED_RGB_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGBA_PVRTC_2BPPV1_IMG)]
    COMPRESSED_RGBA_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGB_PVRTC_4BPPV1_IMG)]
    COMPRESSED_RGB_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    #[gl_enum(COMPRESSED_RGBA_PVRTC_4BPPV1_IMG)]
    COMPRESSED_RGBA_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_etc1` enabled.
    #[gl_enum(COMPRESSED_RGB_ETC1_WEBGL)]
    COMPRESSED_RGB_ETC1_WEBGL,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_4X4_KHR)]
    COMPRESSED_RGBA_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_5X4_KHR)]
    COMPRESSED_RGBA_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_5X5_KHR)]
    COMPRESSED_RGBA_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_6X5_KHR)]
    COMPRESSED_RGBA_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_6X6_KHR)]
    COMPRESSED_RGBA_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X5_KHR)]
    COMPRESSED_RGBA_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X6_KHR)]
    COMPRESSED_RGBA_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_8X8_KHR)]
    COMPRESSED_RGBA_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X5_KHR)]
    COMPRESSED_RGBA_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X6_KHR)]
    COMPRESSED_RGBA_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_10X8_KHR)]
    COMPRESSED_RGBA_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_10X8_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_12X10_KHR)]
    COMPRESSED_RGBA_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_RGBA_ASTC_12X12_KHR)]
    COMPRESSED_RGBA_ASTC_12x12,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    #[gl_enum(COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR)]
    COMPRESSED_SRGB8_ALPHA8_ASTC_12x12,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGBA_BPTC_UNORM_EXT)]
    COMPRESSED_RGBA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT)]
    COMPRESSED_SRGB_ALPHA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT)]
    COMPRESSED_RGB_BPTC_SIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    #[gl_enum(COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT)]
    COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_RED_RGTC1_EXT)]
    COMPRESSED_RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RED_RGTC1_EXT)]
    COMPRESSED_SIGNED_RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_RED_GREEN_RGTC2_EXT)]
    COMPRESSED_RED_GREEN_RGTC2,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    #[gl_enum(COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT)]
    COMPRESSED_SIGNED_RED_GREEN_RGTC2,
}

impl WebGlTextureCompressedFormat {
    fn bytes_length_of(&self, width: usize, height: usize) -> usize {
        match self {
            // for S3TC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc/ for more details
            Self::COMPRESSED_RGB_S3TC_DXT1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_RGBA_S3TC_DXT1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_RGBA_S3TC_DXT3 => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGBA_S3TC_DXT5 => width.div_ceil(4) * height.div_ceil(4) * 16,
            // for S3TC RGBA, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc_srgb/ for more details
            Self::COMPRESSED_SRGB_S3TC_DXT1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_SRGB_ALPHA_S3TC_DXT1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_SRGB_ALPHA_S3TC_DXT3 => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SRGB_ALPHA_S3TC_DXT5 => width.div_ceil(4) * height.div_ceil(4) * 16,
            // for ETC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc/ for more details
            Self::COMPRESSED_R11_EAC => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_SIGNED_R11_EAC => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_RG11_EAC => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SIGNED_RG11_EAC => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGB8_ETC2 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_SRGB8_ETC2 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_RGBA8_ETC2_EAC => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ETC2_EAC => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                width.div_ceil(4) * height.div_ceil(4) * 8
            }
            Self::COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                width.div_ceil(4) * height.div_ceil(4) * 8
            }
            // for PVRTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_pvrtc/ for more details
            Self::COMPRESSED_RGB_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            Self::COMPRESSED_RGBA_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
            Self::COMPRESSED_RGB_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            Self::COMPRESSED_RGBA_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
            // for ETC1, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc1/ for more details
            Self::COMPRESSED_RGB_ETC1_WEBGL => width.div_ceil(4) * height.div_ceil(4) * 8,
            // for ASTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_astc/ for more details
            Self::COMPRESSED_RGBA_ASTC_4x4 => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_4x4 => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGBA_ASTC_5x4 => width.div_ceil(5) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_5x4 => width.div_ceil(5) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGBA_ASTC_5x5 => width.div_ceil(5) * height.div_ceil(5) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_5x5 => width.div_ceil(5) * height.div_ceil(5) * 16,
            Self::COMPRESSED_RGBA_ASTC_6x5 => width.div_ceil(6) * height.div_ceil(5) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_6x5 => width.div_ceil(6) * height.div_ceil(5) * 16,
            Self::COMPRESSED_RGBA_ASTC_6x6 => width.div_ceil(6) * height.div_ceil(6) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_6x6 => width.div_ceil(6) * height.div_ceil(6) * 16,
            Self::COMPRESSED_RGBA_ASTC_8x5 => width.div_ceil(8) * height.div_ceil(5) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_8x5 => width.div_ceil(8) * height.div_ceil(5) * 16,
            Self::COMPRESSED_RGBA_ASTC_8x6 => width.div_ceil(8) * height.div_ceil(6) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_8x6 => width.div_ceil(8) * height.div_ceil(6) * 16,
            Self::COMPRESSED_RGBA_ASTC_8x8 => width.div_ceil(8) * height.div_ceil(8) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_8x8 => width.div_ceil(8) * height.div_ceil(8) * 16,
            Self::COMPRESSED_RGBA_ASTC_10x5 => width.div_ceil(10) * height.div_ceil(5) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_10x5 => width.div_ceil(10) * height.div_ceil(5) * 16,
            Self::COMPRESSED_RGBA_ASTC_10x6 => width.div_ceil(10) * height.div_ceil(6) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_10x6 => width.div_ceil(10) * height.div_ceil(6) * 16,
            Self::COMPRESSED_RGBA_ASTC_10x10 => width.div_ceil(10) * height.div_ceil(10) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_10x10 => {
                width.div_ceil(10) * height.div_ceil(10) * 16
            }
            Self::COMPRESSED_RGBA_ASTC_12x10 => width.div_ceil(12) * height.div_ceil(10) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_12x10 => {
                width.div_ceil(12) * height.div_ceil(10) * 16
            }
            Self::COMPRESSED_RGBA_ASTC_12x12 => width.div_ceil(12) * height.div_ceil(12) * 16,
            Self::COMPRESSED_SRGB8_ALPHA8_ASTC_12x12 => {
                width.div_ceil(12) * height.div_ceil(12) * 16
            }
            // for BPTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_bptc/ for more details
            Self::COMPRESSED_RGBA_BPTC_UNORM => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SRGB_ALPHA_BPTC_UNORM => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGB_BPTC_SIGNED_FLOAT => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT => width.div_ceil(4) * height.div_ceil(4) * 16,
            // for RGTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_rgtc/ for more details
            Self::COMPRESSED_RED_RGTC1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_SIGNED_RED_RGTC1 => width.div_ceil(4) * height.div_ceil(4) * 8,
            Self::COMPRESSED_RED_GREEN_RGTC2 => width.div_ceil(4) * height.div_ceil(4) * 16,
            Self::COMPRESSED_SIGNED_RED_GREEN_RGTC2 => width.div_ceil(4) * height.div_ceil(4) * 16,
        }
    }

    fn check_compressed_format_supported(&self, capabilities: &WebGlCapabilities) {
        let (name, supported) = match self {
            WebGlTextureCompressedFormat::COMPRESSED_RGB_S3TC_DXT1
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_S3TC_DXT1
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_S3TC_DXT3
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_S3TC_DXT5 => (
                "WEBGL_compressed_texture_s3tc",
                capabilities.compressed_s3tc_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_SRGB_S3TC_DXT1
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB_ALPHA_S3TC_DXT1
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB_ALPHA_S3TC_DXT3
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB_ALPHA_S3TC_DXT5 => (
                "WEBGL_compressed_texture_s3tc_srgb",
                capabilities.compressed_s3tc_srgb_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_R11_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_SIGNED_R11_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_RG11_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_SIGNED_RG11_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_RGB8_ETC2
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA8_ETC2_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ETC2
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ETC2_EAC
            | WebGlTextureCompressedFormat::COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => (
                "WEBGL_compressed_texture_etc",
                capabilities.compressed_etc_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_RGB_PVRTC_2BPPV1_IMG
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_PVRTC_2BPPV1_IMG
            | WebGlTextureCompressedFormat::COMPRESSED_RGB_PVRTC_4BPPV1_IMG
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_PVRTC_4BPPV1_IMG => (
                "WEBGL_compressed_texture_pvrtc",
                capabilities.compressed_pvrtc_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_RGB_ETC1_WEBGL => (
                "WEBGL_compressed_texture_etc1",
                capabilities.compressed_etc1_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_4x4
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_4x4
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_5x4
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_5x4
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_5x5
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_5x5
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_6x5
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_6x5
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_6x6
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_6x6
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_8x5
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_8x5
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_8x6
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_8x6
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_8x8
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_8x8
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_10x5
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_10x5
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_10x6
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_10x6
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_10x10
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_10x10
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_12x10
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_12x10
            | WebGlTextureCompressedFormat::COMPRESSED_RGBA_ASTC_12x12
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB8_ALPHA8_ASTC_12x12 => (
                "WEBGL_compressed_texture_astc",
                capabilities.compressed_astc_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_RGBA_BPTC_UNORM
            | WebGlTextureCompressedFormat::COMPRESSED_SRGB_ALPHA_BPTC_UNORM
            | WebGlTextureCompressedFormat::COMPRESSED_RGB_BPTC_SIGNED_FLOAT
            | WebGlTextureCompressedFormat::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT => (
                "EXT_texture_compression_bptc",
                capabilities.compressed_bptc_supported(),
            ),
            WebGlTextureCompressedFormat::COMPRESSED_RED_RGTC1
            | WebGlTextureCompressedFormat::COMPRESSED_SIGNED_RED_RGTC1
            | WebGlTextureCompressedFormat::COMPRESSED_RED_GREEN_RGTC2
            | WebGlTextureCompressedFormat::COMPRESSED_SIGNED_RED_GREEN_RGTC2 => (
                "EXT_texture_compression_rgtc",
                capabilities.compressed_rgtc_supported(),
            ),
        };

        if !supported {
            warn!("compressed format {name} does not supported.");
        }
    }
}

/// Available texture internal formats mapped from [`WebGl2RenderingContext`],
/// including plain internal formats and compressed formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlTextureInternalFormat {
    Plain(WebGlTexturePlainInternalFormat),
    Compressed(WebGlTextureCompressedFormat),
}

impl WebGlTextureInternalFormat {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            WebGlTextureInternalFormat::Plain(f) => f.to_gl_enum(),
            WebGlTextureInternalFormat::Compressed(f) => f.to_gl_enum(),
        }
    }
}

/// Available texture parameters mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureParameter {
    #[gl_enum(TEXTURE_BASE_LEVEL)]
    BaseLevel,
    #[gl_enum(TEXTURE_MAX_LEVEL)]
    MaxLevel,
}

/// A collection of texture parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebGlTextureParameters {
    pub base_level: usize,
    pub max_level: usize,
}

impl Default for WebGlTextureParameters {
    fn default() -> Self {
        Self {
            base_level: 0,
            max_level: 1000,
        }
    }
}

impl WebGlTextureParameters {
    fn set_texture_parameters(&self, gl: &WebGl2RenderingContext, layout: WebGlTextureLayout) {
        gl.tex_parameteri(
            layout.to_gl_enum(),
            WebGlTextureParameter::BaseLevel.to_gl_enum(),
            self.base_level as i32,
        );
        gl.tex_parameteri(
            layout.to_gl_enum(),
            WebGlTextureParameter::MaxLevel.to_gl_enum(),
            self.max_level as i32,
        );
    }
}

/// Available texture sample parameters for [`WebGlSampler`] mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlSamplerParameter {
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
    /// Available when extension `EXT_texture_filter_anisotropic` is enabled.
    #[gl_enum(TEXTURE_MAX_ANISOTROPY_EXT)]
    MaxAnisotropy,
}

/// A collection of sampler parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebGlSamplerParameters {
    pub magnification_filter: WebGlSampleMagnificationFilter,
    pub minification_filter: WebGlSampleMinificationFilter,
    pub wrap_s: WebGlSampleWrapMethod,
    pub wrap_t: WebGlSampleWrapMethod,
    pub wrap_r: WebGlSampleWrapMethod,
    pub compare_function: WebGlSampleCompareFunction,
    pub compare_mode: WebGlSampleCompareMode,
    pub max_lod: OrderedFloat<f32>,
    pub min_lod: OrderedFloat<f32>,
    /// Available when extension `EXT_texture_filter_anisotropic` is enabled.
    pub max_anisotropy: Option<OrderedFloat<f32>>,
}

impl Default for WebGlSamplerParameters {
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
            max_anisotropy: None,
        }
    }
}

impl WebGlSamplerParameters {
    fn set_sampler_parameters(
        &self,
        gl: &WebGl2RenderingContext,
        sampler: &WebGlSampler,
        capabilities: &WebGlCapabilities,
    ) {
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::MagnificationFilter.to_gl_enum(),
            self.magnification_filter.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::MinificationFilter.to_gl_enum(),
            self.minification_filter.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::WrapS.to_gl_enum(),
            self.wrap_s.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::WrapT.to_gl_enum(),
            self.wrap_t.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::WrapR.to_gl_enum(),
            self.wrap_r.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::CompareFunction.to_gl_enum(),
            self.compare_function.to_gl_enum() as i32,
        );
        gl.sampler_parameteri(
            sampler,
            WebGlSamplerParameter::CompareMode.to_gl_enum(),
            self.compare_mode.to_gl_enum() as i32,
        );
        gl.sampler_parameterf(
            sampler,
            WebGlSamplerParameter::MaxLod.to_gl_enum(),
            self.max_lod.0,
        );
        gl.sampler_parameterf(
            sampler,
            WebGlSamplerParameter::MinLod.to_gl_enum(),
            self.min_lod.0,
        );

        let max_anisotropy = if let Some(max_anisotropy) = &self.max_anisotropy {
            capabilities.texture_filter_anisotropic_supported();
            max_anisotropy.0
        } else {
            1.0
        };
        gl.sampler_parameterf(
            sampler,
            WebGlSamplerParameter::MaxAnisotropy.to_gl_enum(),
            max_anisotropy,
        );
    }
}

/// Available uncompressed texture data types.
pub enum WebGlPlainTextureData<'a> {
    /// Pixel data type of binary is restricted to [`WebGlPixelDataType::UnsignedByte`].
    Binary {
        width: usize,
        height: usize,
        data: &'a [u8],
        bytes_offset: Option<usize>,
    },
    PixelBufferObject {
        pixel_data_type: WebGlPixelDataType,
        width: usize,
        height: usize,
        buffering: &'a WebGlBuffering,
        bytes_offset: Option<usize>,
    },
    /// Pixel data type of Int8Array is restricted to [`WebGlPixelDataType::Byte`].
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Uint8Array is restricted to [`WebGlPixelDataType::UnsignedByte`].
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Uint8ClampedArray is restricted to [`WebGlPixelDataType::UnsignedByte`].
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Int32Array is restricted to [`WebGlPixelDataType::Short`].
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Uint16Array can be [`WebGlPixelDataType::UnsignedShort`],
    /// [`WebGlPixelDataType::UnsignedShort_5_6_5`], [`WebGlPixelDataType::UnsignedShort_5_5_5_1`],
    /// [`WebGlPixelDataType::UnsignedShort_4_4_4_4`] or [`WebGlPixelDataType::HalfFloat`].
    Uint16Array {
        pixel_data_type: WebGlPixelDataType,
        width: usize,
        height: usize,
        data: Uint16Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Int32Array is restricted to [`WebGlPixelDataType::Int`].
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Uint32Array can be [`WebGlPixelDataType::UnsignedInt`],
    /// [`WebGlPixelDataType::UnsignedInt_5_9_9_9Rev`], [`WebGlPixelDataType::UnsignedInt_2_10_10_10Rev`],
    /// [`WebGlPixelDataType::UnsignedInt_10F_11F_11F_Rev`] or [`WebGlPixelDataType::UnsignedInt_24_8`].
    Uint32Array {
        pixel_data_type: WebGlPixelDataType,
        width: usize,
        height: usize,
        data: Uint32Array,
        element_offset: Option<usize>,
    },
    /// Pixel data type of Float32Array is restricted to [`WebGlPixelDataType::Float`].
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        element_offset: Option<usize>,
    },
    HtmlCanvasElement {
        pixel_data_type: WebGlPixelDataType,
        data: HtmlCanvasElement,
    },
    HtmlImageElement {
        pixel_data_type: WebGlPixelDataType,
        data: HtmlImageElement,
    },
    HtmlVideoElement {
        pixel_data_type: WebGlPixelDataType,
        data: HtmlVideoElement,
    },
    ImageData {
        pixel_data_type: WebGlPixelDataType,
        data: ImageData,
    },
    ImageBitmap {
        pixel_data_type: WebGlPixelDataType,
        data: ImageBitmap,
    },
}

impl<'a> WebGlPlainTextureData<'a> {
    fn width(&self) -> usize {
        match self {
            WebGlPlainTextureData::Binary { width, .. }
            | WebGlPlainTextureData::PixelBufferObject { width, .. }
            | WebGlPlainTextureData::Int8Array { width, .. }
            | WebGlPlainTextureData::Uint8Array { width, .. }
            | WebGlPlainTextureData::Uint8ClampedArray { width, .. }
            | WebGlPlainTextureData::Int16Array { width, .. }
            | WebGlPlainTextureData::Uint16Array { width, .. }
            | WebGlPlainTextureData::Int32Array { width, .. }
            | WebGlPlainTextureData::Uint32Array { width, .. }
            | WebGlPlainTextureData::Float32Array { width, .. } => *width,
            WebGlPlainTextureData::HtmlCanvasElement { data, .. } => data.width() as usize,
            WebGlPlainTextureData::HtmlImageElement { data, .. } => data.natural_width() as usize,
            WebGlPlainTextureData::HtmlVideoElement { data, .. } => data.video_width() as usize,
            WebGlPlainTextureData::ImageData { data, .. } => data.width() as usize,
            WebGlPlainTextureData::ImageBitmap { data, .. } => data.width() as usize,
        }
    }

    fn height(&self) -> usize {
        match self {
            WebGlPlainTextureData::Binary { height, .. }
            | WebGlPlainTextureData::PixelBufferObject { height, .. }
            | WebGlPlainTextureData::Int8Array { height, .. }
            | WebGlPlainTextureData::Uint8Array { height, .. }
            | WebGlPlainTextureData::Uint8ClampedArray { height, .. }
            | WebGlPlainTextureData::Int16Array { height, .. }
            | WebGlPlainTextureData::Uint16Array { height, .. }
            | WebGlPlainTextureData::Int32Array { height, .. }
            | WebGlPlainTextureData::Uint32Array { height, .. }
            | WebGlPlainTextureData::Float32Array { height, .. } => *height,
            WebGlPlainTextureData::HtmlCanvasElement { data, .. } => data.height() as usize,
            WebGlPlainTextureData::HtmlImageElement { data, .. } => data.natural_height() as usize,
            WebGlPlainTextureData::HtmlVideoElement { data, .. } => data.video_height() as usize,
            WebGlPlainTextureData::ImageData { data, .. } => data.height() as usize,
            WebGlPlainTextureData::ImageBitmap { data, .. } => data.height() as usize,
        }
    }

    fn upload(
        self,
        gl: &WebGl2RenderingContext,
        layout: &WebGlTextureLayoutWithSize,
        cube_map_face: TextureCubeMapFace,
        pixel_format: WebGlPixelFormat,
        pixel_unpack_stores: WebGlPixelUnpackStores,
        level: usize,
        dst_origin_x: Option<usize>,
        dst_origin_y: Option<usize>,
        dst_origin_z: Option<usize>,
        dst_width: Option<usize>,
        dst_height: Option<usize>,
        dst_depth_or_len: Option<usize>,
        buffer_manager: &mut WebGlBufferManager,
        using_ubos: &mut HashMap<usize, (WebGlBuffer, Option<(usize, usize)>)>,
    ) -> Result<(), Error> {
        let dst_origin_x = dst_origin_x.unwrap_or(0);
        let dst_origin_y = dst_origin_y.unwrap_or(0);
        let dst_origin_z = dst_origin_z.unwrap_or(0);
        let dst_width = dst_width.unwrap_or(self.width());
        let dst_height = dst_height.unwrap_or(self.height());
        let dst_depth_or_len = dst_depth_or_len.unwrap_or(0);
        let target = match layout {
            WebGlTextureLayoutWithSize::Texture2D { .. } => WebGlTextureTarget::Texture2D,
            WebGlTextureLayoutWithSize::TextureCubeMap { .. } => match cube_map_face {
                TextureCubeMapFace::PositiveX => WebGlTextureTarget::TextureCubeMapPositiveX,
                TextureCubeMapFace::NegativeX => WebGlTextureTarget::TextureCubeMapNegativeX,
                TextureCubeMapFace::PositiveY => WebGlTextureTarget::TextureCubeMapPositiveY,
                TextureCubeMapFace::NegativeY => WebGlTextureTarget::TextureCubeMapNegativeY,
                TextureCubeMapFace::PositiveZ => WebGlTextureTarget::TextureCubeMapPositiveZ,
                TextureCubeMapFace::NegativeZ => WebGlTextureTarget::TextureCubeMapNegativeZ,
            },
            WebGlTextureLayoutWithSize::Texture2DArray { .. } => WebGlTextureTarget::Texture2DArray,
            WebGlTextureLayoutWithSize::Texture3D { .. } => WebGlTextureTarget::Texture3D,
        };
        let is3d = match layout {
            WebGlTextureLayoutWithSize::Texture2D { .. } => false,
            WebGlTextureLayoutWithSize::TextureCubeMap { .. } => false,
            WebGlTextureLayoutWithSize::Texture2DArray { .. } => true,
            WebGlTextureLayoutWithSize::Texture3D { .. } => true,
        };

        // sets pixel ubpack stores
        pixel_unpack_stores.set_pixel_store(gl);

        match self {
            WebGlPlainTextureData::Binary {
                data, bytes_offset, ..
            } => {
                let bytes_offset = bytes_offset.unwrap_or(0);
                match is3d {
                    true => gl.tex_sub_image_3d_with_opt_u8_array_and_src_offset(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        WebGlPixelDataType::UnsignedByte.to_gl_enum(),
                        Some(data),
                        bytes_offset as u32,
                    ).unwrap(),
                    false => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        WebGlPixelDataType::UnsignedByte.to_gl_enum(),
                        data,
                        bytes_offset as u32,
                    ).unwrap(),
                };
            }
            WebGlPlainTextureData::PixelBufferObject {
                pixel_data_type,
                buffering,
                bytes_offset,
                ..
            } => {
                let item = buffer_manager.sync_buffering(buffering, using_ubos)?;
                gl.bind_buffer(
                    WebGlBufferTarget::PixelUnpackBuffer.to_gl_enum(),
                    Some(item.gl_buffer()),
                );
                let bytes_offset = bytes_offset.unwrap_or(0);
                match is3d {
                    true => gl
                        .tex_sub_image_3d_with_i32(
                            target.to_gl_enum(),
                            level as i32,
                            dst_origin_x as i32,
                            dst_origin_y as i32,
                            dst_origin_z as i32,
                            dst_width as i32,
                            dst_height as i32,
                            dst_depth_or_len as i32,
                            pixel_format.to_gl_enum(),
                            pixel_data_type.to_gl_enum(),
                            bytes_offset as i32,
                        )
                        .unwrap(),
                    false => gl
                        .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                            target.to_gl_enum(),
                            level as i32,
                            dst_origin_x as i32,
                            dst_origin_y as i32,
                            dst_width as i32,
                            dst_height as i32,
                            pixel_format.to_gl_enum(),
                            pixel_data_type.to_gl_enum(),
                            bytes_offset as i32,
                        )
                        .unwrap(),
                };
                gl.bind_buffer(WebGlBufferTarget::PixelUnpackBuffer.to_gl_enum(), None);
            }
            WebGlPlainTextureData::HtmlCanvasElement {
                pixel_data_type,
                data,
            } => match is3d {
                true => gl
                    .tex_sub_image_3d_with_html_canvas_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
                false => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
            },
            WebGlPlainTextureData::HtmlImageElement {
                pixel_data_type,
                data,
            } => match is3d {
                true => gl
                    .tex_sub_image_3d_with_html_image_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
                false => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
            },
            WebGlPlainTextureData::HtmlVideoElement {
                pixel_data_type,
                data,
            } => match is3d {
                true => gl
                    .tex_sub_image_3d_with_html_video_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
                false => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
            },
            WebGlPlainTextureData::ImageData {
                pixel_data_type,
                data,
            } => match is3d {
                true => gl
                    .tex_sub_image_3d_with_image_data(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
                false => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
            },
            WebGlPlainTextureData::ImageBitmap {
                pixel_data_type,
                data,
            } => match is3d {
                true => gl
                    .tex_sub_image_3d_with_image_bitmap(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
                false => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        &data,
                    )
                    .map_err(|err| {
                        Error::TextureImageSourceError(err.dyn_into::<DomException>().unwrap())
                    })?,
            },
            _ => {
                let (data, pixel_data_type, element_offset) = match self {
                    WebGlPlainTextureData::Int8Array {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::BYTE,
                        element_offset,
                    ),
                    WebGlPlainTextureData::Uint8Array {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::UNSIGNED_BYTE,
                        element_offset,
                    ),
                    WebGlPlainTextureData::Uint8ClampedArray {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::UNSIGNED_BYTE,
                        element_offset,
                    ),
                    WebGlPlainTextureData::Int16Array {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::SHORT,
                        element_offset,
                    ),
                    WebGlPlainTextureData::Uint16Array {
                        pixel_data_type,
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        pixel_data_type.to_gl_enum(),
                        element_offset,
                    ),
                    WebGlPlainTextureData::Int32Array {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::INT,
                        element_offset,
                    ),
                    WebGlPlainTextureData::Uint32Array {
                        pixel_data_type,
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        pixel_data_type.to_gl_enum(),
                        element_offset,
                    ),
                    WebGlPlainTextureData::Float32Array {
                        data,
                        element_offset,
                        ..
                    } => (
                        Object::from(data),
                        WebGl2RenderingContext::FLOAT,
                        element_offset,
                    ),
                    _ => unreachable!(),
                };
                let element_offset = element_offset.clone().unwrap_or(0);
                // those calls never throws an error
                match is3d {
                    true => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type,
                        Some(&data),
                        element_offset as u32,
                    ).unwrap(),
                    false => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type,
                        &data,
                        element_offset as u32,
                    ).unwrap(),
                };
            }
        };

        // reset pixel unpack stores
        WebGlPixelUnpackStores::default().set_pixel_store(gl);

        Ok(())
    }
}

/// Available compressed texture data types.
pub enum WebGlCompressedTextureData<'a> {
    Binary {
        width: usize,
        height: usize,
        data: &'a mut [u8],
        bytes_offset: Option<usize>,
        bytes_length_override: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffering: &'a WebGlBuffering,
        bytes_offset: Option<usize>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Uint16Array {
        width: usize,
        height: usize,
        data: Uint16Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Uint32Array {
        width: usize,
        height: usize,
        data: Uint32Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        element_offset: Option<usize>,
        element_length_override: Option<usize>,
    },
}

impl<'a> WebGlCompressedTextureData<'a> {
    fn width(&self) -> usize {
        match self {
            WebGlCompressedTextureData::Binary { width, .. }
            | WebGlCompressedTextureData::PixelBufferObject { width, .. }
            | WebGlCompressedTextureData::Int8Array { width, .. }
            | WebGlCompressedTextureData::Uint8Array { width, .. }
            | WebGlCompressedTextureData::Uint8ClampedArray { width, .. }
            | WebGlCompressedTextureData::Int16Array { width, .. }
            | WebGlCompressedTextureData::Uint16Array { width, .. }
            | WebGlCompressedTextureData::Int32Array { width, .. }
            | WebGlCompressedTextureData::Uint32Array { width, .. }
            | WebGlCompressedTextureData::Float32Array { width, .. } => *width,
        }
    }

    fn height(&self) -> usize {
        match self {
            WebGlCompressedTextureData::Binary { height, .. }
            | WebGlCompressedTextureData::PixelBufferObject { height, .. }
            | WebGlCompressedTextureData::Int8Array { height, .. }
            | WebGlCompressedTextureData::Uint8Array { height, .. }
            | WebGlCompressedTextureData::Uint8ClampedArray { height, .. }
            | WebGlCompressedTextureData::Int16Array { height, .. }
            | WebGlCompressedTextureData::Uint16Array { height, .. }
            | WebGlCompressedTextureData::Int32Array { height, .. }
            | WebGlCompressedTextureData::Uint32Array { height, .. }
            | WebGlCompressedTextureData::Float32Array { height, .. } => *height,
        }
    }

    fn upload(
        self,
        gl: &WebGl2RenderingContext,
        layout: &WebGlTextureLayoutWithSize,
        cube_map_face: TextureCubeMapFace,
        compressed_format: WebGlTextureCompressedFormat,
        level: usize,
        dst_origin_x: Option<usize>,
        dst_origin_y: Option<usize>,
        dst_origin_z: Option<usize>,
        dst_width: Option<usize>,
        dst_height: Option<usize>,
        dst_depth_or_len: Option<usize>,
        buffer_manager: &mut WebGlBufferManager,
        using_ubos: &mut HashMap<usize, (WebGlBuffer, Option<(usize, usize)>)>,
    ) -> Result<(), Error> {
        let dst_origin_x = dst_origin_x.unwrap_or(0);
        let dst_origin_y = dst_origin_y.unwrap_or(0);
        let dst_origin_z = dst_origin_z.unwrap_or(0);
        let dst_width = dst_width.unwrap_or(self.width());
        let dst_height = dst_height.unwrap_or(self.height());
        let dst_depth_or_len = dst_depth_or_len.unwrap_or(0);
        let target = match layout {
            WebGlTextureLayoutWithSize::Texture2D { .. } => WebGlTextureTarget::Texture2D,
            WebGlTextureLayoutWithSize::TextureCubeMap { .. } => match cube_map_face {
                TextureCubeMapFace::PositiveX => WebGlTextureTarget::TextureCubeMapPositiveX,
                TextureCubeMapFace::NegativeX => WebGlTextureTarget::TextureCubeMapNegativeX,
                TextureCubeMapFace::PositiveY => WebGlTextureTarget::TextureCubeMapPositiveY,
                TextureCubeMapFace::NegativeY => WebGlTextureTarget::TextureCubeMapNegativeY,
                TextureCubeMapFace::PositiveZ => WebGlTextureTarget::TextureCubeMapPositiveZ,
                TextureCubeMapFace::NegativeZ => WebGlTextureTarget::TextureCubeMapNegativeZ,
            },
            WebGlTextureLayoutWithSize::Texture2DArray { .. } => WebGlTextureTarget::Texture2DArray,
            WebGlTextureLayoutWithSize::Texture3D { .. } => WebGlTextureTarget::Texture3D,
        };
        let is3d = match layout {
            WebGlTextureLayoutWithSize::Texture2D { .. } => false,
            WebGlTextureLayoutWithSize::TextureCubeMap { .. } => false,
            WebGlTextureLayoutWithSize::Texture2DArray { .. } => true,
            WebGlTextureLayoutWithSize::Texture3D { .. } => true,
        };
        match self {
            WebGlCompressedTextureData::Binary {
                data,
                bytes_offset,
                bytes_length_override,
                ..
            } => {
                let bytes_offset = bytes_offset.unwrap_or(0);
                let bytes_length_override = bytes_length_override.unwrap_or(0);
                match is3d {
                    true => gl
                        .compressed_tex_sub_image_3d_with_u8_array_and_u32_and_src_length_override(
                            target.to_gl_enum(),
                            level as i32,
                            dst_origin_x as i32,
                            dst_origin_y as i32,
                            dst_origin_z as i32,
                            dst_width as i32,
                            dst_height as i32,
                            dst_depth_or_len as i32,
                            compressed_format.to_gl_enum(),
                            data,
                            bytes_offset as u32,
                            bytes_length_override as u32,
                        ),
                    false => gl
                        .compressed_tex_sub_image_2d_with_u8_array_and_u32_and_src_length_override(
                            target.to_gl_enum(),
                            level as i32,
                            dst_origin_x as i32,
                            dst_origin_y as i32,
                            dst_width as i32,
                            dst_height as i32,
                            compressed_format.to_gl_enum(),
                            data,
                            bytes_offset as u32,
                            bytes_length_override as u32,
                        ),
                };
            }
            WebGlCompressedTextureData::PixelBufferObject {
                buffering,
                bytes_offset,
                ..
            } => {
                let item = buffer_manager.sync_buffering(buffering, using_ubos)?;
                gl.bind_buffer(
                    WebGlBufferTarget::PixelUnpackBuffer.to_gl_enum(),
                    Some(item.gl_buffer()),
                );
                let bytes_length = compressed_format.bytes_length_of(dst_width, dst_height);
                let bytes_offset = bytes_offset.unwrap_or(0);
                match is3d {
                    true => gl.compressed_tex_sub_image_3d_with_i32_and_i32(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        compressed_format.to_gl_enum(),
                        bytes_length as i32,
                        bytes_offset as i32,
                    ),
                    false => gl.compressed_tex_sub_image_2d_with_i32_and_i32(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        compressed_format.to_gl_enum(),
                        bytes_length as i32,
                        bytes_offset as i32,
                    ),
                };
                gl.bind_buffer(WebGlBufferTarget::PixelUnpackBuffer.to_gl_enum(), None);
            }
            _ => {
                let (data, element_offset, element_length_override) = match self {
                    WebGlCompressedTextureData::Int8Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Uint8Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Uint8ClampedArray {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Int16Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Uint16Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Int32Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Uint32Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    WebGlCompressedTextureData::Float32Array {
                        data,
                        element_offset,
                        element_length_override,
                        ..
                    } => (Object::from(data), element_offset, element_length_override),
                    _ => unreachable!(),
                };
                let element_offset = element_offset.clone().unwrap_or(0);
                let element_length_override = element_length_override.clone().unwrap_or(0);
                match is3d {
                    true => gl.compressed_tex_sub_image_3d_with_array_buffer_view_and_u32_and_src_length_override(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_origin_z as i32,
                        dst_width as i32,
                        dst_height as i32,
                        dst_depth_or_len as i32,
                        compressed_format.to_gl_enum(),
                        &data,
                        element_offset as u32,
                        element_length_override as u32
                    ),
                    false => gl.compressed_tex_sub_image_2d_with_array_buffer_view_and_u32_and_src_length_override(
                        target.to_gl_enum(),
                        level as i32,
                        dst_origin_x as i32,
                        dst_origin_y as i32,
                        dst_width as i32,
                        dst_height as i32,
                        compressed_format.to_gl_enum(),
                        &data,
                        element_offset as u32,
                        element_length_override as u32
                    ),
                };
            }
        };

        Ok(())
    }
}

/// Texture data for uploading data to WebGL runtime.
pub enum WebGlTextureData<'a> {
    Plain {
        pixel_format: WebGlPixelFormat,
        pixel_unpack_stores: WebGlPixelUnpackStores,
        generate_mipmap: bool,
        data: WebGlPlainTextureData<'a>,
    },
    Compressed {
        data: WebGlCompressedTextureData<'a>,
    },
}

struct WebGlSamplerManager {
    gl: WebGl2RenderingContext,
    samplers: HashMap<WebGlSamplerParameters, WebGlSampler>,
}

impl WebGlSamplerManager {
    fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            samplers: HashMap::new(),
        }
    }

    fn get_or_create_sampler(
        &mut self,
        params: WebGlSamplerParameters,
        capabilities: &WebGlCapabilities,
    ) -> Result<WebGlSampler, Error> {
        if let Some(sampler) = self.samplers.get(&params) {
            return Ok(sampler.clone());
        }

        let sampler = self
            .gl
            .create_sampler()
            .ok_or(Error::CreateSamplerFailure)?;
        params.set_sampler_parameters(&self.gl, &sampler, capabilities);

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

    sampler_manager: WebGlSamplerManager,
    textures: Rc<RefCell<HashMap<Uuid, WebGlTextureItem>>>,

    abortion: broadcast::Sender<()>,
}

impl WebGlTextureManager {
    /// Constructs a new texture manager.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        let textures = Rc::new(RefCell::new(HashMap::new()));

        Self {
            id: Uuid::new_v4(),
            sampler_manager: WebGlSamplerManager::new(gl.clone()),
            textures,
            gl,

            abortion: broadcast::channel(5).0,
        }
    }

    /// Returns texture manager id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Manages a [`WebGlTexturing`] and syncs its queueing [`TextureData`](super::super::super::texturing::TextureData) into WebGl context.
    pub fn sync_texturing(
        &mut self,
        texturing: &WebGlTexturing,
        activating_texture_unit: WebGlTextureUnit,
        using_textures: &HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        using_ubos: &mut HashMap<usize, (WebGlBuffer, Option<(usize, usize)>)>,
        buffer_manager: &mut WebGlBufferManager,
        capabilities: &WebGlCapabilities,
    ) -> Result<WebGlTextureItem, Error> {
        let layout = texturing.create_options.layout;
        let internal_format = texturing.create_options.internal_format;

        let gl_sampler = self
            .sampler_manager
            .get_or_create_sampler(texturing.sampler_parameters, capabilities)?;

        let mut textures = self.textures.borrow_mut();
        let item = match textures.entry(*texturing.id()) {
            Entry::Occupied(entry) => {
                let item = entry.into_mut();
                item.gl_sampler = gl_sampler;
                self.gl
                    .bind_texture(item.layout.to_gl_enum(), Some(&item.gl_texture));
                item
            }
            Entry::Vacant(entry) => {
                // checks whether compressed format is supported.
                // Throws no error even is not supported, prints a warning log only.
                if let WebGlTextureInternalFormat::Compressed(f) = internal_format {
                    f.check_compressed_format_supported(capabilities);
                }

                let gl_texture = self
                    .gl
                    .create_texture()
                    .ok_or(Error::CreateTextureFailure)?;

                self.gl.bind_texture(layout.to_gl_enum(), Some(&gl_texture));
                layout.tex_store(&self.gl, internal_format);

                let item = WebGlTextureItem {
                    gl_texture,
                    gl_sampler,
                    layout,
                    internal_format,
                };

                self.listen_texturing_dropped(texturing);

                entry.insert(item)
            }
        };

        texturing
            .texture_parameters
            .set_texture_parameters(&self.gl, layout.as_layout());

        for level in 0..layout.get_or_auto_levels() {
            for item in texturing.queue_of_level(level).drain() {
                let TexturingItem {
                    data,
                    cube_map_face,
                    dst_origin_x,
                    dst_origin_y,
                    dst_origin_z,
                    dst_width,
                    dst_height,
                    dst_depth_or_len,
                } = item;
                let Some(data) = data.as_webgl_texture_data() else {
                    warn!("texture data is not supported for WebGL, skipped");
                    continue;
                };

                match (data, internal_format) {
                    (
                        WebGlTextureData::Plain {
                            pixel_format,
                            pixel_unpack_stores: pixel_stores,
                            generate_mipmap,
                            data,
                        },
                        WebGlTextureInternalFormat::Plain(_),
                    ) => {
                        data.upload(
                            &self.gl,
                            &layout,
                            cube_map_face,
                            pixel_format,
                            pixel_stores,
                            level,
                            dst_origin_x,
                            dst_origin_y,
                            dst_origin_z,
                            dst_width,
                            dst_height,
                            dst_depth_or_len,
                            buffer_manager,
                            using_ubos,
                        )?;

                        if generate_mipmap {
                            self.gl.generate_mipmap(layout.to_gl_enum());
                        }
                    }
                    (
                        WebGlTextureData::Compressed { data },
                        WebGlTextureInternalFormat::Compressed(compressed_format),
                    ) => data.upload(
                        &self.gl,
                        &layout,
                        cube_map_face,
                        compressed_format,
                        level,
                        dst_origin_x,
                        dst_origin_y,
                        dst_origin_z,
                        dst_width,
                        dst_height,
                        dst_depth_or_len,
                        buffer_manager,
                        using_ubos,
                    )?,
                    _ => {
                        warn!("incompatible texture data and internal format, skipped");
                    }
                }
            }
        }

        let using_gl_texture = using_textures
            .get(&(activating_texture_unit, item.layout.as_layout()))
            .map(|(t, _)| t);
        self.gl.bind_texture(layout.to_gl_enum(), using_gl_texture);

        Ok(item.clone())
    }

    fn listen_texturing_dropped(&self, texturing: &Texturing) {
        let id = *texturing.id();
        let mut rx = texturing.receiver();
        let mut abortion = self.abortion.subscribe();
        let textures = Rc::clone(&self.textures);
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let result = select! {
                    _ = abortion.recv() => break,
                    result = rx.recv() => result
                };

                match result {
                    Ok(msg) => match msg {
                        TexturingMessage::Dropped => {
                            textures.borrow_mut().remove(&id);
                        }
                        #[allow(unreachable_patterns)]
                        _ => {}
                    },
                    Err(err) => match err {
                        RecvError::Closed => break,
                        RecvError::Lagged(_) => continue,
                    },
                }
            }
        });
    }
}

impl Drop for WebGlTextureManager {
    fn drop(&mut self) {
        let _ = self.abortion.send(());
    }
}
