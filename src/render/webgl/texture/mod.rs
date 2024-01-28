use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap};
use log::debug;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
        Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlTexture,
};

use crate::lru::{Lru, LruNode};

use super::{
    abilities::Abilities, conversion::ToGlEnum, error::Error, utils::pixel_unpack_buffer_binding,
};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureTarget {
    TEXTURE_2D,
    TEXTURE_CUBE_MAP,
    TEXTURE_CUBE_MAP_POSITIVE_X,
    TEXTURE_CUBE_MAP_POSITIVE_Y,
    TEXTURE_CUBE_MAP_POSITIVE_Z,
    TEXTURE_CUBE_MAP_NEGATIVE_X,
    TEXTURE_CUBE_MAP_NEGATIVE_Y,
    TEXTURE_CUBE_MAP_NEGATIVE_Z,
    TEXTURE_2D_ARRAY,
    TEXTURE_3D,
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

/// Available texture internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureInternalFormat {
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

impl TextureInternalFormat {
    /// Calculates the bytes length of of a specified internal format in specified size.
    pub fn bytes_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureInternalFormat::RGBA32I => width * height * 16,
            TextureInternalFormat::RGBA32UI => width * height * 16,
            TextureInternalFormat::RGBA16I => width * height * 4,
            TextureInternalFormat::RGBA16UI => width * height * 4,
            TextureInternalFormat::RGBA8 => width * height * 4,
            TextureInternalFormat::RGBA8I => width * height * 4,
            TextureInternalFormat::RGBA8UI => width * height * 4,
            TextureInternalFormat::SRGB8_ALPHA8 => width * height * 4,
            TextureInternalFormat::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureInternalFormat::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureInternalFormat::RGBA4 => width * height * 2,
            TextureInternalFormat::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
            TextureInternalFormat::RGB8 => width * height * 3,
            TextureInternalFormat::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
            TextureInternalFormat::RG32I => width * height * 4,
            TextureInternalFormat::RG32UI => width * height * 4,
            TextureInternalFormat::RG16I => width * height * 4,
            TextureInternalFormat::RG16UI => width * height * 4,
            TextureInternalFormat::RG8 => width * height * 2,
            TextureInternalFormat::RG8I => width * height * 2,
            TextureInternalFormat::RG8UI => width * height * 2,
            TextureInternalFormat::R32I => width * height * 4,
            TextureInternalFormat::R32UI => width * height * 4,
            TextureInternalFormat::R16I => width * height * 2,
            TextureInternalFormat::R16UI => width * height * 2,
            TextureInternalFormat::R8 => width * height * 1,
            TextureInternalFormat::R8I => width * height * 1,
            TextureInternalFormat::R8UI => width * height * 1,
            TextureInternalFormat::RGBA32F => width * height * 16,
            TextureInternalFormat::RGBA16F => width * height * 4,
            TextureInternalFormat::RGBA8_SNORM => width * height * 4,
            TextureInternalFormat::RGB32F => width * height * 12,
            TextureInternalFormat::RGB32I => width * height * 12,
            TextureInternalFormat::RGB32UI => width * height * 12,
            TextureInternalFormat::RGB16F => width * height * 6,
            TextureInternalFormat::RGB16I => width * height * 6,
            TextureInternalFormat::RGB16UI => width * height * 6,
            TextureInternalFormat::RGB8_SNORM => width * height * 3,
            TextureInternalFormat::RGB8I => width * height * 3,
            TextureInternalFormat::RGB8UI => width * height * 3,
            TextureInternalFormat::SRGB8 => width * height * 3,
            TextureInternalFormat::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
            TextureInternalFormat::RGB9_E5 => width * height * 4,        // 9 + 9 + 9 + 5 in bits
            TextureInternalFormat::RG32F => width * height * 4,
            TextureInternalFormat::RG16F => width * height * 4,
            TextureInternalFormat::RG8_SNORM => width * height * 2,
            TextureInternalFormat::R32F => width * height * 4,
            TextureInternalFormat::R16F => width * height * 2,
            TextureInternalFormat::R8_SNORM => width * height * 1,
            TextureInternalFormat::DEPTH_COMPONENT32F => width * height * 4,
            TextureInternalFormat::DEPTH_COMPONENT24 => width * height * 3,
            TextureInternalFormat::DEPTH_COMPONENT16 => width * height * 2,
            TextureInternalFormat::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
            TextureInternalFormat::DEPTH24_STENCIL8 => width * height * 4,
        }
    }
}

/// Available texture internal formats mapped from [`WebGl2RenderingContext`].
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
    pub fn bytes_length(&self, width: usize, height: usize) -> usize {
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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackColorSpaceConversion {
    NONE,
    BROWSER_DEFAULT_WEBGL,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePixelStorage {
    PACK_ALIGNMENT(i32),
    UNPACK_ALIGNMENT(i32),
    UNPACK_FLIP_Y_WEBGL(bool),
    UNPACK_PREMULTIPLY_ALPHA_WEBGL(bool),
    UNPACK_COLORSPACE_CONVERSION_WEBGL(TextureUnpackColorSpaceConversion),
    PACK_ROW_LENGTH(i32),
    PACK_SKIP_PIXELS(i32),
    PACK_SKIP_ROWS(i32),
    UNPACK_ROW_LENGTH(i32),
    UNPACK_IMAGE_HEIGHT(i32),
    UNPACK_SKIP_PIXELS(i32),
    UNPACK_SKIP_ROWS(i32),
    UNPACK_SKIP_IMAGES(i32),
}

impl TexturePixelStorage {
    pub fn key(&self) -> u32 {
        self.gl_enum()
    }

    pub fn value(&self) -> i32 {
        match self {
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v)
            | TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v) => v.gl_enum() as i32,
            TexturePixelStorage::PACK_ALIGNMENT(v)
            | TexturePixelStorage::UNPACK_ALIGNMENT(v)
            | TexturePixelStorage::PACK_ROW_LENGTH(v)
            | TexturePixelStorage::PACK_SKIP_PIXELS(v)
            | TexturePixelStorage::PACK_SKIP_ROWS(v)
            | TexturePixelStorage::UNPACK_ROW_LENGTH(v)
            | TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v)
            | TexturePixelStorage::UNPACK_SKIP_PIXELS(v)
            | TexturePixelStorage::UNPACK_SKIP_ROWS(v)
            | TexturePixelStorage::UNPACK_SKIP_IMAGES(v) => *v,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMagnificationFilter {
    LINEAR,
    NEAREST,
}

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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapMethod {
    REPEAT,
    CLAMP_TO_EDGE,
    MIRRORED_REPEAT,
}

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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareMode {
    NONE,
    COMPARE_REF_TO_TEXTURE,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameter {
    MAG_FILTER(TextureMagnificationFilter),
    MIN_FILTER(TextureMinificationFilter),
    WRAP_S(TextureWrapMethod),
    WRAP_T(TextureWrapMethod),
    WRAP_R(TextureWrapMethod),
    BASE_LEVEL(i32),
    COMPARE_FUNC(TextureCompareFunction),
    COMPARE_MODE(TextureCompareMode),
    MAX_LEVEL(i32),
    MAX_LOD(f32),
    MIN_LOD(f32),
}

pub struct Restorer {
    callback: Rc<RefCell<dyn Fn() -> TextureSource>>,
}

pub struct RestorerCompressed {
    callback: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
}

pub struct CubeMapRestorer {
    positive_x: Rc<RefCell<dyn Fn() -> TextureSource>>,
    negative_x: Rc<RefCell<dyn Fn() -> TextureSource>>,
    positive_y: Rc<RefCell<dyn Fn() -> TextureSource>>,
    negative_y: Rc<RefCell<dyn Fn() -> TextureSource>>,
    positive_z: Rc<RefCell<dyn Fn() -> TextureSource>>,
    negative_z: Rc<RefCell<dyn Fn() -> TextureSource>>,
}

pub struct CubeMapRestorerCompressed {
    positive_x: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
    negative_x: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
    positive_y: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
    negative_y: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
    positive_z: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
    negative_z: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
}

/// Memory freeing policies.
pub enum MemoryPolicy<R> {
    Unfree,
    Restorable(R),
}

impl<R> Default for MemoryPolicy<R> {
    fn default() -> Self {
        Self::Unfree
    }
}

pub enum TextureSource {
    Function {
        width: usize,
        height: usize,
        callback: Rc<RefCell<dyn Fn() -> TextureSource>>,
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
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Uint8ClampedArray {
        width: usize,
        height: usize,
        data: Uint8ClampedArray,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Int8Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
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
    Int16Array {
        width: usize,
        height: usize,
        data: Int16Array,
        format: TextureFormat,
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
    Int32Array {
        width: usize,
        height: usize,
        data: Int32Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    HtmlCanvasElement {
        canvas: HtmlCanvasElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    HtmlImageElement {
        image: HtmlImageElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    HtmlVideoElement {
        video: HtmlVideoElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    ImageData {
        data: ImageData,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    ImageBitmap {
        bitmap: ImageBitmap,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
}

macro_rules! texture_source_data_views {
    ($self:ident, $(($name:ident, $data_type:expr))+) => {
        match $self {
            $(
                TextureSource::$name {
                    width,
                    height,
                    data,
                    format,
                    src_offset,
                    ..
                } => (
                    width,
                    height,
                    data as &Object,
                    format,
                    $data_type,
                    src_offset,
                ),
            )+
            TextureSource::Uint16Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => (
                width,
                height,
                data as &Object,
                format,
                data_type.gl_enum(),
                src_offset,
            ),
            TextureSource::Uint32Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => (
                width,
                height,
                data as &Object,
                format,
                data_type.gl_enum(),
                src_offset,
            ),
            _ => unreachable!(),
        }
    };
}

macro_rules! texture_source_size {
    ($self:ident, $dimension:ident, $custom_size:expr, $canvas_size:ident, $image_size:ident, $video_size:ident, $image_data_size:ident) => {
        match $self {
            TextureSource::Function { $dimension, .. }
            | TextureSource::PixelBufferObject { $dimension, .. }
            | TextureSource::Binary { $dimension, .. }
            | TextureSource::Uint8Array { $dimension, .. }
            | TextureSource::Uint8ClampedArray { $dimension, .. }
            | TextureSource::Int8Array { $dimension, .. }
            | TextureSource::Uint16Array { $dimension, .. }
            | TextureSource::Int16Array { $dimension, .. }
            | TextureSource::Uint32Array { $dimension, .. }
            | TextureSource::Int32Array { $dimension, .. }
            | TextureSource::Float32Array { $dimension, .. } => *$dimension,
            TextureSource::HtmlCanvasElement {
                canvas,
                custom_size,
                ..
            } => custom_size
                .map($custom_size)
                .unwrap_or(canvas.$canvas_size() as usize),
            TextureSource::HtmlImageElement {
                image, custom_size, ..
            } => custom_size
                .map($custom_size)
                .unwrap_or(image.$image_size() as usize),
            TextureSource::HtmlVideoElement {
                video, custom_size, ..
            } => custom_size
                .map($custom_size)
                .unwrap_or(video.$video_size() as usize),
            TextureSource::ImageData {
                data, custom_size, ..
            } => custom_size
                .map($custom_size)
                .unwrap_or(data.$image_data_size() as usize),
            TextureSource::ImageBitmap {
                bitmap,
                custom_size,
                ..
            } => custom_size
                .map($custom_size)
                .unwrap_or(bitmap.$image_data_size() as usize),
        }
    };
}

impl TextureSource {
    pub fn width(&self) -> usize {
        texture_source_size! {
            self,
            width,
            |(width, _)| width,
            width,
            natural_width,
            video_width,
            width
        }
    }

    pub fn height(&self) -> usize {
        texture_source_size! {
            self,
            height,
            |(_, height)| height,
            height,
            natural_height,
            video_height,
            height
        }
    }

    fn pixel_storages(&self, gl: &WebGl2RenderingContext) {
        match self {
            TextureSource::PixelBufferObject { pixel_storages, .. }
            | TextureSource::Binary { pixel_storages, .. }
            | TextureSource::Uint8Array { pixel_storages, .. }
            | TextureSource::Uint8ClampedArray { pixel_storages, .. }
            | TextureSource::Int8Array { pixel_storages, .. }
            | TextureSource::Uint16Array { pixel_storages, .. }
            | TextureSource::Int16Array { pixel_storages, .. }
            | TextureSource::Uint32Array { pixel_storages, .. }
            | TextureSource::Int32Array { pixel_storages, .. }
            | TextureSource::Float32Array { pixel_storages, .. }
            | TextureSource::HtmlCanvasElement { pixel_storages, .. }
            | TextureSource::HtmlImageElement { pixel_storages, .. }
            | TextureSource::HtmlVideoElement { pixel_storages, .. }
            | TextureSource::ImageData { pixel_storages, .. }
            | TextureSource::ImageBitmap { pixel_storages, .. } => {
                // setups pixel storage parameters
                pixel_storages
                    .iter()
                    .for_each(|param| gl.pixel_storei(param.key(), param.value()));
            }
            TextureSource::Function { .. } => {}
        };
    }

    fn tex_sub_image_2d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.pixel_storages(gl);

        // buffers image sub data
        let result = match self {
            TextureSource::Function {
                width,
                height,
                callback,
            } => {
                let source = callback.borrow_mut()();
                if let TextureSource::Function { .. } = source {
                    panic!("recursive TextureSource::Function is not allowed");
                }
                if *width != source.width() {
                    panic!("source returned from TextureSource::Function should have same width");
                }
                if *height != source.height() {
                    panic!("source returned from TextureSource::Function should have same height");
                }
                source.tex_sub_image_2d(gl, target, level, x_offset, y_offset)?;
                Ok(())
            }
            TextureSource::PixelBufferObject {
                width,
                height,
                buffer,
                format,
                data_type,
                pbo_offset,
                ..
            } => {
                let current_buffer = pixel_unpack_buffer_binding(gl);
                gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                let result = gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_i32(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    *pbo_offset as i32,
                );
                gl.bind_buffer(
                    WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                    current_buffer.as_ref(),
                );
                result
            }
            TextureSource::Binary {
                width,
                height,
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
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data.as_ref().as_ref(),
                *src_offset as u32,
            ),
            TextureSource::Uint8Array { .. }
            | TextureSource::Uint8ClampedArray { .. }
            | TextureSource::Int8Array { .. }
            | TextureSource::Uint16Array { .. }
            | TextureSource::Int16Array { .. }
            | TextureSource::Uint32Array { .. }
            | TextureSource::Int32Array { .. }
            | TextureSource::Float32Array { .. } => {
                let (width, height, data, format, data_type, src_offset) = texture_source_data_views! {
                    self,
                    (Uint8Array, WebGl2RenderingContext::UNSIGNED_BYTE)
                    (Uint8ClampedArray, WebGl2RenderingContext::UNSIGNED_BYTE)
                    (Int8Array, WebGl2RenderingContext::BYTE)
                    (Int16Array, WebGl2RenderingContext::SHORT)
                    (Int32Array, WebGl2RenderingContext::INT)
                    (Float32Array, WebGl2RenderingContext::FLOAT)
                };
                gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type,
                    data,
                    *src_offset  as u32
                )
            }
            TextureSource::HtmlCanvasElement {
                format,
                data_type,
                canvas,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        canvas,
                    ),
                None => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        canvas.width() as i32,
                        canvas.height() as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        canvas,
                    ),
            },
            TextureSource::HtmlImageElement {
                format,
                data_type,
                image,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        image,
                    ),
                None => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        image.natural_width() as i32,
                        image.natural_height() as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        image,
                    ),
            },
            TextureSource::HtmlVideoElement {
                video,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        video,
                    ),
                None => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        video.video_width() as i32,
                        video.video_height() as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        video,
                    ),
            },
            TextureSource::ImageData {
                data,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        data,
                    ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    data.width() as i32,
                    data.height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    data,
                ),
            },
            TextureSource::ImageBitmap {
                bitmap,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl
                    .tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                        target.gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        *width as i32,
                        *height as i32,
                        format.gl_enum(),
                        data_type.gl_enum(),
                        bitmap,
                    ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    bitmap.width() as i32,
                    bitmap.height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    bitmap,
                ),
            },
        };

        result.map_err(|err| Error::TexImageFailure(err.as_string()))
    }

    fn tex_sub_image_3d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        self.pixel_storages(gl);

        // buffers image sub data
        let result = match self {
            TextureSource::Function {
                width,
                height,
                callback,
            } => {
                let source = callback.borrow_mut()();
                if let TextureSource::Function { .. } = source {
                    panic!("recursive TextureSource::Function is not allowed");
                }
                if *width != source.width() {
                    panic!("source returned from TextureSource::Function should have same width");
                }
                if *height != source.height() {
                    panic!("source returned from TextureSource::Function should have same height");
                }
                source.tex_sub_image_3d(gl, target, level, depth, x_offset, y_offset, z_offset)?;
                Ok(())
            }
            TextureSource::PixelBufferObject {
                width,
                height,
                buffer,
                format,
                data_type,
                pbo_offset,
                ..
            } => {
                let current_buffer = pixel_unpack_buffer_binding(gl);
                gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                let result = gl.tex_sub_image_3d_with_i32(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    *pbo_offset as i32,
                );
                gl.bind_buffer(
                    WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                    current_buffer.as_ref(),
                );
                result
            }
            TextureSource::Binary {
                width,
                height,
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
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                Some(data.as_ref().as_ref()),
                *src_offset as u32,
            ),
            TextureSource::Uint8Array { .. }
            | TextureSource::Uint8ClampedArray { .. }
            | TextureSource::Int8Array { .. }
            | TextureSource::Uint16Array { .. }
            | TextureSource::Int16Array { .. }
            | TextureSource::Uint32Array { .. }
            | TextureSource::Int32Array { .. }
            | TextureSource::Float32Array { .. } => {
                let (width, height, data, format, data_type, src_offset) = texture_source_data_views! {
                    self,
                    (Uint8Array, WebGl2RenderingContext::UNSIGNED_BYTE)
                    (Uint8ClampedArray, WebGl2RenderingContext::UNSIGNED_BYTE)
                    (Int8Array, WebGl2RenderingContext::BYTE)
                    (Int16Array, WebGl2RenderingContext::SHORT)
                    (Int32Array, WebGl2RenderingContext::INT)
                    (Float32Array, WebGl2RenderingContext::FLOAT)
                };
                gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type,
                    Some(data),
                    *src_offset as u32,
                )
            }
            TextureSource::HtmlCanvasElement {
                format,
                data_type,
                canvas,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_3d_with_html_canvas_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    canvas,
                ),
                None => gl.tex_sub_image_3d_with_html_canvas_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    canvas.width() as i32,
                    canvas.height() as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    canvas,
                ),
            },
            TextureSource::HtmlImageElement {
                format,
                data_type,
                image,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_3d_with_html_image_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    image,
                ),
                None => gl.tex_sub_image_3d_with_html_image_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    image.natural_width() as i32,
                    image.natural_height() as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    image,
                ),
            },
            TextureSource::HtmlVideoElement {
                video,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_3d_with_html_video_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    video,
                ),
                None => gl.tex_sub_image_3d_with_html_video_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    video.video_width() as i32,
                    video.video_height() as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    video,
                ),
            },
            TextureSource::ImageData {
                data,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_3d_with_image_data(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    data,
                ),
                None => gl.tex_sub_image_3d_with_image_data(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    data.width() as i32,
                    data.height() as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    data,
                ),
            },
            TextureSource::ImageBitmap {
                bitmap,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_3d_with_image_bitmap(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    *width as i32,
                    *height as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    bitmap,
                ),
                None => gl.tex_sub_image_3d_with_image_bitmap(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    z_offset as i32,
                    bitmap.width() as i32,
                    bitmap.height() as i32,
                    depth as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    bitmap,
                ),
            },
        };

        result.map_err(|err| Error::TexImageFailure(err.as_string()))
    }
}

macro_rules! texture_sources_compressed {
    ($(($name:ident, $data_view:ident))+) => {
        pub enum TextureSourceCompressed {
            Function {
                width: usize,
                height: usize,
                bytes_length: usize,
                callback: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
            },
            PixelBufferObject {
                width: usize,
                height: usize,
                buffer: WebGlBuffer,
                image_size: usize,
                pbo_offset: usize,
            },
            $(
                $name {
                    width: usize,
                    height: usize,
                    data: $data_view,
                    src_offset: usize,
                    src_length_override: Option<usize>,
                },
            )+
        }

        impl TextureSourceCompressed {
            pub fn bytes_length(&self) -> usize {
                match self {
                    TextureSourceCompressed::Function { bytes_length, .. } => *bytes_length,
                    TextureSourceCompressed::PixelBufferObject { image_size, .. } => *image_size,
                    $(
                        TextureSourceCompressed::$name {
                            data,
                            src_length_override,
                            src_offset,
                            ..
                        } => src_length_override.unwrap_or(data.byte_length() as usize - *src_offset),
                    )+
                }
            }

            pub fn width(&self) -> usize {
                match self {
                    TextureSourceCompressed::Function { width, .. }
                    | TextureSourceCompressed::PixelBufferObject { width, .. }
                    $(
                        | TextureSourceCompressed::$name { width, .. }
                    )+
                    => *width,
                }
            }

            pub fn height(&self) -> usize {
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
                target: TextureTarget,
                format: TextureCompressedFormat,
                level: usize,
                x_offset: usize,
                y_offset: usize,
            ) -> Result<(), Error> {
                // buffers image sub data
                match self {
                    TextureSourceCompressed::Function {
                        width,
                        height,
                        bytes_length,
                        callback,
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceCompressed::Function { .. } = source {
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if *width != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if *height != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
                        }
                        if *bytes_length != source.bytes_length() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
                        }
                        source.tex_sub_image_2d(gl, target, format, level, x_offset, y_offset)?;
                        Ok(())
                    }
                    TextureSourceCompressed::PixelBufferObject {
                        width,
                        height,
                        buffer,
                        image_size,
                        pbo_offset,
                        ..
                    } => {
                        let current_buffer = pixel_unpack_buffer_binding(gl);
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        gl.compressed_tex_sub_image_2d_with_i32_and_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            *width as i32,
                            *height as i32,
                            format.gl_enum(),
                            *image_size as i32,
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            current_buffer.as_ref(),
                        );
                        Ok(())
                    }
                    $(
                        TextureSourceCompressed::$name { .. }
                    ) | + => {
                        let (width, height, data, src_offset, src_length_override) = match self {
                            $(
                                TextureSourceCompressed::$name {
                                    width,
                                    height,
                                    data,
                                    src_offset,
                                    src_length_override,
                                    ..
                                } => (
                                    width,
                                    height,
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
                            *width as i32,
                            *height as i32,
                            format.gl_enum(),
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
                target: TextureTarget,
                format: TextureCompressedFormat,
                level: usize,
                depth: usize,
                x_offset: usize,
                y_offset: usize,
                z_offset: usize,
            ) -> Result<(), Error> {
                // buffers image sub data
                match self {
                    TextureSourceCompressed::Function {
                        width,
                        height,
                        bytes_length,
                        callback,
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceCompressed::Function { .. } = source {
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if *width != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if *height != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
                        }
                        if *bytes_length != source.bytes_length() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
                        }
                        source.tex_sub_image_3d(
                            gl, target, format, level, depth, x_offset, y_offset, z_offset,
                        )?;
                        Ok(())
                    }
                    TextureSourceCompressed::PixelBufferObject {
                        width,
                        height,
                        buffer,
                        image_size,
                        pbo_offset,
                        ..
                    } => {
                        let current_buffer = pixel_unpack_buffer_binding(gl);
                        gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, Some(buffer));
                        gl.compressed_tex_sub_image_3d_with_i32_and_i32(
                            target.gl_enum(),
                            level as i32,
                            x_offset as i32,
                            y_offset as i32,
                            z_offset as i32,
                            *width as i32,
                            *height as i32,
                            depth as i32,
                            format.gl_enum(),
                            *image_size as i32,
                            *pbo_offset as i32,
                        );
                        gl.bind_buffer(
                            WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
                            current_buffer.as_ref(),
                        );
                        Ok(())
                    }
                    $(
                        TextureSourceCompressed::$name { .. }
                    ) | + => {
                        let (width, height, data, src_offset, src_length_override) = match self {
                            $(
                                TextureSourceCompressed::$name {
                                    width,
                                    height,
                                    data,
                                    src_offset,
                                    src_length_override,
                                    ..
                                } => (
                                    width,
                                    height,
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
                            *width as i32,
                            *height as i32,
                            depth as i32,
                            format.gl_enum(),
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
    (Uint8ClampedArray, Uint8Array)
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

pub struct Texture2DArray {
    width: usize,
    height: usize,
    array_length: usize,
    internal_format: TextureInternalFormat,
    memory_policy: MemoryPolicy<Restorer>,
    queue: Vec<(TextureSource, usize, usize, usize, usize, usize)>,
}

pub struct TextureCubeMap {
    width: usize,
    height: usize,
    internal_format: TextureInternalFormat,
    memory_policy: MemoryPolicy<CubeMapRestorer>,
    positive_x: Vec<(TextureSource, usize, usize, usize)>,
    negative_x: Vec<(TextureSource, usize, usize, usize)>,
    positive_y: Vec<(TextureSource, usize, usize, usize)>,
    negative_y: Vec<(TextureSource, usize, usize, usize)>,
    positive_z: Vec<(TextureSource, usize, usize, usize)>,
    negative_z: Vec<(TextureSource, usize, usize, usize)>,
}

struct Runtime<T> {
    id: Uuid,
    gl: WebGl2RenderingContext,
    store_id: Uuid,
    texture: WebGlTexture,
    bytes_length: usize,
    lru_node: *mut LruNode<Uuid>,
    using: bool,

    used_memory: *mut usize,
    descriptors: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<T>>>>,
    lru: *mut Lru<Uuid>,
}

impl<T> Drop for Runtime<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.descriptors).remove(&self.id);
            (*self.lru).remove(self.lru_node);
            (*self.used_memory) -= self.bytes_length;
            self.gl.delete_texture(Some(&self.texture));
        }
    }
}

struct TextureDescriptorInner<T> {
    name: Option<Cow<'static, str>>,
    layout: T,
    generate_mipmap: bool,

    runtime: Option<Box<Runtime<T>>>,
}

impl TextureDescriptorInner<Texture2DArray> {
    fn max_mipmap_level(&self) -> usize {
        if !self.generate_mipmap {
            return 0;
        }

        (self.layout.width as f64)
            .max(self.layout.height as f64)
            .log2()
            .floor() as usize
    }

    fn width_of_level(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.layout.width >> level).max(1))
    }

    fn height_of_level(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.layout.height >> level).max(1))
    }

    fn bytes_length(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        let array_length = self.layout.array_length;
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += self.layout.internal_format.bytes_length(width, height) * array_length;
        }
        used_memory
    }

    fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };

        Some(self.layout.internal_format.bytes_length(width, height) * self.layout.array_length)
    }

    fn verify_tex_image_level_size(
        &self,
        level: usize,
        width: usize,
        height: usize,
        array_index: usize,
    ) -> Result<(), Error> {
        if self
            .width_of_level(level)
            .map(|w| w != width)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if self
            .height_of_level(level)
            .map(|h| h != height)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if array_index >= self.layout.array_length {
            return Err(Error::TextureSizeMismatched);
        }

        Ok(())
    }

    fn verify_tex_sub_image_level_size(
        &self,
        level: usize,
        width: usize,
        height: usize,
        array_index: usize,
        x_offset: usize,
        y_offset: usize,
        array_index_offset: usize,
    ) -> Result<(), Error> {
        if self
            .width_of_level(level)
            .map(|w| width + x_offset > w)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if self
            .height_of_level(level)
            .map(|h| height + y_offset > h)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if array_index_offset + array_index >= self.layout.array_length {
            return Err(Error::TextureSizeMismatched);
        }

        Ok(())
    }
}

impl TextureDescriptorInner<TextureCubeMap> {
    fn max_mipmap_level(&self) -> usize {
        if !self.generate_mipmap {
            return 0;
        }

        (self.layout.width as f64)
            .max(self.layout.height as f64)
            .log2()
            .floor() as usize
    }

    fn width_of_level(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.layout.width >> level).max(1))
    }

    fn height_of_level(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.layout.height >> level).max(1))
    }

    fn bytes_length(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += self.layout.internal_format.bytes_length(width, height) * 6;
        }
        used_memory
    }

    fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };

        Some(self.layout.internal_format.bytes_length(width, height) * 6)
    }

    fn verify_tex_image_level_size(
        &self,
        level: usize,
        width: usize,
        height: usize,
    ) -> Result<(), Error> {
        if width != height {
            return Err(Error::TextureCubeMapWidthAndHeightNotEqual);
        }
        if self
            .width_of_level(level)
            .map(|w| w != width)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if self
            .height_of_level(level)
            .map(|h| h != height)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }

        Ok(())
    }

    fn verify_tex_sub_image_level_size(
        &self,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        if width != height {
            return Err(Error::TextureCubeMapWidthAndHeightNotEqual);
        }
        if self
            .width_of_level(level)
            .map(|w| width + x_offset > w)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }
        if self
            .height_of_level(level)
            .map(|h| height + y_offset > h)
            .unwrap_or(true)
        {
            return Err(Error::TextureSizeMismatched);
        }

        Ok(())
    }
}

pub struct TextureDescriptor<T>(Rc<RefCell<TextureDescriptorInner<T>>>);

impl<T> Clone for TextureDescriptor<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> TextureDescriptor<T> {
    /// Returns buffer descriptor name.
    pub fn name(&self) -> Ref<Option<Cow<'static, str>>> {
        Ref::map(self.0.borrow(), |inner| &inner.name)
    }

    /// Sets buffer descriptor name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.0.borrow_mut().name.replace(Cow::Owned(name.into()));
    }

    /// Sets buffer descriptor name.
    pub fn set_name_str(&mut self, name: &'static str) {
        self.0.borrow_mut().name.replace(Cow::Borrowed(name));
    }

    pub fn generate_mipmap(&self) -> bool {
        self.0.borrow().generate_mipmap
    }
}

macro_rules! texture_2d {
    ($(($name:ident, $f:ident, $r:ident, $s:ident))+) => {
        $(
            pub struct $name {
                width: usize,
                height: usize,
                internal_format: $f,
                memory_policy: MemoryPolicy<$r>,
                queue: Vec<($s, usize, usize, usize)>,
            }

            impl TextureDescriptorInner<$name> {
                fn max_mipmap_level(&self) -> usize {
                    if !self.generate_mipmap {
                        return 0;
                    }

                    (self.layout.width as f64)
                        .max(self.layout.height as f64)
                        .log2()
                        .floor() as usize
                }

                fn width_of_level(&self, level: usize) -> Option<usize> {
                    let max_level = self.max_mipmap_level();
                    if level > max_level {
                        return None;
                    }

                    Some((self.layout.width >> level).max(1))
                }

                fn height_of_level(&self, level: usize) -> Option<usize> {
                    let max_level = self.max_mipmap_level();
                    if level > max_level {
                        return None;
                    }

                    Some((self.layout.height >> level).max(1))
                }

                fn bytes_length(&self) -> usize {
                    // estimates used memory of all levels
                    let mut used_memory = 0;
                    for level in 0..=self.max_mipmap_level() {
                        let width = self.width_of_level(level).unwrap();
                        let height = self.height_of_level(level).unwrap();
                        used_memory += self.layout.internal_format.bytes_length(width, height);
                    }
                    used_memory
                }

                fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
                    let Some(width) = self.width_of_level(level) else {
                        return None;
                    };
                    let Some(height) = self.height_of_level(level) else {
                        return None;
                    };

                    Some(self.layout.internal_format.bytes_length(width, height))
                }

                fn verify_tex_image_level_size(
                    &self,
                    level: usize,
                    width: usize,
                    height: usize,
                ) -> Result<(), Error> {
                    if self
                        .width_of_level(level)
                        .map(|w| w != width)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .height_of_level(level)
                        .map(|h| h != height)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }

                    Ok(())
                }

                fn verify_tex_sub_image_level_size(
                    &self,
                    level: usize,
                    width: usize,
                    height: usize,
                    x_offset: usize,
                    y_offset: usize,
                ) -> Result<(), Error> {
                    if self
                        .width_of_level(level)
                        .map(|w| width + x_offset > w)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .height_of_level(level)
                        .map(|h| height + y_offset > h)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }

                    Ok(())
                }
            }

            impl TextureDescriptor<$name> {
                pub fn internal_format(&self) -> $f {
                    self.0.borrow().layout.internal_format
                }

                pub fn memory_policy(&self) -> Ref<MemoryPolicy<$r>> {
                    Ref::map(self.0.borrow(), |inner| &inner.layout.memory_policy)
                }

                pub fn width(&self) -> usize {
                    self.0.borrow().layout.width
                }

                pub fn height(&self) -> usize {
                    self.0.borrow().layout.height
                }

                pub fn max_mipmap_level(&self) -> usize {
                    self.0.borrow().max_mipmap_level()
                }

                pub fn width_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().width_of_level(level)
                }

                pub fn height_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().height_of_level(level)
                }

                pub fn bytes_length(&self) -> usize {
                    self.0.borrow().bytes_length()
                }

                pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().bytes_length_of_level(level)
                }
            }
        )+
    };
}

texture_2d! {
    (Texture2D, TextureInternalFormat, Restorer, TextureSource)
    (Texture2DCompressed, TextureCompressedFormat, RestorerCompressed, TextureSourceCompressed)
}

impl TextureDescriptorInner<Texture2D> {
    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.queue.is_empty() {
            return Ok(());
        }

        for (source, level, x_offset, y_offset) in self.layout.queue.drain(..) {
            abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_2d(gl, TextureTarget::TEXTURE_2D, level, x_offset, y_offset)?;
        }

        Ok(())
    }
}

impl TextureDescriptorInner<Texture2DCompressed> {
    fn verify_compressed_size(
        &self,
        source: &TextureSourceCompressed,
        width: usize,
        height: usize,
    ) -> Result<(), Error> {
        if self.layout.internal_format.bytes_length(width, height) != source.bytes_length() {
            return Err(Error::TextureCompressedImageSizeMismatched);
        }

        Ok(())
    }

    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.queue.is_empty() {
            return Ok(());
        }

        for (source, level, x_offset, y_offset) in self.layout.queue.drain(..) {
            abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_2d(
                gl,
                TextureTarget::TEXTURE_2D,
                self.layout.internal_format,
                level,
                x_offset,
                y_offset,
            )?;
        }

        Ok(())
    }
}

impl TextureDescriptor<Texture2D> {
    pub fn new(
        width: usize,
        height: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2D {
                width,
                height,
                memory_policy,
                internal_format,
                queue: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Result<Self, Error> {
        let width = source.width();
        let height = source.height();

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2D {
                width,
                height,
                internal_format,
                memory_policy,
                queue: vec![(source, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        }))))
    }

    pub fn tex_image(&mut self, source: TextureSource, level: usize) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_image_level_size(level, width, height)?;

        inner.layout.queue.push((source, level, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSource,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_sub_image_level_size(level, width, height, x_offset, y_offset)?;

        inner.layout.queue.push((source, level, x_offset, y_offset));
        Ok(())
    }
}

impl TextureDescriptor<Texture2DCompressed> {
    pub fn new(
        width: usize,
        height: usize,
        internal_format: TextureCompressedFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<RestorerCompressed>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DCompressed {
                width,
                height,
                memory_policy,
                internal_format,
                queue: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSourceCompressed,
        internal_format: TextureCompressedFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<RestorerCompressed>,
    ) -> Result<Self, Error> {
        let width = source.width();
        let height = source.height();
        if internal_format.bytes_length(width, height) != source.bytes_length() {
            return Err(Error::TextureCompressedImageSizeMismatched);
        }

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DCompressed {
                width,
                height,
                internal_format,
                memory_policy,
                queue: vec![(source, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        }))))
    }

    pub fn tex_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_image_level_size(level, width, height)?;
        inner.verify_compressed_size(&source, width, height)?;

        inner.layout.queue.push((source, level, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_sub_image_level_size(level, width, height, x_offset, y_offset)?;
        inner.verify_compressed_size(&source, width, height)?;

        inner.layout.queue.push((source, level, x_offset, y_offset));
        Ok(())
    }
}

macro_rules! texture_3d {
    ($(($name:ident, $f:ident, $r:ident, $s:ident))+) => {
        $(
            pub struct $name {
                width: usize,
                height: usize,
                depth: usize,
                internal_format: $f,
                memory_policy: MemoryPolicy<$r>,
                queue: Vec<($s, usize, usize, usize, usize, usize)>,
            }

            impl TextureDescriptorInner<$name> {
                fn max_mipmap_level(&self) -> usize {
                    if !self.generate_mipmap {
                        return 0;
                    }

                    (self.layout.width as f64)
                        .max(self.layout.height as f64)
                        .log2()
                        .floor() as usize
                }

                fn width_of_level(&self, level: usize) -> Option<usize> {
                    let max_level = self.max_mipmap_level();
                    if level > max_level {
                        return None;
                    }

                    Some((self.layout.width >> level).max(1))
                }

                fn height_of_level(&self, level: usize) -> Option<usize> {
                    let max_level = self.max_mipmap_level();
                    if level > max_level {
                        return None;
                    }

                    Some((self.layout.height >> level).max(1))
                }

                fn depth_of_level(&self, level: usize) -> Option<usize> {
                    let max_level = self.max_mipmap_level();
                    if level > max_level {
                        return None;
                    }

                    Some((self.layout.depth >> level).max(1))
                }

                fn bytes_length(&self) -> usize {
                    // estimates used memory of all levels
                    let mut used_memory = 0;
                    for level in 0..=self.max_mipmap_level() {
                        let width = self.width_of_level(level).unwrap();
                        let height = self.height_of_level(level).unwrap();
                        let depth = self.depth_of_level(level).unwrap();
                        used_memory += self
                            .layout
                            .internal_format
                            .bytes_length(width, height) * depth;
                    }
                    used_memory
                }

                fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
                    let Some(width) = self.width_of_level(level) else {
                        return None;
                    };
                    let Some(height) = self.height_of_level(level) else {
                        return None;
                    };
                    let Some(depth) = self.depth_of_level(level) else {
                        return None;
                    };

                    Some(
                        self.layout
                            .internal_format
                            .bytes_length(width, height) * depth,
                    )
                }

                fn verify_tex_image_level_size(
                    &self,
                    level: usize,
                    width: usize,
                    height: usize,
                    depth: usize,
                ) -> Result<(), Error> {
                    if self
                        .width_of_level(level)
                        .map(|w| w != width)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .height_of_level(level)
                        .map(|h| h != height)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .depth_of_level(level)
                        .map(|d| d != depth)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }

                    Ok(())
                }

                fn verify_tex_sub_image_level_size(
                    &self,
                    level: usize,
                    width: usize,
                    height: usize,
                    depth: usize,
                    x_offset: usize,
                    y_offset: usize,
                    z_offset: usize,
                ) -> Result<(), Error> {
                    if self
                        .width_of_level(level)
                        .map(|w| width + x_offset > w)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .height_of_level(level)
                        .map(|h| height + y_offset > h)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }
                    if self
                        .depth_of_level(level)
                        .map(|d| depth + z_offset > d)
                        .unwrap_or(true)
                    {
                        return Err(Error::TextureSizeMismatched);
                    }

                    Ok(())
                }
            }

            impl TextureDescriptor<$name> {

                pub fn internal_format(&self) -> $f {
                    self.0.borrow().layout.internal_format
                }

                pub fn memory_policy(&self) -> Ref<MemoryPolicy<$r>> {
                    Ref::map(self.0.borrow(), |inner| &inner.layout.memory_policy)
                }

                pub fn width(&self) -> usize {
                    self.0.borrow().layout.width
                }

                pub fn height(&self) -> usize {
                    self.0.borrow().layout.height
                }

                pub fn depth(&self) -> usize {
                    self.0.borrow().layout.depth
                }

                pub fn max_mipmap_level(&self) -> usize {
                    self.0.borrow().max_mipmap_level()
                }

                pub fn width_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().width_of_level(level)
                }

                pub fn height_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().height_of_level(level)
                }

                pub fn depth_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().depth_of_level(level)
                }

                pub fn bytes_length(&self) -> usize {
                    self.0.borrow().bytes_length()
                }

                pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
                    self.0.borrow().bytes_length_of_level(level)
                }
            }
        )+
    };
}

texture_3d! {
    (Texture3D, TextureInternalFormat, Restorer, TextureSource)
    (Texture3DCompressed, TextureCompressedFormat, RestorerCompressed, TextureSourceCompressed)
}

impl TextureDescriptorInner<Texture3D> {
    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.queue.is_empty() {
            return Ok(());
        }

        for (source, level, depth, x_offset, y_offset, z_offset) in self.layout.queue.drain(..) {
            abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_3d(
                gl,
                TextureTarget::TEXTURE_3D,
                level,
                depth,
                x_offset,
                y_offset,
                z_offset,
            )?;
        }

        Ok(())
    }
}

impl TextureDescriptorInner<Texture3DCompressed> {
    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.queue.is_empty() {
            return Ok(());
        }

        for (source, level, depth, x_offset, y_offset, z_offset) in self.layout.queue.drain(..) {
            abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_3d(
                gl,
                TextureTarget::TEXTURE_3D,
                self.layout.internal_format,
                level,
                depth,
                x_offset,
                y_offset,
                z_offset,
            )?;
        }

        Ok(())
    }
}

impl TextureDescriptor<Texture3D> {
    pub fn new(
        width: usize,
        height: usize,
        depth: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3D {
                width,
                height,
                depth,
                internal_format,
                memory_policy,
                queue: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        depth: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3D {
                width: source.width(),
                height: source.height(),
                depth,
                internal_format,
                memory_policy,
                queue: vec![(source, 0, 0, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn tex_image(
        &mut self,
        source: TextureSource,
        level: usize,
        depth: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_image_level_size(level, width, height, depth)?;

        inner.layout.queue.push((source, level, depth, 0, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSource,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_sub_image_level_size(
            level, width, height, depth, x_offset, y_offset, z_offset,
        )?;

        inner
            .layout
            .queue
            .push((source, level, depth, x_offset, y_offset, z_offset));
        Ok(())
    }
}

impl TextureDescriptor<Texture3DCompressed> {
    pub fn new(
        width: usize,
        height: usize,
        depth: usize,
        internal_format: TextureCompressedFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<RestorerCompressed>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3DCompressed {
                width,
                height,
                depth,
                internal_format,
                memory_policy,
                queue: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSourceCompressed,
        depth: usize,
        internal_format: TextureCompressedFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<RestorerCompressed>,
    ) -> Result<Self, Error> {
        let width = source.width();
        let height = source.height();
        if internal_format.bytes_length(width, height) != source.bytes_length() {
            return Err(Error::TextureCompressedImageSizeMismatched);
        }

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3DCompressed {
                width: source.width(),
                height: source.height(),
                depth,
                internal_format,
                memory_policy,
                queue: vec![(source, 0, 0, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        }))))
    }

    pub fn tex_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
        depth: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_image_level_size(level, width, height, depth)?;

        inner.layout.queue.push((source, level, depth, 0, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
        depth: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_sub_image_level_size(
            level, width, height, depth, x_offset, y_offset, z_offset,
        )?;

        inner
            .layout
            .queue
            .push((source, level, depth, x_offset, y_offset, z_offset));
        Ok(())
    }
}

impl TextureDescriptorInner<Texture2DArray> {
    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.queue.is_empty() {
            return Ok(());
        }

        for (source, level, array_index, x_offset, y_offset, array_index_offset) in
            self.layout.queue.drain(..)
        {
            abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_3d(
                gl,
                TextureTarget::TEXTURE_2D_ARRAY,
                level,
                array_index,
                x_offset,
                y_offset,
                array_index_offset,
            )?;
        }

        Ok(())
    }
}

impl TextureDescriptorInner<TextureCubeMap> {
    fn tex(&mut self, gl: &WebGl2RenderingContext, abilities: &Abilities) -> Result<(), Error> {
        if self.layout.positive_x.len()
            + self.layout.negative_x.len()
            + self.layout.positive_y.len()
            + self.layout.negative_y.len()
            + self.layout.positive_z.len()
            + self.layout.negative_z.len()
            == 0
        {
            return Ok(());
        }

        let faces = [
            (
                TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_X,
                &mut self.layout.positive_x,
            ),
            (
                TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_X,
                &mut self.layout.negative_x,
            ),
            (
                TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_Y,
                &mut self.layout.positive_y,
            ),
            (
                TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                &mut self.layout.negative_y,
            ),
            (
                TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_Z,
                &mut self.layout.positive_z,
            ),
            (
                TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                &mut self.layout.negative_z,
            ),
        ];
        for (target, face) in faces {
            for (source, level, x_offset, y_offset) in face.drain(..) {
                abilities.verify_texture_size(source.width(), source.height())?;
                source.tex_sub_image_2d(gl, target, level, x_offset, y_offset)?;
            }
        }

        Ok(())
    }
}

impl TextureDescriptor<Texture2DArray> {
    pub fn new(
        width: usize,
        height: usize,
        array_length: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DArray {
                width,
                height,
                array_length,
                internal_format,
                memory_policy,
                queue: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        array_length: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<Restorer>,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DArray {
                width: source.width(),
                height: source.height(),
                array_length,
                internal_format,
                memory_policy,
                queue: vec![(source, 0, 0, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        })))
    }

    pub fn internal_format(&self) -> TextureInternalFormat {
        self.0.borrow().layout.internal_format
    }

    pub fn memory_policy(&self) -> Ref<MemoryPolicy<Restorer>> {
        Ref::map(self.0.borrow(), |inner| &inner.layout.memory_policy)
    }

    pub fn width(&self) -> usize {
        self.0.borrow().layout.width
    }

    pub fn height(&self) -> usize {
        self.0.borrow().layout.height
    }

    pub fn array_length(&self) -> usize {
        self.0.borrow().layout.array_length
    }

    pub fn max_mipmap_level(&self) -> usize {
        self.0.borrow().max_mipmap_level()
    }

    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().width_of_level(level)
    }

    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().height_of_level(level)
    }

    pub fn bytes_length(&self) -> usize {
        self.0.borrow().bytes_length()
    }

    pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().bytes_length_of_level(level)
    }

    pub fn tex_image(
        &mut self,
        source: TextureSource,
        level: usize,
        array_index: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_image_level_size(level, width, height, array_index)?;

        inner
            .layout
            .queue
            .push((source, level, array_index, 0, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSource,
        level: usize,
        array_index: usize,
        x_offset: usize,
        y_offset: usize,
        array_index_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_tex_sub_image_level_size(
            level,
            width,
            height,
            array_index,
            x_offset,
            y_offset,
            array_index_offset,
        )?;

        inner.layout.queue.push((
            source,
            level,
            array_index,
            x_offset,
            y_offset,
            array_index_offset,
        ));
        Ok(())
    }
}

impl TextureDescriptor<TextureCubeMap> {
    pub fn new(
        width: usize,
        height: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<CubeMapRestorer>,
    ) -> Result<Self, Error> {
        if width != height {
            return Err(Error::TextureCubeMapWidthAndHeightNotEqual);
        }

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: TextureCubeMap {
                width,
                height,
                internal_format,
                memory_policy,
                positive_x: Vec::new(),
                negative_x: Vec::new(),
                positive_y: Vec::new(),
                negative_y: Vec::new(),
                positive_z: Vec::new(),
                negative_z: Vec::new(),
            },
            generate_mipmap,

            runtime: None,
        }))))
    }

    pub fn with_sources(
        positive_x: TextureSource,
        negative_x: TextureSource,
        positive_y: TextureSource,
        negative_y: TextureSource,
        positive_z: TextureSource,
        negative_z: TextureSource,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy<CubeMapRestorer>,
    ) -> Result<Self, Error> {
        let width = positive_x.width();
        let height = positive_x.height();
        if width != height {
            return Err(Error::TextureCubeMapWidthAndHeightNotEqual);
        }

        macro_rules! check_sizes {
            ($($s:ident),+) => {
                $(
                    if width != $s.width() {
                        return Err(Error::TextureCubeMapFacesSizeNotEqual);
                    }
                    if height != $s.height() {
                        return Err(Error::TextureCubeMapFacesSizeNotEqual);
                    }
                )+
            };
        }
        check_sizes!(negative_x, positive_y, negative_y, positive_z, negative_z);

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: TextureCubeMap {
                width,
                height,
                internal_format,
                memory_policy,
                positive_x: vec![(positive_x, 0, 0, 0)],
                negative_x: vec![(negative_x, 0, 0, 0)],
                positive_y: vec![(positive_y, 0, 0, 0)],
                negative_y: vec![(negative_y, 0, 0, 0)],
                positive_z: vec![(positive_z, 0, 0, 0)],
                negative_z: vec![(negative_z, 0, 0, 0)],
            },
            generate_mipmap,

            runtime: None,
        }))))
    }

    pub fn internal_format(&self) -> TextureInternalFormat {
        self.0.borrow().layout.internal_format
    }

    pub fn memory_policy(&self) -> Ref<MemoryPolicy<CubeMapRestorer>> {
        Ref::map(self.0.borrow(), |inner| &inner.layout.memory_policy)
    }

    pub fn width(&self) -> usize {
        self.0.borrow().layout.width
    }

    pub fn height(&self) -> usize {
        self.0.borrow().layout.height
    }

    pub fn max_mipmap_level(&self) -> usize {
        self.0.borrow().max_mipmap_level()
    }

    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().width_of_level(level)
    }

    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().height_of_level(level)
    }

    pub fn bytes_length(&self) -> usize {
        self.0.borrow().bytes_length()
    }

    pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().bytes_length_of_level(level)
    }
}

macro_rules! tex_cube_map {
    ($(($tex_func:ident, $tex_sub_func:ident, $queue:ident))+) => {
        $(
            impl TextureDescriptor<TextureCubeMap> {
                pub fn $tex_func(
                    &mut self,
                    source: TextureSource,
                    level: usize,
                ) -> Result<(), Error> {
                    let mut inner = self.0.borrow_mut();
                    let width = source.width();
                    let height = source.height();
                    inner.verify_tex_image_level_size(level, width, height)?;
                    inner.layout.$queue.push((source, level, 0, 0));
                    Ok(())
                }

                pub fn $tex_sub_func(
                    &mut self,
                    source: TextureSource,
                    level: usize,
                    x_offset: usize,
                    y_offset: usize,
                ) -> Result<(), Error> {
                    let mut inner = self.0.borrow_mut();
                    let width = source.width();
                    let height = source.height();
                    inner.verify_tex_sub_image_level_size(level, width, height, x_offset, y_offset)?;
                    inner
                        .layout
                        .$queue
                        .push((source, level, x_offset, y_offset));
                    Ok(())
                }
            }
        )+
    };
}

tex_cube_map! {
    (tex_image_positive_x, tex_sub_image_positive_x, positive_x)
    (tex_image_negative_x, tex_sub_image_negative_x, negative_x)
    (tex_image_positive_y, tex_sub_image_positive_y, positive_y)
    (tex_image_negative_y, tex_sub_image_negative_y, negative_y)
    (tex_image_positive_z, tex_sub_image_positive_z, positive_z)
    (tex_image_negative_z, tex_sub_image_negative_z, negative_z)
}

pub struct TextureStore {
    id: Uuid,
    gl: WebGl2RenderingContext,
    abilities: Abilities,
    available_memory: usize,
    used_memory: *mut usize,
    lru: *mut Lru<Uuid>,
    descriptors_2d: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture2D>>>>,
    descriptors_2d_compressed:
        *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture2DCompressed>>>>,
    descriptors_2d_array: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture2DArray>>>>,
    descriptors_3d: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture3D>>>>,
    descriptors_3d_compressed:
        *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture3DCompressed>>>>,
    descriptors_cube_map: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<TextureCubeMap>>>>,
}

impl TextureStore {
    pub fn new(gl: WebGl2RenderingContext, abilities: Abilities) -> Self {
        Self::with_available_memory(gl, abilities, i32::MAX as usize)
    }

    pub fn with_available_memory(
        gl: WebGl2RenderingContext,
        abilities: Abilities,
        available_memory: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            abilities,
            available_memory,
            used_memory: Box::leak(Box::new(0)),
            descriptors_2d: Box::leak(Box::new(HashMap::new())),
            descriptors_2d_compressed: Box::leak(Box::new(HashMap::new())),
            descriptors_2d_array: Box::leak(Box::new(HashMap::new())),
            descriptors_3d: Box::leak(Box::new(HashMap::new())),
            descriptors_3d_compressed: Box::leak(Box::new(HashMap::new())),
            descriptors_cube_map: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),
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

                macro_rules! free_descriptor {
                    ($((
                        $descriptors:ident,
                        $(($restore:ident, $tex_queue:ident, $source:ident, $($tex_params:expr),+)),+
                    )),+) => {
                        $(
                            if let Entry::Occupied(occupied) = (*self.$descriptors).entry(*id) {
                                let descriptor = occupied.get();
                                let Some(descriptor) = descriptor.upgrade() else {
                                    occupied.remove();
                                    next_node = (*current_node).more_recently();
                                    continue;
                                };

                                let descriptor = descriptor.borrow();
                                let runtime = descriptor.runtime.as_ref().unwrap();

                                // skips if using
                                if runtime.using {
                                    next_node = (*current_node).more_recently();
                                    continue;
                                }
                                // skips if unfree
                                if let MemoryPolicy::Unfree = &descriptor.layout.memory_policy {
                                    next_node = (*current_node).more_recently();
                                    continue;
                                }

                                // free
                                let descriptor = occupied.remove().upgrade().unwrap();
                                let mut descriptor = descriptor.borrow_mut();
                                let runtime = descriptor.runtime.take().unwrap();
                                match &descriptor.layout.memory_policy {
                                    MemoryPolicy::Unfree => unreachable!(),
                                    MemoryPolicy::Restorable(restorer) => {
                                        self.gl.delete_texture(Some(&runtime.texture));
                                        let width = descriptor.layout.width;
                                        let height = descriptor.layout.height;

                                        $(
                                            let $source = TextureSource::Function {
                                                width,
                                                height,
                                                callback: Rc::clone(&restorer.$restore),
                                            };
                                        )+

                                        $(
                                            descriptor.layout.$tex_queue.push(($source, $($tex_params),+));
                                        )+
                                    }
                                }
                                // reduces used memory
                                (*self.used_memory) -= runtime.bytes_length;
                                // removes LRU
                                (*self.lru).remove(runtime.lru_node);

                                next_node = (*current_node).more_recently();
                                continue;
                            };
                        )+
                    };
                }
                free_descriptor!(
                    (descriptors_2d, (callback, queue, source, 0, 0, 0)),
                    (
                        descriptors_2d_array,
                        (callback, queue, source, 0, 0, 0, 0, 0)
                    ),
                    (descriptors_3d, (callback, queue, source, 0, 0, 0, 0, 0)),
                    (
                        descriptors_cube_map,
                        (positive_x, positive_x, source_px, 0, 0, 0),
                        (negative_x, negative_x, source_nx, 0, 0, 0),
                        (positive_y, positive_y, source_py, 0, 0, 0),
                        (negative_y, negative_y, source_ny, 0, 0, 0),
                        (positive_z, positive_z, source_pz, 0, 0, 0),
                        (negative_z, negative_z, source_nz, 0, 0, 0)
                    )
                );
            }
        }
    }
}

macro_rules! store_use_textures {
    ($((
        $layout:tt,
        $target:expr,
        $descriptors:ident,
        $verify_func:ident,
        $tex_storage_func:ident, ($($tex_storage_params:ident),+),
        $use_func: ident,
        $unuse_func:ident
    ))+) => {
        impl TextureStore {
            $(
                pub fn $use_func(
                    &mut self,
                    descriptor: &TextureDescriptor<$layout>,
                    unit: TextureUnit,
                ) -> Result<WebGlTexture, Error> {
                    self.abilities.verify_texture_unit(unit)?;

                    unsafe {
                        let mut inner = descriptor.0.borrow_mut();

                        let (texture, is_new) = match inner.runtime.as_mut() {
                            Some(runtime) => {
                                if runtime.store_id != self.id {
                                    panic!("share texture descriptor between texture store is not allowed");
                                }

                                runtime.using = true;
                                (*self.lru).cache(runtime.lru_node);

                                (runtime.texture.clone(), false)
                            }
                            None => {
                                debug!(
                                    target: "TextureBuffer",
                                    "create new texture for {}",
                                    inner.name.as_deref().unwrap_or("unnamed"),
                                );

                                self.abilities.$verify_func(inner.layout.internal_format)?;
                                let texture = self
                                    .gl
                                    .create_texture()
                                    .ok_or(Error::CreateTextureFailure)?;
                                self.gl.active_texture(unit.gl_enum());
                                self.gl
                                    .bind_texture($target, Some(&texture));
                                self.gl.$tex_storage_func(
                                    $target,
                                    (1 + inner.max_mipmap_level()) as i32,
                                    inner.layout.internal_format.gl_enum(),
                                    $(
                                        inner.layout.$tex_storage_params as i32
                                    ),+
                                );

                                let id = Uuid::new_v4();
                                let lru_node = LruNode::new(id);
                                let bytes_length = inner.bytes_length();
                                (*self.$descriptors).insert(id, Rc::downgrade(&descriptor.0));
                                (*self.lru).cache(lru_node);
                                (*self.used_memory) += bytes_length;
                                inner.runtime = Some(Box::new(Runtime {
                                    id,
                                    gl: self.gl.clone(),
                                    store_id: self.id,
                                    texture: texture.clone(),
                                    bytes_length,
                                    lru_node,
                                    using: true,

                                    used_memory: self.used_memory,
                                    descriptors: self.$descriptors,
                                    lru: self.lru,
                                }));
                                (texture, true)
                            }
                        };

                        self.gl.active_texture(unit.gl_enum());
                        self.gl
                            .bind_texture($target, Some(&texture));

                        inner.tex(&self.gl, &self.abilities)?;

                        if is_new && inner.generate_mipmap {
                            self.gl.generate_mipmap($target);
                        }

                        self.gl
                            .bind_texture($target, None);
                        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

                        drop(inner);
                        self.free();

                        Ok(texture)
                    }
                }

                pub fn $unuse_func(&mut self, descriptor: &TextureDescriptor<$layout>) {
                    let mut inner = descriptor.0.borrow_mut();

                    if let Some(runtime) = inner.runtime.as_mut() {
                        runtime.using = false;
                    }
                }
            )+
        }
    };
}

store_use_textures! {
    (
        Texture2D,
        WebGl2RenderingContext::TEXTURE_2D,
        descriptors_2d,
        verify_internal_format,
        tex_storage_2d, (width, height),
        use_texture_2d,
        unuse_texture_2d
    )
    (
        Texture2DCompressed,
        WebGl2RenderingContext::TEXTURE_2D,
        descriptors_2d_compressed,
        verify_compressed_format,
        tex_storage_2d,
            (width, height),
        use_texture_2d_compressed,
        unuse_texture_2d_compressed
    )
    (
        Texture3D,
        WebGl2RenderingContext::TEXTURE_3D,
        descriptors_3d,
        verify_internal_format,
        tex_storage_3d,
            (width, height, depth),
        use_texture_3d,
        unuse_texture_3d
    )
    (
        Texture3DCompressed,
        WebGl2RenderingContext::TEXTURE_3D,
        descriptors_3d_compressed,
        verify_compressed_format,
        tex_storage_3d,
            (width, height, depth),
        use_texture_3d_compressed,
        unuse_texture_3d_compressed
    )
    (
        Texture2DArray,
        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
        descriptors_2d_array,
        verify_internal_format,
        tex_storage_3d,
            (width, height, array_length),
        use_texture_2d_array,
        unuse_texture_2d_array
    )
    (
        TextureCubeMap,
        WebGl2RenderingContext::TEXTURE_CUBE_MAP,
        descriptors_cube_map,
        verify_internal_format,
        tex_storage_2d,
            (width, height),
        use_texture_cube_map,
        unuse_texture_cube_map
    )
}

macro_rules! store_drop {
    ($($d:ident),+) => {
        impl Drop for TextureStore {
            fn drop(&mut self) {
                unsafe {
                    $(
                        for (_, descriptor) in (*self.$d).iter() {
                            let Some(descriptor) = descriptor.upgrade() else {
                                return;
                            };
                            let mut descriptor = descriptor.borrow_mut();
                            let Some(runtime) = descriptor.runtime.take() else {
                                return;
                            };

                            self.gl.delete_texture(Some(&runtime.texture));
                            // store dropped, no need to update LRU anymore
                        }
                        drop(Box::from_raw(self.$d));
                    )+

                    drop(Box::from_raw(self.used_memory));
                    drop(Box::from_raw(self.lru));
                }
            }
        }
    };
}

store_drop!(
    descriptors_2d,
    descriptors_2d_compressed,
    descriptors_2d_array,
    descriptors_3d,
    descriptors_3d_compressed,
    descriptors_cube_map
);
