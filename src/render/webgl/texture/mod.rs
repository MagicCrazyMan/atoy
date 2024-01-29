//! Texture is created in Immutable Storage using `texStorage2D`
//! and then image data are uploaded by `texSubImage2D`.
//! Once the texture is created, the memory layout is no more alterable,
//! meaning that `texImage2D` and `texStorage2D` are no longer work.
//! But developer could still modify image data using `texSubImage2D`.
//! You have to create a new texture if you want to allocate a new texture with different layout.
//!

pub mod texture2d;
pub mod texture2d_compressed;
pub mod texture2darray;
pub mod texture2darray_compressed;
pub mod texture3d;
pub mod texture3d_compressed;
pub mod texture_cubemap;

use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap};
use log::debug;
use uuid::Uuid;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
        Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlTexture,
};

use crate::lru::{Lru, LruNode};

use self::{
    texture2d::Texture2D, texture2d_compressed::Texture2DCompressed,
    texture2darray::Texture2DArray, texture2darray_compressed::Texture2DArrayCompressed,
    texture3d::Texture3D, texture3d_compressed::Texture3DCompressed,
    texture_cubemap::TextureCubeMap,
};

use super::{
    capabilities::Capabilities, conversion::ToGlEnum, error::Error,
    utils::pixel_unpack_buffer_binding,
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

/// Available texture uncompressed internal formats mapped from [`WebGl2RenderingContext`].
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

/// Available texture compressed formats mapped from [`WebGl2RenderingContext`].
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
    COMPARE_FUNC(TextureCompareFunction),
    COMPARE_MODE(TextureCompareMode),
    BASE_LEVEL(i32),
    MAX_LEVEL(i32),
    MAX_LOD(f32),
    MIN_LOD(f32),
}

/// Calculates max available mipmap level for a specified size in Rounding Down mode.
pub fn max_available_mipmap_level(width: usize, height: usize) -> usize {
    (width as f64).max(height as f64).max(1.0).log2().floor() as usize
}

macro_rules! texture_sources_uncompressed {
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
        pub enum TextureSourceUncompressed {
            Function {
                width: usize,
                height: usize,
                callback: Rc<RefCell<dyn Fn() -> TextureSourceUncompressed>>,
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

        impl TextureSourceUncompressed {
            fn pixel_storages(&self, gl: &WebGl2RenderingContext) {
                match self {
                    TextureSourceUncompressed::PixelBufferObject { pixel_storages, .. }
                    | TextureSourceUncompressed::Binary { pixel_storages, .. }
                    | TextureSourceUncompressed::Uint16Array { pixel_storages, .. }
                    | TextureSourceUncompressed::Uint32Array { pixel_storages, .. }
                    $(
                        $(
                            | TextureSourceUncompressed::$html_name { pixel_storages, .. }
                        )+
                        $(
                            | TextureSourceUncompressed::$buffer_name { pixel_storages, .. }
                        )+
                    )+
                    => {
                        // setups pixel storage parameters
                        pixel_storages
                            .iter()
                            .for_each(|param| gl.pixel_storei(param.key(), param.value()));
                    }
                    TextureSourceUncompressed::Function { .. } => {}
                };
            }
        }

        impl TextureSource for TextureSourceUncompressed {
            /// Returns the width of the texture source.
            fn width(&self) -> usize {
                match self {
                    TextureSourceUncompressed::Function { width, .. } => *width,
                    TextureSourceUncompressed::PixelBufferObject { width, .. } => *width,
                    TextureSourceUncompressed::Binary { width, .. } => *width,
                    TextureSourceUncompressed::Uint16Array { width, .. } => *width,
                    TextureSourceUncompressed::Uint32Array { width, .. } => *width,
                    $(
                        $(
                            TextureSourceUncompressed::$html_name { data, .. } => data.$html_width() as usize,
                        )+
                        $(
                            TextureSourceUncompressed::$buffer_name { width, .. } => *width,
                        )+
                    )+
                }
            }

            /// Returns the height of the texture source.
            fn height(&self) -> usize {
                match self {
                    TextureSourceUncompressed::Function { height, .. } => *height,
                    TextureSourceUncompressed::PixelBufferObject { height, .. } => *height,
                    TextureSourceUncompressed::Binary { height, .. } => *height,
                    TextureSourceUncompressed::Uint16Array { height, .. } => *height,
                    TextureSourceUncompressed::Uint32Array { height, .. } => *height,
                    $(
                        $(
                            TextureSourceUncompressed::$html_name { data, .. } => data.$html_height() as usize,
                        )+
                        $(
                            TextureSourceUncompressed::$buffer_name { height, .. } => *height,
                        )+
                    )+
                }
            }

            fn tex_sub_image_2d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureTarget,
                level: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
            ) -> Result<(), Error> {
                self.pixel_storages(gl);
                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);

                // buffers image sub data
                let result = match self {
                    TextureSourceUncompressed::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceUncompressed::Function { .. } = source {
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if self.width() != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if self.height() != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
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
                    TextureSourceUncompressed::PixelBufferObject {
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
                            width as i32,
                            height as i32,
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
                    TextureSourceUncompressed::Binary {
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
                            TextureSourceUncompressed::$html_name {
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
                    TextureSourceUncompressed::Uint16Array { .. }
                    | TextureSourceUncompressed::Uint32Array { .. }
                    $(
                        $(
                            | TextureSourceUncompressed::$buffer_name { .. }
                        )+
                    )+
                    => {
                        let (data, format, data_type, src_offset) = match self {
                            TextureSourceUncompressed::Uint16Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            TextureSourceUncompressed::Uint32Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            $(
                                $(
                                    TextureSourceUncompressed::$buffer_name { data, format, src_offset, .. } => (data as &Object, format, $buffer_targe, src_offset),
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

                result.map_err(|err| Error::TexImageFailure(err.as_string()))
            }

            fn tex_sub_image_3d(
                &self,
                gl: &WebGl2RenderingContext,
                target: TextureTarget,
                level: usize,
                depth: usize,
                width: Option<usize>,
                height: Option<usize>,
                x_offset: Option<usize>,
                y_offset: Option<usize>,
                z_offset: Option<usize>,
            ) -> Result<(), Error> {
                self.pixel_storages(gl);
                let width = width.unwrap_or_else(|| self.width());
                let height = height.unwrap_or_else(|| self.height());
                let x_offset = x_offset.unwrap_or(0);
                let y_offset = y_offset.unwrap_or(0);
                let z_offset = z_offset.unwrap_or(0);

                // buffers image sub data
                let result = match self {
                    TextureSourceUncompressed::Function {
                        callback,
                        ..
                    } => {
                        let source = callback.borrow_mut()();
                        if let TextureSourceUncompressed::Function { .. } = source {
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if self.width() != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if self.height() != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
                        }
                        source.tex_sub_image_3d(gl, target, level, depth, Some(width), Some(height), Some(x_offset), Some(y_offset), Some(z_offset))?;
                        Ok(())
                    }
                    TextureSourceUncompressed::PixelBufferObject {
                        buffer,
                        format,
                        data_type,
                        pbo_offset,
                        ..
                    } => {
                        let bound = pixel_unpack_buffer_binding(gl);
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
                            bound.as_ref(),
                        );
                        result
                    }
                    TextureSourceUncompressed::Binary {
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
                            TextureSourceUncompressed::$html_name {
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
                    TextureSourceUncompressed::Uint16Array { .. }
                    | TextureSourceUncompressed::Uint32Array { .. }
                    $(
                        $(
                            | TextureSourceUncompressed::$buffer_name { .. }
                        )+
                    )+
                    => {
                        let (data, format, data_type, src_offset) = match self {
                            TextureSourceUncompressed::Uint16Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            TextureSourceUncompressed::Uint32Array { data, format, data_type, src_offset, .. } => (data as &Object, format, data_type.gl_enum(), src_offset),
                            $(
                                $(
                                    TextureSourceUncompressed::$buffer_name { data, format, src_offset, .. } => (data as &Object, format, $buffer_targe, src_offset),
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

                result.map_err(|err| Error::TexImageFailure(err.as_string()))
            }
        }
    }
}

texture_sources_uncompressed! {
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
                compressed_format: TextureCompressedFormat,
                bytes_length: usize,
                callback: Rc<RefCell<dyn Fn() -> TextureSourceCompressed>>,
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
            /// Returns the compressed format of the texture source.
            pub fn compressed_format(&self) -> TextureCompressedFormat {
                match self {
                    TextureSourceCompressed::Function { compressed_format, .. } => *compressed_format,
                    TextureSourceCompressed::PixelBufferObject { compressed_format, .. } => *compressed_format,
                    $(
                        TextureSourceCompressed::$name {
                            compressed_format,
                            ..
                        } => *compressed_format,
                    )+
                }
            }

            fn bytes_length(&self) -> usize {
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
        }

        impl TextureSource for TextureSourceCompressed {
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
                target: TextureTarget,
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
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if self.width() != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if self.height() != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
                        }
                        if self.bytes_length() != source.bytes_length() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
                        }
                        if self.compressed_format() != source.compressed_format() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
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
                        let current_buffer = pixel_unpack_buffer_binding(gl);
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
                            current_buffer.as_ref(),
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
                target: TextureTarget,
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
                            panic!("recursive TextureSource::Function is not allowed");
                        }
                        if self.width() != source.width() {
                            panic!("source returned from TextureSource::Function should have same width");
                        }
                        if self.height() != source.height() {
                            panic!("source returned from TextureSource::Function should have same height");
                        }
                        if self.bytes_length() != source.bytes_length() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
                        }
                        if self.compressed_format() != source.compressed_format() {
                            panic!("source returned from TextureSource::Function should have same bytes length");
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
                        let bound = pixel_unpack_buffer_binding(gl);
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
                            bound.as_ref(),
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

trait TextureSource {
    /// Returns the width of the texture source.
    fn width(&self) -> usize;

    /// Returns the height of the texture source.
    fn height(&self) -> usize;

    /// Uploads data to 2d or cube map texture source to WebGL.
    fn tex_sub_image_2d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Result<(), Error>;

    /// Uploads data to 3d or 2d array map texture source to WebGL.
    fn tex_sub_image_3d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        depth: usize,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<(), Error>;
}

/// Configurations specify a [`TextureSource`] to upload and
/// a target sub-rectangle in a mipmap level to replace with in the texture.
pub struct TextureUpload<T> {
    source: T,
    level: usize,
    depth: usize,
    width: Option<usize>,
    height: Option<usize>,
    x_offset: Option<usize>,
    y_offset: Option<usize>,
    z_offset: Option<usize>,
}

#[allow(private_bounds)]
impl<T> TextureUpload<T>
where
    T: TextureSource,
{
    /// Constructs a new upload texture data for 2d texture in mipmap level.
    pub fn new_2d(source: T, level: usize) -> Self {
        Self::with_params_2d(source, level, None, None, None, None)
    }

    /// Constructs a new upload texture data for 2d texture with custom parameters.
    pub fn with_params_2d(
        source: T,
        level: usize,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Self {
        Self {
            source,
            level,
            depth: 0,
            width,
            height,
            x_offset,
            y_offset,
            z_offset: None,
        }
    }
    /// Constructs a new upload texture data for 3d texture in mipmap level.
    pub fn new_3d(source: T, level: usize, depth: usize) -> Self {
        Self::with_params_3d(source, level, depth, None, None, None, None, None)
    }

    /// Constructs a new upload texture data for 3d texture with custom parameters.
    pub fn with_params_3d(
        source: T,
        level: usize,
        depth: usize,
        width: Option<usize>,
        height: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Self {
        Self {
            source,
            level,
            depth,
            width,
            height,
            x_offset,
            y_offset,
            z_offset,
        }
    }

    /// Returns [`TextureSource`].
    pub fn source(&self) -> &T {
        &self.source
    }

    /// Returns mipmap level to upload to.
    pub fn level(&self) -> usize {
        self.level
    }

    /// Returns sub-rectangle width to replace with in texture.
    /// If not specified, the whole width of the texture source will be used.
    pub fn width(&self) -> usize {
        self.width.unwrap_or_else(|| self.source.width())
    }

    /// Returns sub-rectangle height to replace with in texture.
    /// If not specified, the whole height of the texture source will be used.
    pub fn height(&self) -> usize {
        self.height.unwrap_or_else(|| self.source.height())
    }

    /// Returns sub-rectangle x offset. No offset applied if not specified.
    pub fn x_offset(&self) -> usize {
        self.x_offset.unwrap_or(0)
    }

    /// Returns sub-rectangle y offset. No offset applied if not specified.
    pub fn y_offset(&self) -> usize {
        self.y_offset.unwrap_or(0)
    }

    /// Uploads texture source to WebGL.
    pub fn tex_sub_image_2d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
    ) -> Result<(), Error> {
        self.source.tex_sub_image_2d(
            gl,
            target,
            self.level,
            self.width,
            self.height,
            self.x_offset,
            self.y_offset,
        )
    }

    /// Uploads texture source to WebGL.
    pub fn tex_sub_image_3d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
    ) -> Result<(), Error> {
        self.source.tex_sub_image_3d(
            gl,
            target,
            self.level,
            self.depth,
            self.width,
            self.height,
            self.x_offset,
            self.y_offset,
            self.z_offset,
        )
    }
}

struct Runtime {
    id: Uuid,
    gl: WebGl2RenderingContext,
    store_id: Uuid,
    texture: WebGlTexture,
    bytes_length: usize,
    lru_node: *mut LruNode<Uuid>,
    using: bool,

    used_memory: *mut usize,
    textures: *mut HashMap<Uuid, TextureKind>,
    lru: *mut Lru<Uuid>,
}

pub struct TextureDescriptor<T>(Rc<RefCell<T>>);

impl<T> Clone for TextureDescriptor<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

enum TextureKind {
    Texture2D(Weak<RefCell<Texture2D>>),
    Texture2DCompressed(Weak<RefCell<Texture2DCompressed>>),
    Texture2DArray(Weak<RefCell<Texture2DArray>>),
    Texture2DArrayCompressed(Weak<RefCell<Texture2DArrayCompressed>>),
    Texture3D(Weak<RefCell<Texture3D>>),
    Texture3DCompressed(Weak<RefCell<Texture3DCompressed>>),
    TextureCubeMap(Weak<RefCell<TextureCubeMap>>),
}

pub struct TextureStore {
    id: Uuid,
    gl: WebGl2RenderingContext,
    capabilities: Capabilities,
    available_memory: usize,
    used_memory: *mut usize,
    lru: *mut Lru<Uuid>,
    textures: *mut HashMap<Uuid, TextureKind>,
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
            textures: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),
        }
    }
}

macro_rules! impl_texture_store {
    ($((
        $texture:tt,
        $target:expr,
        $kind:ident,
        $use_func: ident,
        $unuse_func:ident
    ))+) => {
        impl TextureStore {
            $(
                pub fn $use_func(
                    &mut self,
                    descriptor: &TextureDescriptor<$texture>,
                    unit: TextureUnit,
                ) -> Result<WebGlTexture, Error> {
                    self.capabilities.verify_texture_unit(unit)?;

                    let texture = unsafe {
                        let mut t = descriptor.texture_mut();

                        let texture = match t.runtime.as_mut() {
                            Some(runtime) => {
                                if runtime.store_id != self.id {
                                    panic!("share texture descriptor between texture store is not allowed");
                                }

                                runtime.using = true;
                                (*self.lru).cache(runtime.lru_node);

                                runtime.texture.clone()
                            }
                            None => {
                                let texture = t.create_texture(&self.gl, &self.capabilities)?;

                                let id = Uuid::new_v4();
                                let lru_node = LruNode::new(id);
                                let bytes_length = t.bytes_length();
                                (*self.textures).insert(id, TextureKind::$kind(Rc::downgrade(&descriptor.0)));
                                (*self.lru).cache(lru_node);
                                (*self.used_memory) += bytes_length;
                                t.runtime = Some(Box::new(Runtime {
                                    id,
                                    gl: self.gl.clone(),
                                    store_id: self.id,
                                    texture: texture.clone(),
                                    bytes_length,
                                    lru_node,
                                    using: true,

                                    used_memory: self.used_memory,
                                    textures: self.textures,
                                    lru: self.lru,
                                }));

                                debug!(
                                    target: "TextureBuffer",
                                    "create new texture for {}",
                                    id,
                                );

                                texture
                            }
                        };

                        t.tex()?;
                        texture
                    };

                    self.free();
                    Ok(texture)
                }

                pub fn $unuse_func(&mut self, descriptor: &TextureDescriptor<$texture>) {
                    let mut inner = descriptor.0.borrow_mut();

                    if let Some(runtime) = inner.runtime.as_mut() {
                        runtime.using = false;
                    }
                }
            )+

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
                        let mut kind = occupied.get();

                        let released = match &mut kind {
                            $(
                                TextureKind::$kind(texture) => {
                                    let Some(texture) = texture.upgrade() else {
                                        occupied.remove();
                                        next_node = (*current_node).more_recently();
                                        continue;
                                    };
                                    let mut texture = texture.borrow_mut();
                                    let runtime = texture.runtime.as_deref_mut().unwrap();

                                    // skips if using
                                    if runtime.using {
                                        next_node = (*current_node).more_recently();
                                        continue;
                                    }

                                    // let texture takes free procedure itself.
                                    texture.free()
                                },
                            )+
                        };

                        if released {
                            let kind = occupied.remove();
                            let runtime = match kind {
                                $(
                                    TextureKind::$kind(texture) => {
                                        let texture = texture.upgrade().unwrap();
                                        let mut texture = texture.borrow_mut();
                                        texture.runtime.take().unwrap()
                                    },
                                )+
                            };
                            // reduces used memory
                            (*self.used_memory) -= runtime.bytes_length;
                            // removes LRU
                            (*self.lru).remove(runtime.lru_node);
                        }

                        next_node = (*current_node).more_recently();
                    }
                }
            }
        }

        impl Drop for TextureStore {
            fn drop(&mut self) {
                unsafe {
                    for (_, kind) in (*self.textures).iter_mut() {
                        let runtime = match kind {
                            $(
                                TextureKind::$kind(texture) => texture
                                .upgrade()
                                .and_then(|t| t.borrow_mut().runtime.take()),
                            )+
                        };
                        let Some(runtime) = runtime else {
                            continue;
                        };

                        self.gl.delete_texture(Some(&runtime.texture));
                        // store dropped, no need to update LRU anymore
                    }
                    drop(Box::from_raw(self.textures));
                    drop(Box::from_raw(self.used_memory));
                    drop(Box::from_raw(self.lru));
                }
            }
        }
    };
}

impl_texture_store! {
    (
        Texture2D,
        WebGl2RenderingContext::TEXTURE_2D,
        Texture2D,
        use_texture_2d,
        unuse_texture_2d
    )
    (
        Texture2DCompressed,
        WebGl2RenderingContext::TEXTURE_2D,
        Texture2DCompressed,
        use_texture_2d_compressed,
        unuse_texture_2d_compressed
    )
    (
        Texture3D,
        WebGl2RenderingContext::TEXTURE_3D,
        Texture3D,
        use_texture_3d,
        unuse_texture_3d
    )
    (
        Texture3DCompressed,
        WebGl2RenderingContext::TEXTURE_3D,
        Texture3DCompressed,
        use_texture_3d_compressed,
        unuse_texture_3d_compressed
    )
    (
        Texture2DArray,
        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
        Texture2DArray,
        use_texture_2d_array,
        unuse_texture_2d_array
    )
    (
        Texture2DArrayCompressed,
        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
        Texture2DArrayCompressed,
        use_texture_2d_array_compressed,
        unuse_texture_2d_array_compressed
    )
    (
        TextureCubeMap,
        TextureCubeMap::TEXTURE_CUBE_MAP,
        TextureCubeMap,
        use_texture_cube_map,
        unuse_texture_cube_map
    )
}
