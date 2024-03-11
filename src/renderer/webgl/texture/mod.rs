//! Texture is created in Immutable Storage using `texStorage2D`
//! and then image data are uploaded by `texSubImage2D`.
//! Once the texture is created, the memory layout is no more alterable,
//! meaning that `texImage2D` and `texStorage2D` are no longer work.
//! But developer could still modify image data using `texSubImage2D`.
//! You have to create a new texture if you want to allocate a new texture with different layout.
//!

pub mod texture2d;
pub mod texture2darray;
pub mod texture3d;
pub mod texture_cubemap;

use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    hash::Hash,
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use uuid::Uuid;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
        Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    ExtTextureFilterAnisotropic, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement,
    ImageBitmap, ImageData, WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use crate::{
    lru::{Lru, LruNode},
    share::{Share, WeakShare},
};

use super::{
    capabilities::{Capabilities, EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC},
    conversion::ToGlEnum,
    error::Error,
    params::GetWebGlParameters,
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

/// Available texture upload targets for `texImage2d`, `texImage3d`, `texSubImage2d` and ``texSubImage3d`` mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUploadTarget {
    TEXTURE_2D,
    TEXTURE_CUBE_MAP_POSITIVE_X,
    TEXTURE_CUBE_MAP_NEGATIVE_X,
    TEXTURE_CUBE_MAP_POSITIVE_Y,
    TEXTURE_CUBE_MAP_NEGATIVE_Y,
    TEXTURE_CUBE_MAP_POSITIVE_Z,
    TEXTURE_CUBE_MAP_NEGATIVE_Z,
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
    pub fn unit_index(&self) -> usize {
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

/// Available texture data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDataType {
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

/// Available texture formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
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
pub enum TextureColorFormat {
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
}

impl TextureColorFormat {
    /// Calculates the bytes length of of a specified internal format in specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureColorFormat::RGBA32I => width * height * 16,
            TextureColorFormat::RGBA32UI => width * height * 16,
            TextureColorFormat::RGBA16I => width * height * 4,
            TextureColorFormat::RGBA16UI => width * height * 4,
            TextureColorFormat::RGBA8 => width * height * 4,
            TextureColorFormat::RGBA8I => width * height * 4,
            TextureColorFormat::RGBA8UI => width * height * 4,
            TextureColorFormat::SRGB8_ALPHA8 => width * height * 4,
            TextureColorFormat::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureColorFormat::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureColorFormat::RGBA4 => width * height * 2,
            TextureColorFormat::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
            TextureColorFormat::RGB8 => width * height * 3,
            TextureColorFormat::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
            TextureColorFormat::RG32I => width * height * 4,
            TextureColorFormat::RG32UI => width * height * 4,
            TextureColorFormat::RG16I => width * height * 4,
            TextureColorFormat::RG16UI => width * height * 4,
            TextureColorFormat::RG8 => width * height * 2,
            TextureColorFormat::RG8I => width * height * 2,
            TextureColorFormat::RG8UI => width * height * 2,
            TextureColorFormat::R32I => width * height * 4,
            TextureColorFormat::R32UI => width * height * 4,
            TextureColorFormat::R16I => width * height * 2,
            TextureColorFormat::R16UI => width * height * 2,
            TextureColorFormat::R8 => width * height * 1,
            TextureColorFormat::R8I => width * height * 1,
            TextureColorFormat::R8UI => width * height * 1,
            TextureColorFormat::RGBA32F => width * height * 16,
            TextureColorFormat::RGBA16F => width * height * 4,
            TextureColorFormat::RGBA8_SNORM => width * height * 4,
            TextureColorFormat::RGB32F => width * height * 12,
            TextureColorFormat::RGB32I => width * height * 12,
            TextureColorFormat::RGB32UI => width * height * 12,
            TextureColorFormat::RGB16F => width * height * 6,
            TextureColorFormat::RGB16I => width * height * 6,
            TextureColorFormat::RGB16UI => width * height * 6,
            TextureColorFormat::RGB8_SNORM => width * height * 3,
            TextureColorFormat::RGB8I => width * height * 3,
            TextureColorFormat::RGB8UI => width * height * 3,
            TextureColorFormat::SRGB8 => width * height * 3,
            TextureColorFormat::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
            TextureColorFormat::RGB9_E5 => width * height * 4,        // 9 + 9 + 9 + 5 in bits
            TextureColorFormat::RG32F => width * height * 4,
            TextureColorFormat::RG16F => width * height * 4,
            TextureColorFormat::RG8_SNORM => width * height * 2,
            TextureColorFormat::R32F => width * height * 4,
            TextureColorFormat::R16F => width * height * 2,
            TextureColorFormat::R8_SNORM => width * height * 1,
        }
    }
}

/// Available texture depth internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDepthFormat {
    DEPTH_COMPONENT32F,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT16,
    DEPTH32F_STENCIL8,
    DEPTH24_STENCIL8,
}

impl TextureDepthFormat {
    /// Calculates the bytes length of of a specified internal format in specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureDepthFormat::DEPTH_COMPONENT32F => width * height * 4,
            TextureDepthFormat::DEPTH_COMPONENT24 => width * height * 3,
            TextureDepthFormat::DEPTH_COMPONENT16 => width * height * 2,
            TextureDepthFormat::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
            TextureDepthFormat::DEPTH24_STENCIL8 => width * height * 4,
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
    /// Calculates the bytes length of of a specified internal format in specified size.
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
    fn save(&self, gl: &WebGl2RenderingContext) -> Option<TexturePixelStorage> {
        match self {
            TexturePixelStorage::PACK_ALIGNMENT(_) => gl
                .texture_pixel_storage_pack_alignment()
                .map(|v| TexturePixelStorage::PACK_ALIGNMENT(v)),
            TexturePixelStorage::PACK_ROW_LENGTH(_) => gl
                .texture_pixel_storage_pack_row_length()
                .map(|v| TexturePixelStorage::PACK_ROW_LENGTH(v)),
            TexturePixelStorage::PACK_SKIP_PIXELS(_) => gl
                .texture_pixel_storage_pack_skip_pixels()
                .map(|v| TexturePixelStorage::PACK_SKIP_PIXELS(v)),
            TexturePixelStorage::PACK_SKIP_ROWS(_) => gl
                .texture_pixel_storage_pack_skip_rows()
                .map(|v| TexturePixelStorage::PACK_SKIP_ROWS(v)),
            TexturePixelStorage::UNPACK_ALIGNMENT(_) => gl
                .texture_pixel_storage_unpack_alignment()
                .map(|v| TexturePixelStorage::UNPACK_ALIGNMENT(v)),
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(_) => gl
                .texture_pixel_storage_unpack_flip_y()
                .map(|v| TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v)),
            TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(_) => gl
                .texture_pixel_storage_unpack_premultiply_alpha()
                .map(|v| TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v)),
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(_) => gl
                .texture_pixel_storage_unpack_colorspace_conversion()
                .map(|v| TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v)),
            TexturePixelStorage::UNPACK_ROW_LENGTH(_) => gl
                .texture_pixel_storage_unpack_row_length()
                .map(|v| TexturePixelStorage::UNPACK_ROW_LENGTH(v)),
            TexturePixelStorage::UNPACK_IMAGE_HEIGHT(_) => gl
                .texture_pixel_storage_unpack_image_height()
                .map(|v| TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v)),
            TexturePixelStorage::UNPACK_SKIP_PIXELS(_) => gl
                .texture_pixel_storage_unpack_skip_pixels()
                .map(|v| TexturePixelStorage::UNPACK_SKIP_PIXELS(v)),
            TexturePixelStorage::UNPACK_SKIP_ROWS(_) => gl
                .texture_pixel_storage_unpack_skip_rows()
                .map(|v| TexturePixelStorage::UNPACK_SKIP_ROWS(v)),
            TexturePixelStorage::UNPACK_SKIP_IMAGES(_) => gl
                .texture_pixel_storage_unpack_skip_images()
                .map(|v| TexturePixelStorage::UNPACK_SKIP_IMAGES(v)),
        }
    }

    fn pixel_store(&self, gl: &WebGl2RenderingContext) {
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
    fn tex_parameter(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        capabilities: &Capabilities,
    ) -> Result<(), Error> {
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
                    return Err(Error::ExtensionUnsupported(
                        EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC,
                    ));
                }
                gl.tex_parameterf(
                    target.gl_enum(),
                    ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT,
                    *v,
                );
            }
        };
        Ok(())
    }
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
    fn sampler_parameter(&self, gl: &WebGl2RenderingContext, sampler: &WebGlSampler) {
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

/// WebGL internal formats of a texture, including [`TextureColorFormat`], [`TextureDepthFormat`] and [`TextureCompressedFormat`].
/// Different internal formats have different memory layout in GPU.
pub trait TextureInternalFormat: ToGlEnum + Copy {
    /// Calculates the bytes length of of a specified internal format in specified size.
    fn byte_length(&self, width: usize, height: usize) -> usize;

    /// Checks capabilities of the internal format.
    fn capabilities(&self, capabilities: &Capabilities) -> Result<(), Error>;
}

impl TextureInternalFormat for TextureColorFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
        self.byte_length(width, height)
    }

    fn capabilities(&self, capabilities: &Capabilities) -> Result<(), Error> {
        capabilities.verify_internal_format_uncompressed(*self)?;
        Ok(())
    }
}

impl TextureInternalFormat for TextureDepthFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
        self.byte_length(width, height)
    }

    fn capabilities(&self, _: &Capabilities) -> Result<(), Error> {
        Ok(())
    }
}

impl TextureInternalFormat for TextureCompressedFormat {
    fn byte_length(&self, width: usize, height: usize) -> usize {
        self.byte_length(width, height)
    }

    fn capabilities(&self, capabilities: &Capabilities) -> Result<(), Error> {
        capabilities.verify_internal_format_compressed(*self)?;
        Ok(())
    }
}

macro_rules! texture_sources {
    ($(
        (
            $(($html_name:ident, $html_type:ident, $html_width:ident, $html_height:ident, $tex_2d_func:ident, $tex_3d_func:ident))+
        ),
        (
            $(($buffer_name:ident, $buffer_type:ident, $buffer_targe:expr))+
        )
    )+) => {
        /// Uncompressed data source for uploading data to [`WebGlTexture`].
        ///
        /// Width and height in a texture source must be the exactly size of the image, not the size to be uploaded to the texture.
        ///
        /// Provides custom [`TexturePixelStorage`]s to tell WebGL how to unpack the data.
        /// For image data from Web API, some pixel storages configurations are ignored,
        /// checks https://registry.khronos.org/webgl/specs/latest/2.0/#5.35 for more details.
        pub enum TextureSource {
            Function {
                width: usize,
                height: usize,
                callback: Share<dyn Fn() -> TextureSource>,
            },
            PixelBufferObject {
                width: usize,
                height: usize,
                buffer: WebGlBuffer,
                format: TextureFormat,
                data_type: TextureDataType,
                pbo_offset: usize,
                pixel_storages: Vec<TexturePixelStorage>,
            },
            Binary {
                width: usize,
                height: usize,
                data: Box<dyn AsRef<[u8]>>,
                format: TextureFormat,
                data_type: TextureDataType,
                src_offset: usize,
                pixel_storages: Vec<TexturePixelStorage>,
            },
            $(
                $(
                    $html_name {
                        data: $html_type,
                        format: TextureFormat,
                        data_type: TextureDataType,
                        pixel_storages: Vec<TexturePixelStorage>,

                    },
                )+
            )+
            Uint16Array {
                width: usize,
                height: usize,
                data: Uint16Array,
                format: TextureFormat,
                /// Only [`TextureDataType::UNSIGNED_SHORT`],
                /// [`TextureDataType::UNSIGNED_SHORT_5_6_5`],
                /// [`TextureDataType::UNSIGNED_SHORT_4_4_4_4`],
                /// [`TextureDataType::UNSIGNED_SHORT_5_5_5_1`],
                /// [`TextureDataType::HALF_FLOAT`] are accepted.
                data_type: TextureDataType,
                src_offset: usize,
                pixel_storages: Vec<TexturePixelStorage>,
            },
            Uint32Array {
                width: usize,
                height: usize,
                data: Uint32Array,
                format: TextureFormat,
                /// Only [`TextureDataType::UNSIGNED_INT`],
                /// [`TextureDataType::UNSIGNED_INT_24_8`]
                /// are accepted.
                data_type: TextureDataType,
                src_offset: usize,
                pixel_storages: Vec<TexturePixelStorage>,
            },
            $(

                $(
                    $buffer_name {
                        width: usize,
                        height: usize,
                        data: $buffer_type,
                        format: TextureFormat,
                        src_offset: usize,
                        pixel_storages: Vec<TexturePixelStorage>,
                    },
                )+
            )+
        }

        impl TextureSource {
            fn pixel_storages(&self, gl: &WebGl2RenderingContext) -> Option<Vec<TexturePixelStorage>> {
                match self {
                    TextureSource::PixelBufferObject { pixel_storages, .. }
                    | TextureSource::Binary { pixel_storages, .. }
                    | TextureSource::Uint16Array { pixel_storages, .. }
                    | TextureSource::Uint32Array { pixel_storages, .. }
                    $(
                        $(
                            | TextureSource::$html_name { pixel_storages, .. }
                        )+
                        $(
                            | TextureSource::$buffer_name { pixel_storages, .. }
                        )+
                    )+
                    => {
                        let mut bounds = Vec::with_capacity(pixel_storages.len());
                        pixel_storages
                            .iter()
                            .for_each(|p| {
                                if let Some(bound) = p.save(gl) {
                                    bounds.push(bound);
                                }
                                p.pixel_store(gl);
                            });
                        Some(bounds)
                    }
                    TextureSource::Function { .. } => None
                }
            }

            /// Returns the width of the texture source.
            fn width(&self) -> usize {
                match self {
                    TextureSource::Function { width, .. } => *width,
                    TextureSource::PixelBufferObject { width, .. } => *width,
                    TextureSource::Binary { width, .. } => *width,
                    TextureSource::Uint16Array { width, .. } => *width,
                    TextureSource::Uint32Array { width, .. } => *width,
                    $(
                        $(
                            TextureSource::$html_name { data, .. } => data.$html_width() as usize,
                        )+
                        $(
                            TextureSource::$buffer_name { width, .. } => *width,
                        )+
                    )+
                }
            }

            /// Returns the height of the texture source.
            fn height(&self) -> usize {
                match self {
                    TextureSource::Function { height, .. } => *height,
                    TextureSource::PixelBufferObject { height, .. } => *height,
                    TextureSource::Binary { height, .. } => *height,
                    TextureSource::Uint16Array { height, .. } => *height,
                    TextureSource::Uint32Array { height, .. } => *height,
                    $(
                        $(
                            TextureSource::$html_name { data, .. } => data.$html_height() as usize,
                        )+
                        $(
                            TextureSource::$buffer_name { height, .. } => *height,
                        )+
                    )+
                }
            }

            fn tex_sub_image_2d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureUploadTarget,
                level: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
            ) -> Result<(), Error> {
                // sets pixel storages and saves current binding states.
                let bounds = self.pixel_storages(gl);

                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);

                // buffers image sub data
                let result = match self {
                    TextureSource::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSource::Function { .. } = source {
                            return Err(Error::TextureSourceFunctionRecursionDisallowed);
                        }
                        if self.width() != source.width() || self.height() != source.height() {
                            return Err(Error::TextureSourceFunctionSizeMismatched);
                        }
                        source.tex_sub_image_2d(
                            gl,
                            target,
                            level,
                            Some(width),
                            Some(height),
                            Some(x_offset),
                            Some(y_offset),
                        )?;
                        Ok(())
                    }
                    TextureSource::PixelBufferObject {
                        buffer,
                        format,
                        data_type,
                        pbo_offset,
                        ..
                    } => {
                        let binding = gl.pixel_unpack_buffer_binding();
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        let result = gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            width as i32,
                            height as i32,
                            format.gl_enum(),
                            data_type.gl_enum(),
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            binding.as_ref(),
                        );
                        result
                    }
                    TextureSource::Binary {
                        data,
                        format,
                        data_type,
                        src_offset,
                        ..
                    } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        width as i32,
                        height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        data.as_ref().as_ref(),
                        *src_offset as u32,
                    ),
                    $(
                        $(
                            TextureSource::$html_name {
                                format,
                                data_type,
                                data,
                                ..
                            } => gl.$tex_2d_func(
                                target.gl_enum(),
                                level as i32,
                                x_offset as i32,
                                y_offset as i32,
                                width as i32,
                                height as i32,
                                format.gl_enum(),
                                data_type.gl_enum(),
                                data,
                            ),
                        )+
                    )+
                    TextureSource::Uint16Array { .. }
                    | TextureSource::Uint32Array { .. }
                    $(
                        $(
                            | TextureSource::$buffer_name { .. }
                        )+
                    )+
                    => {
                        let (data, format, data_type, src_offset) = match self {
                            TextureSource::Uint16Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            TextureSource::Uint32Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            $(
                                $(
                                    TextureSource::$buffer_name { data, format, src_offset, .. } => (data as &Object, format, $buffer_targe, src_offset),
                                )+

                            )+
                            _ => unreachable!(),
                        };
                        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            width as i32,
                            height as i32,
                            format.gl_enum(),
                            data_type,
                            data,
                            *src_offset  as u32
                        )
                    }
                };

                // restores
                if let Some(bounds) = bounds {
                    bounds.into_iter().for_each(|p| p.pixel_store(gl));
                }

                result.map_err(|err| Error::TexImageFailure(err.as_string()))
            }

            fn tex_sub_image_3d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureUploadTarget,
                level: usize,
                depth: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
                z_offset: Option<usize>,
            ) -> Result<(), Error> {
                // sets pixel storages and saves current binding states.
                let bounds = self.pixel_storages(gl);

                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);
                let z_offset = z_offset.unwrap_or(0);

                // buffers image sub data
                let result = match self {
                    TextureSource::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSource::Function { .. } = source {
                            return Err(Error::TextureSourceFunctionRecursionDisallowed);
                        }
                        if self.width() != source.width() || self.height() != source.height() {
                            return Err(Error::TextureSourceFunctionSizeMismatched);
                        }
                        source.tex_sub_image_3d(gl, target, level, depth, Some(width), Some(height), Some(x_offset), Some(y_offset), Some(z_offset))?;
                        Ok(())
                    }
                    TextureSource::PixelBufferObject {
                        buffer,
                        format,
                        data_type,
                        pbo_offset,
                        ..
                    } => {
                        let binding = gl.pixel_unpack_buffer_binding();
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        let result = gl.tex_sub_image_3d_with_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            z_offset as i32,
                            width as i32,
                            height as i32,
                            depth as i32,
                            format.gl_enum(),
                            data_type.gl_enum(),
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            binding.as_ref(),
                        );
                        result
                    }
                    TextureSource::Binary {
                        data,
                        format,
                        data_type,
                        src_offset,
                        ..
                    } => gl.tex_sub_image_3d_with_opt_u8_array_and_src_offset(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        width as i32,
                        height as i32,
                        depth as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        Some(data.as_ref().as_ref()),
                        *src_offset as u32,
                    ),
                    $(
                        $(
                            TextureSource::$html_name {
                                format,
                                data_type,
                                data,
                                ..
                            } => gl.$tex_3d_func(
                                target.gl_enum(),
                                level as i32,
                                x_offset as i32,
                                y_offset as i32,
                                z_offset as i32,
                                width as i32,
                                height as i32,
                                depth as i32,
                                format.gl_enum(),
                                data_type.gl_enum(),
                                data,
                            ),
                        )+
                    )+
                    TextureSource::Uint16Array { .. }
                    | TextureSource::Uint32Array { .. }
                    $(
                        $(
                            | TextureSource::$buffer_name { .. }
                        )+
                    )+
                    => {
                        let (data, format, data_type, src_offset) = match self {
                            TextureSource::Uint16Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            TextureSource::Uint32Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            $(
                                $(
                                    TextureSource::$buffer_name { data, format, src_offset, .. } => (data as &Object, format, $buffer_targe, src_offset),
                                )+

                            )+
                            _ => unreachable!(),
                        };
                        gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            z_offset as i32,
                            width as i32,
                            height as i32,
                            depth as i32,
                            format.gl_enum(),
                            data_type,
                            Some(data),
                            *src_offset as u32,
                        )
                    }
                };

                // restores
                if let Some(bounds) = bounds {
                    bounds.into_iter().for_each(|p| p.pixel_store(gl));
                }

                result.map_err(|err| Error::TexImageFailure(err.as_string()))
            }
        }
    }
}

texture_sources! {
    (
        (HtmlCanvasElement, HtmlCanvasElement, width, height, tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element, tex_sub_image_3d_with_html_canvas_element)
        (HtmlImageElement, HtmlImageElement, natural_width, natural_height, tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element, tex_sub_image_3d_with_html_image_element)
        (HtmlVideoElement, HtmlVideoElement, video_width, video_height, tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element, tex_sub_image_3d_with_html_video_element)
        (ImageData, ImageData, width, height, tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data, tex_sub_image_3d_with_image_data)
        (ImageBitmap, ImageBitmap, width, height, tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap, tex_sub_image_3d_with_image_bitmap)
    ),
    (
        (Int8Array, Int8Array, WebGl2RenderingContext::BYTE)
        (Uint8Array, Uint8Array, WebGl2RenderingContext::UNSIGNED_BYTE)
        (Uint8ClampedArray, Uint8ClampedArray, WebGl2RenderingContext::UNSIGNED_BYTE)
        (Int16Array, Int16Array, WebGl2RenderingContext::SHORT)
        (Int32Array, Int32Array, WebGl2RenderingContext::INT)
        (Float32Array, Float32Array, WebGl2RenderingContext::FLOAT)
    )
}

macro_rules! texture_sources_compressed {
    ($(($name:ident, $data_view:ident))+) => {
        /// Compressed data source for uploading data to [`WebGlTexture`].
        ///
        /// Width and height in a texture source must be the exactly size of the image, not the size to be uploaded to the texture.
        ///
        /// [`TexturePixelStorage`]s are not available on compressed format.
        pub enum TextureSourceCompressed {
            Function {
                width: usize,
                height: usize,
                callback: Share<dyn Fn() -> TextureSourceCompressed>,
            },
            PixelBufferObject {
                width: usize,
                height: usize,
                compressed_format: TextureCompressedFormat,
                buffer: WebGlBuffer,
                image_size: usize,
                pbo_offset: usize,
            },
            $(
                $name {
                    width: usize,
                    height: usize,
                    compressed_format: TextureCompressedFormat,
                    data: $data_view,
                    src_offset: usize,
                    src_length_override: Option<usize>,
                },
            )+
        }

        impl TextureSourceCompressed {
            /// Returns the width of the texture source.
            fn width(&self) -> usize {
                match self {
                    TextureSourceCompressed::Function { width, .. }
                    | TextureSourceCompressed::PixelBufferObject { width, .. }
                    $(
                        | TextureSourceCompressed::$name { width, .. }
                    )+
                    => *width,
                }
            }

            /// Returns the height of the texture source.
            fn height(&self) -> usize {
                match self {
                    TextureSourceCompressed::Function { height, .. }
                    | TextureSourceCompressed::PixelBufferObject { height, .. }
                    $(
                        | TextureSourceCompressed::$name { height, .. }
                    )+
                    => *height,
                }
            }

            fn tex_sub_image_2d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureUploadTarget,
                level: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
            ) -> Result<(), Error> {
                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);

                // buffers image sub data
                match self {
                    TextureSourceCompressed::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceCompressed::Function { .. } = source {
                            return Err(Error::TextureSourceFunctionRecursionDisallowed);
                        }
                        if self.width() != source.width() || self.height() != source.height() {
                            return Err(Error::TextureSourceFunctionSizeMismatched);
                        }
                        source.tex_sub_image_2d(
                            gl,
                            target,
                            level,
                            Some(width),
                            Some(height),
                            Some(x_offset),
                            Some(y_offset),
                        )?;
                        Ok(())
                    }
                    TextureSourceCompressed::PixelBufferObject {
                        compressed_format,
                        buffer,
                        image_size,
                        pbo_offset,
                        ..
                    } => {
                        let binding = gl.pixel_unpack_buffer_binding();
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        gl.compressed_tex_sub_image_2d_with_i32_and_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            width as i32,
                            height as i32,
                            compressed_format.gl_enum(),
                            *image_size as i32,
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            binding.as_ref(),
                        );
                        Ok(())
                    }
                    $(
                        TextureSourceCompressed::$name { .. }
                    ) | + => {
                        let (width, height, compressed_format, data, src_offset, src_length_override) = match self {
                            $(
                                TextureSourceCompressed::$name {
                                    compressed_format,
                                    data,
                                    src_offset,
                                    src_length_override,
                                    ..
                                } => (
                                    width,
                                    height,
                                    compressed_format,
                                    data as &Object,
                                    src_offset,
                                    src_length_override,
                                ),
                            )+
                            _ => unreachable!(),
                        };
                        gl.compressed_tex_sub_image_2d_with_array_buffer_view_and_u32_and_src_length_override(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            width as i32,
                            height as i32,
                            compressed_format.gl_enum(),
                            data,
                            *src_offset as u32,
                            src_length_override.unwrap_or(0) as u32,
                        );
                        Ok(())
                    }
                }
            }

            fn tex_sub_image_3d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureUploadTarget,
                level: usize,
                depth: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
                z_offset: Option<usize>,
            ) -> Result<(), Error> {
                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);
                let z_offset = z_offset.unwrap_or(0);

                // buffers image sub data
                match self {
                    TextureSourceCompressed::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceCompressed::Function { .. } = source {
                            return Err(Error::TextureSourceFunctionRecursionDisallowed);
                        }
                        if self.width() != source.width() || self.height() != source.height() {
                            return Err(Error::TextureSourceFunctionSizeMismatched);
                        }
                        source.tex_sub_image_3d(
                            gl,
                            target,
                            level,
                            depth,
                            Some(width),
                            Some(height),
                            Some(x_offset),
                            Some(y_offset),
                            Some(z_offset),
                        )?;
                        Ok(())
                    }
                    TextureSourceCompressed::PixelBufferObject {
                        compressed_format,
                        buffer,
                        image_size,
                        pbo_offset,
                        ..
                    } => {
                        let binding = gl.pixel_unpack_buffer_binding();
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        gl.compressed_tex_sub_image_3d_with_i32_and_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            z_offset as i32,
                            width as i32,
                            height as i32,
                            depth as i32,
                            compressed_format.gl_enum(),
                            *image_size as i32,
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            binding.as_ref(),
                        );
                        Ok(())
                    }
                    $(
                        TextureSourceCompressed::$name { .. }
                    ) | + => {
                        let (width, height, compressed_format, data, src_offset, src_length_override) = match self {
                            $(
                                TextureSourceCompressed::$name {
                                    compressed_format,
                                    data,
                                    src_offset,
                                    src_length_override,
                                    ..
                                } => (
                                    width,
                                    height,
                                    compressed_format,
                                    data as &Object,
                                    src_offset,
                                    src_length_override,
                                ),
                            )+
                            _ => unreachable!(),
                        };
                        gl.compressed_tex_sub_image_3d_with_array_buffer_view_and_u32_and_src_length_override(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            z_offset as i32,
                            width as i32,
                            height as i32,
                            depth as i32,
                            compressed_format.gl_enum(),
                            data,
                            *src_offset as u32,
                            src_length_override.unwrap_or(0) as u32,
                        );
                        Ok(())
                    }
                }
            }
        }
    };
}

texture_sources_compressed! {
    (Int8Array, Int8Array)
    (Uint8Array, Uint8Array)
    (Uint8ClampedArray, Uint8ClampedArray)
    (Int16Array, Int16Array)
    (Uint16Array, Uint16Array)
    (Int32Array, Int32Array)
    (Uint32Array, Uint32Array)
    (Float32Array, Float32Array)
    (Float64Array, Float64Array)
    (BigInt64Array, BigInt64Array)
    (BigUint64Array, BigUint64Array)
    (DataView, DataView)
}

/// Abstract texture trait for creating a [`WebGlTexture`] using [`TextureSource`].
pub trait Texture {
    /// Returns [`TextureTarget`].
    fn target(&self) -> TextureTarget;

    /// Returns a list of [`SamplerParameter`]s.
    fn sampler_parameters(&self) -> &HashMap<u32, SamplerParameter>;

    /// Returns a list of [`TextureParameter`]s.
    fn texture_parameters(&self) -> &HashMap<u32, TextureParameter>;

    /// Calculates max available mipmap level under a specified size in Rounding Down mode.
    fn max_available_mipmap_level(&self) -> usize;

    /// Returns max mipmap level.
    fn max_mipmap_level(&self) -> usize;

    /// Returns bytes length of the whole texture in all levels.
    fn byte_length(&self) -> usize;

    /// Returns bytes length of a mipmap level.
    fn byte_length_of_level(&self, level: usize) -> Option<usize>;
}

/// Abstract texture trait indicates that the texture has width and height dimensions.
pub trait TexturePlanar: Texture {
    /// Calculates max available mipmap level under a specified size in Rounding Down mode.
    fn max_available_mipmap_level(width: usize, height: usize) -> usize {
        (width as f64).max(height as f64).max(1.0).log2().floor() as usize
    }

    /// Returns texture base width in level 0.
    fn width(&self) -> usize;

    /// Returns texture base height in level 0.
    fn height(&self) -> usize;

    /// Returns width of a mipmap level.
    /// Returns texture base width in level 0.
    fn width_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.width());
        }
        if level > self.max_mipmap_level() {
            return None;
        }

        Some((self.width() >> level).max(1))
    }

    /// Returns height of a mipmap level.
    /// Returns texture base height in level 0.
    fn height_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.height());
        }
        if level > self.max_mipmap_level() {
            return None;
        }

        Some((self.height() >> level).max(1))
    }
}

/// Abstract texture trait indicates that the texture has depth dimension.
pub trait TextureDepth: Texture {
    /// Calculates max available mipmap level under a specified size in Rounding Down mode.
    fn max_available_mipmap_level(width: usize, height: usize, depth: usize) -> usize {
        (width as f64)
            .max(height as f64)
            .max(depth as f64)
            .max(1.0)
            .log2()
            .floor() as usize
    }

    /// Returns texture base depth in level 0.
    fn depth(&self) -> usize;

    /// Returns depth of a mipmap level.
    /// Returns texture base depth in level 0.
    fn depth_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.depth());
        }
        if level > self.max_mipmap_level() {
            return None;
        }

        Some((self.depth() >> level).max(1))
    }
}

/// Abstract texture trait indicates that this texture has a list containing multiple textures.
pub trait TextureArray: Texture {
    /// Returns the number of array of this texture.
    fn array_length(&self) -> usize;
}

/// Private abstract texture trait for internal use.
trait TextureItem: Texture {
    /// Returns [`Runtime`].
    fn runtime(&self) -> Option<&Runtime>;

    /// Returns [`Runtime`] without existence checking.
    fn runtime_unchecked(&self) -> &Runtime;

    /// Returns mutable [`Runtime`].
    fn runtime_mut(&mut self) -> Option<&mut Runtime>;

    /// Returns mutable [`Runtime`] without existence checking.
    fn runtime_mut_unchecked(&mut self) -> &mut Runtime;

    /// Sets [`Runtime`].
    fn set_runtime(&mut self, runtime: Runtime);

    /// Removes [`Runtime`].
    fn remove_runtime(&mut self) -> Option<Runtime>;

    /// Validates.
    fn validate(&self, capabilities: &Capabilities) -> Result<(), Error>;

    /// Creates and returns a [`WebGlTexture`].
    fn create_texture(&self, gl: &WebGl2RenderingContext) -> Result<WebGlTexture, Error>;

    /// Uploads data to [`WebGlTexture`].
    /// In this stage, [`TextureItem::runtime`] is created and texture unit is bound already,
    /// it's safe to unwrap it and use fields inside and no need to active texture unit again.
    fn upload(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error>;

    /// Applies memory free behavior.
    /// Returns `true` if this texture is released.
    /// In this stage, [`Texture::runtime`] is created already, it's safe to unwrap it and use fields inside.
    fn free(&mut self) -> bool;
}

/// Available texture sources for uploading, including [`TextureSource`] and [`TextureSourceCompressed`].
enum UploadSource {
    Uncompressed(TextureSource),
    Compressed(TextureSourceCompressed),
}

/// Configurations specify a [`TextureSource`] to upload and
/// a target sub-rectangle in a mipmap level to replace with in the texture.
///
/// Parameters are optional, default values of unspecified parameters as below:
/// - `level`: 0
/// - `depth`: 0
/// - `width`: width of the texture source
/// - `height`: height of the texture source
/// - `x_offset`: 0
/// - `y_offset`: 0
/// - `z_offset`: 0
struct UploadItem {
    source: UploadSource,
    level: Option<usize>,
    depth: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    x_offset: Option<usize>,
    y_offset: Option<usize>,
    z_offset: Option<usize>,
}

impl UploadItem {
    /// Constructs a new upload data to upload to texture with a specified [`TextureSource`].
    fn new(source: TextureSource) -> Self {
        Self {
            source: UploadSource::Uncompressed(source),
            level: None,
            depth: None,
            width: None,
            height: None,
            x_offset: None,
            y_offset: None,
            z_offset: None,
        }
    }

    /// Constructs a new upload data to upload to texture with customize parameters and a specified [`TextureSource`].
    fn with_params(
        source: TextureSource,
        level: Option<usize>,
        depth: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Self {
        Self {
            source: UploadSource::Uncompressed(source),
            level,
            depth,
            width,
            height,
            x_offset,
            y_offset,
            z_offset,
        }
    }

    /// Constructs a new upload data to upload to texture with a specified [`TextureSourceCompressed`].
    fn new_compressed(source: TextureSourceCompressed) -> Self {
        Self {
            source: UploadSource::Compressed(source),
            level: None,
            depth: None,
            width: None,
            height: None,
            x_offset: None,
            y_offset: None,
            z_offset: None,
        }
    }

    /// Constructs a new upload data to upload to texture with customize parameters and a specified [`TextureSourceCompressed`].
    fn with_params_compressed(
        source: TextureSourceCompressed,
        level: Option<usize>,
        depth: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Self {
        Self {
            source: UploadSource::Compressed(source),
            level,
            depth,
            width,
            height,
            x_offset,
            y_offset,
            z_offset,
        }
    }

    /// Uploads texture source to WebGL.
    fn tex_sub_image_2d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureUploadTarget,
    ) -> Result<(), Error> {
        match &self.source {
            UploadSource::Uncompressed(source) => source.tex_sub_image_2d(
                gl,
                target,
                self.level.unwrap_or(0),
                self.width,
                self.height,
                self.x_offset,
                self.y_offset,
            ),
            UploadSource::Compressed(source) => source.tex_sub_image_2d(
                gl,
                target,
                self.level.unwrap_or(0),
                self.width,
                self.height,
                self.x_offset,
                self.y_offset,
            ),
        }
    }

    /// Uploads texture source to WebGL.
    fn tex_sub_image_3d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureUploadTarget,
    ) -> Result<(), Error> {
        match &self.source {
            UploadSource::Uncompressed(source) => source.tex_sub_image_3d(
                gl,
                target,
                self.level.unwrap_or(0),
                self.depth.unwrap_or(0),
                self.width,
                self.height,
                self.x_offset,
                self.y_offset,
                self.z_offset,
            ),
            UploadSource::Compressed(source) => source.tex_sub_image_3d(
                gl,
                target,
                self.level.unwrap_or(0),
                self.depth.unwrap_or(0),
                self.width,
                self.height,
                self.x_offset,
                self.y_offset,
                self.z_offset,
            ),
        }
    }
}

struct Runtime {
    id: Uuid,
    gl: WebGl2RenderingContext,
    capabilities: Capabilities,
    store_id: Uuid,
    target: TextureTarget,
    byte_length: usize,
    texture: WebGlTexture,
    sampler: WebGlSampler,
    using: HashSet<TextureUnit>,
    lru_node: *mut LruNode<Uuid>,

    used_memory: *mut usize,
    textures: *mut HashMap<Uuid, WeakShare<dyn TextureItem>>,
    lru: *mut Lru<Uuid>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            (*self.textures).remove(&self.id);
            (*self.lru).remove(self.lru_node);
            (*self.used_memory) -= self.byte_length;

            {
                // unbinds
                let unit = self.gl.texture_active_texture_unit();
                for unit in self.using.drain() {
                    self.gl.active_texture(unit.gl_enum());
                    let texture = self.gl.texture_binding(self.target);
                    if let Some(texture) = texture {
                        if texture == self.texture {
                            self.gl.bind_texture(self.target.gl_enum(), None);
                        }
                    }
                }
                self.gl.active_texture(unit);
            }

            self.gl.delete_sampler(Some(&self.sampler));
            self.gl.delete_texture(Some(&self.texture));
        }
    }
}

pub struct TextureDescriptor<T>(Share<T>);

impl<T> Clone for TextureDescriptor<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

#[allow(private_bounds)]
impl<T> TextureDescriptor<T>
where
    T: TextureItem,
{
    /// Constructs a new texture descriptor.
    pub fn new(texture: T) -> Self {
        Self(Rc::new(RefCell::new(texture)))
    }

    /// Returns [`Texture`] associated with this descriptor.
    pub fn texture(&self) -> Ref<'_, T> {
        self.0.borrow()
    }

    /// Returns mutable [`Texture`] associated with this descriptor.
    pub fn texture_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

pub struct TextureStore {
    id: Uuid,
    gl: WebGl2RenderingContext,
    capabilities: Capabilities,
    available_memory: usize,

    used_memory: *mut usize,
    lru: *mut Lru<Uuid>,
    textures: *mut HashMap<Uuid, WeakShare<dyn TextureItem>>,
}

impl TextureStore {
    pub fn new(gl: WebGl2RenderingContext, capabilities: Capabilities) -> Self {
        Self::with_available_memory(gl, capabilities, i32::MAX as usize)
    }

    pub fn with_available_memory(
        gl: WebGl2RenderingContext,
        capabilities: Capabilities,
        available_memory: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            capabilities,
            available_memory,

            used_memory: Box::leak(Box::new(0)),
            lru: Box::leak(Box::new(Lru::new())),
            textures: Box::leak(Box::new(HashMap::new())),
        }
    }

    fn free(&mut self) {
        unsafe {
            if *self.used_memory <= self.available_memory {
                return;
            }
            let mut next_node = (*self.lru).least_recently();
            while *self.used_memory > self.available_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };
                let id = (*current_node).data();
                let Entry::Occupied(occupied) = (*self.textures).entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let t = occupied.get();
                let Some(t) = t.upgrade() else {
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let mut t = t.borrow_mut();
                let runtime = t.runtime().unwrap();
                // skips if using
                if !runtime.using.is_empty() {
                    next_node = (*current_node).more_recently();
                    continue;
                }
                // let texture takes free procedure itself.
                if t.free() {
                    let runtime = occupied
                        .remove()
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .remove_runtime()
                        .unwrap();
                    drop(runtime);
                    // do not cleanup here, Drop impl of Runtime will do it.
                }
                next_node = (*current_node).more_recently();
            }
        }
    }

    #[allow(private_bounds)]
    pub fn bind_texture<T>(
        &mut self,
        descriptor: &TextureDescriptor<T>,
        unit: TextureUnit,
    ) -> Result<WebGlTexture, Error>
    where
        T: TextureItem + 'static,
    {
        unsafe {
            let mut t = descriptor.texture_mut();
            let target = t.target();

            // creates runtime if not exists
            if t.runtime().is_none() {
                t.validate(&self.capabilities)?;

                // saves current binding texture
                let texture = t.create_texture(&self.gl)?;
                let sampler = self
                    .gl
                    .create_sampler()
                    .ok_or_else(|| Error::CreateSamplerFailure)?;

                self.gl.bind_texture(target.gl_enum(), Some(&texture));

                // sets texture parameters
                for (_, p) in t.texture_parameters() {
                    p.tex_parameter(&self.gl, target, &self.capabilities)?;
                }

                // sets sampler parameters
                for (_, p) in t.sampler_parameters() {
                    p.sampler_parameter(&self.gl, &sampler);
                }

                let id = Uuid::new_v4();
                let lru_node = LruNode::new(id);
                let byte_length = t.byte_length();
                (*self.textures).insert(
                    id,
                    Rc::downgrade(&descriptor.0) as WeakShare<dyn TextureItem>,
                );
                (*self.used_memory) += byte_length;
                t.set_runtime(Runtime {
                    id,
                    gl: self.gl.clone(),
                    capabilities: self.capabilities.clone(),
                    store_id: self.id,
                    texture: texture.clone(),
                    sampler,
                    target,
                    byte_length,
                    lru_node,
                    using: HashSet::new(),

                    used_memory: self.used_memory,
                    textures: self.textures,
                    lru: self.lru,
                });
            }

            // checks sharing
            if t.runtime_unchecked().store_id != self.id {
                return Err(Error::TextureSharingDisallowed);
            }

            let texture = t.runtime_unchecked().texture.clone();
            let bound_unit = self.gl.texture_active_texture_unit();

            // binds objects
            self.gl.active_texture(unit.gl_enum());
            self.gl
                .bind_texture(target.gl_enum(), Some(&t.runtime_unchecked().texture));
            self.gl.bind_sampler(
                unit.unit_index() as u32,
                Some(&t.runtime_unchecked().sampler.as_ref()),
            );

            // uploads data
            t.upload(&self.gl)?;

            // restore unit
            self.gl.active_texture(bound_unit);

            // updates status
            (*self.lru).cache(t.runtime_unchecked().lru_node);
            t.runtime_mut_unchecked().using.insert(unit);

            // do memory free
            drop(t);
            self.free();

            Ok(texture)
        }
    }

    #[allow(private_bounds)]
    pub fn unbind_texture<T>(
        &mut self,
        descriptor: &TextureDescriptor<T>,
        unit: TextureUnit,
    ) -> Result<(), Error>
    where
        T: TextureItem + 'static,
    {
        let mut t = descriptor.texture_mut();
        let target = t.target();
        if let Some(runtime) = t.runtime_mut() {
            let bound = self.gl.texture_active_texture_unit();
            self.gl.active_texture(unit.gl_enum());
            self.gl.bind_texture(target.gl_enum(), None);
            self.gl.bind_sampler(unit.unit_index() as u32, None);
            self.gl.active_texture(bound);
            runtime.using.remove(&unit);
        }

        Ok(())
    }
}

impl Drop for TextureStore {
    fn drop(&mut self) {
        unsafe {
            for (_, t) in (*self.textures).iter_mut() {
                let runtime = t.upgrade().and_then(|t| t.borrow_mut().remove_runtime());
                let Some(runtime) = runtime else {
                    continue;
                };
                drop(runtime);
            }
            drop(Box::from_raw(self.textures));
            drop(Box::from_raw(self.used_memory));
            drop(Box::from_raw(self.lru));
        }
    }
}
