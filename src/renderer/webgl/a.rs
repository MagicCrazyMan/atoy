use std::{borrow::Cow, cell::RefCell, fmt::Debug, hash::Hash, rc::Rc};

use hashbrown::HashSet;
use js_sys::{
    DataView, Float32Array, Int16Array, Int32Array, Int8Array, Uint16Array, Uint32Array,
    Uint8Array, Uint8ClampedArray,
};
use smallvec::SmallVec;
use uuid::Uuid;
use web_sys::{
    ExtTextureFilterAnisotropic, HtmlCanvasElement, HtmlImageElement, HtmlVideoElement,
    ImageBitmap, ImageData, WebGl2RenderingContext, WebGlBuffer, WebGlSampler, WebGlTexture,
};

use super::{
    capabilities::{Capabilities, EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC},
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
pub enum TextureUncompressedFormat {
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

impl TextureUncompressedFormat {
    /// Calculates the bytes length of of a specified internal format in specified size.
    pub fn byte_length(&self, width: usize, height: usize) -> usize {
        match self {
            TextureUncompressedFormat::RGBA32I => width * height * 16,
            TextureUncompressedFormat::RGBA32UI => width * height * 16,
            TextureUncompressedFormat::RGBA16I => width * height * 4,
            TextureUncompressedFormat::RGBA16UI => width * height * 4,
            TextureUncompressedFormat::RGBA8 => width * height * 4,
            TextureUncompressedFormat::RGBA8I => width * height * 4,
            TextureUncompressedFormat::RGBA8UI => width * height * 4,
            TextureUncompressedFormat::SRGB8_ALPHA8 => width * height * 4,
            TextureUncompressedFormat::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureUncompressedFormat::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
            TextureUncompressedFormat::RGBA4 => width * height * 2,
            TextureUncompressedFormat::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
            TextureUncompressedFormat::RGB8 => width * height * 3,
            TextureUncompressedFormat::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
            TextureUncompressedFormat::RG32I => width * height * 4,
            TextureUncompressedFormat::RG32UI => width * height * 4,
            TextureUncompressedFormat::RG16I => width * height * 4,
            TextureUncompressedFormat::RG16UI => width * height * 4,
            TextureUncompressedFormat::RG8 => width * height * 2,
            TextureUncompressedFormat::RG8I => width * height * 2,
            TextureUncompressedFormat::RG8UI => width * height * 2,
            TextureUncompressedFormat::R32I => width * height * 4,
            TextureUncompressedFormat::R32UI => width * height * 4,
            TextureUncompressedFormat::R16I => width * height * 2,
            TextureUncompressedFormat::R16UI => width * height * 2,
            TextureUncompressedFormat::R8 => width * height * 1,
            TextureUncompressedFormat::R8I => width * height * 1,
            TextureUncompressedFormat::R8UI => width * height * 1,
            TextureUncompressedFormat::RGBA32F => width * height * 16,
            TextureUncompressedFormat::RGBA16F => width * height * 4,
            TextureUncompressedFormat::RGBA8_SNORM => width * height * 4,
            TextureUncompressedFormat::RGB32F => width * height * 12,
            TextureUncompressedFormat::RGB32I => width * height * 12,
            TextureUncompressedFormat::RGB32UI => width * height * 12,
            TextureUncompressedFormat::RGB16F => width * height * 6,
            TextureUncompressedFormat::RGB16I => width * height * 6,
            TextureUncompressedFormat::RGB16UI => width * height * 6,
            TextureUncompressedFormat::RGB8_SNORM => width * height * 3,
            TextureUncompressedFormat::RGB8I => width * height * 3,
            TextureUncompressedFormat::RGB8UI => width * height * 3,
            TextureUncompressedFormat::SRGB8 => width * height * 3,
            TextureUncompressedFormat::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
            TextureUncompressedFormat::RGB9_E5 => width * height * 4, // 9 + 9 + 9 + 5 in bits
            TextureUncompressedFormat::RG32F => width * height * 4,
            TextureUncompressedFormat::RG16F => width * height * 4,
            TextureUncompressedFormat::RG8_SNORM => width * height * 2,
            TextureUncompressedFormat::R32F => width * height * 4,
            TextureUncompressedFormat::R16F => width * height * 2,
            TextureUncompressedFormat::R8_SNORM => width * height * 1,
            TextureUncompressedFormat::DEPTH_COMPONENT32F => width * height * 4,
            TextureUncompressedFormat::DEPTH_COMPONENT24 => width * height * 3,
            TextureUncompressedFormat::DEPTH_COMPONENT16 => width * height * 2,
            TextureUncompressedFormat::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
            TextureUncompressedFormat::DEPTH24_STENCIL8 => width * height * 4,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureInternalFormat {
    Uncompressed(TextureUncompressedFormat),
    Compressed(TextureCompressedFormat),
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

// impl TexturePixelStorage {
//     fn save(&self, gl: &WebGl2RenderingContext) -> Option<TexturePixelStorage> {
//         match self {
//             TexturePixelStorage::PACK_ALIGNMENT(_) => gl
//                 .texture_pixel_storage_pack_alignment()
//                 .map(|v| TexturePixelStorage::PACK_ALIGNMENT(v)),
//             TexturePixelStorage::PACK_ROW_LENGTH(_) => gl
//                 .texture_pixel_storage_pack_row_length()
//                 .map(|v| TexturePixelStorage::PACK_ROW_LENGTH(v)),
//             TexturePixelStorage::PACK_SKIP_PIXELS(_) => gl
//                 .texture_pixel_storage_pack_skip_pixels()
//                 .map(|v| TexturePixelStorage::PACK_SKIP_PIXELS(v)),
//             TexturePixelStorage::PACK_SKIP_ROWS(_) => gl
//                 .texture_pixel_storage_pack_skip_rows()
//                 .map(|v| TexturePixelStorage::PACK_SKIP_ROWS(v)),
//             TexturePixelStorage::UNPACK_ALIGNMENT(_) => gl
//                 .texture_pixel_storage_unpack_alignment()
//                 .map(|v| TexturePixelStorage::UNPACK_ALIGNMENT(v)),
//             TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(_) => gl
//                 .texture_pixel_storage_unpack_flip_y()
//                 .map(|v| TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v)),
//             TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(_) => gl
//                 .texture_pixel_storage_unpack_premultiply_alpha()
//                 .map(|v| TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v)),
//             TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(_) => gl
//                 .texture_pixel_storage_unpack_colorspace_conversion()
//                 .map(|v| TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v)),
//             TexturePixelStorage::UNPACK_ROW_LENGTH(_) => gl
//                 .texture_pixel_storage_unpack_row_length()
//                 .map(|v| TexturePixelStorage::UNPACK_ROW_LENGTH(v)),
//             TexturePixelStorage::UNPACK_IMAGE_HEIGHT(_) => gl
//                 .texture_pixel_storage_unpack_image_height()
//                 .map(|v| TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v)),
//             TexturePixelStorage::UNPACK_SKIP_PIXELS(_) => gl
//                 .texture_pixel_storage_unpack_skip_pixels()
//                 .map(|v| TexturePixelStorage::UNPACK_SKIP_PIXELS(v)),
//             TexturePixelStorage::UNPACK_SKIP_ROWS(_) => gl
//                 .texture_pixel_storage_unpack_skip_rows()
//                 .map(|v| TexturePixelStorage::UNPACK_SKIP_ROWS(v)),
//             TexturePixelStorage::UNPACK_SKIP_IMAGES(_) => gl
//                 .texture_pixel_storage_unpack_skip_images()
//                 .map(|v| TexturePixelStorage::UNPACK_SKIP_IMAGES(v)),
//         }
//     }

//     fn pixel_store(&self, gl: &WebGl2RenderingContext) {
//         match self {
//             TexturePixelStorage::PACK_ALIGNMENT(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, *v);
//             }
//             TexturePixelStorage::PACK_ROW_LENGTH(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, *v);
//             }
//             TexturePixelStorage::PACK_SKIP_PIXELS(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, *v);
//             }
//             TexturePixelStorage::PACK_SKIP_ROWS(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, *v);
//             }
//             TexturePixelStorage::UNPACK_ALIGNMENT(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, *v);
//             }
//             TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v) => {
//                 gl.pixel_storei(
//                     WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
//                     if *v { 1 } else { 0 },
//                 );
//             }
//             TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v) => {
//                 gl.pixel_storei(
//                     WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
//                     if *v { 1 } else { 0 },
//                 );
//             }
//             TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v) => {
//                 gl.pixel_storei(
//                     WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
//                     match v {
//                         TextureUnpackColorSpaceConversion::NONE => {
//                             WebGl2RenderingContext::NONE as i32
//                         }
//                         TextureUnpackColorSpaceConversion::BROWSER_DEFAULT_WEBGL => {
//                             WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32
//                         }
//                     },
//                 );
//             }
//             TexturePixelStorage::UNPACK_ROW_LENGTH(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, *v);
//             }
//             TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, *v);
//             }
//             TexturePixelStorage::UNPACK_SKIP_PIXELS(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, *v);
//             }
//             TexturePixelStorage::UNPACK_SKIP_ROWS(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, *v);
//             }
//             TexturePixelStorage::UNPACK_SKIP_IMAGES(v) => {
//                 gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, *v);
//             }
//         }
//     }
// }

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

// impl TextureParameter {
//     fn tex_parameter(
//         &self,
//         gl: &WebGl2RenderingContext,
//         target: TextureTarget,
//         capabilities: &Capabilities,
//     ) -> Result<(), Error> {
//         match self {
//             TextureParameter::BASE_LEVEL(v) => {
//                 gl.tex_parameteri(
//                     target.gl_enum(),
//                     WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
//                     *v,
//                 );
//             }
//             TextureParameter::MAX_LEVEL(v) => {
//                 gl.tex_parameteri(
//                     target.gl_enum(),
//                     WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
//                     *v,
//                 );
//             }
//             TextureParameter::MAX_ANISOTROPY(v) => {
//                 if !capabilities.texture_filter_anisotropic_supported() {
//                     return Err(Error::ExtensionUnsupported(
//                         EXTENSION_EXT_TEXTURE_FILTER_ANISOTROPIC,
//                     ));
//                 }
//                 gl.tex_parameterf(
//                     target.gl_enum(),
//                     ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT,
//                     *v,
//                 );
//             }
//         };
//         Ok(())
//     }
// }

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

// impl SamplerParameter {
//     fn sampler_parameter(&self, gl: &WebGl2RenderingContext, sampler: &WebGlSampler) {
//         match self {
//             SamplerParameter::MAG_FILTER(v) => {
//                 gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
//             }
//             SamplerParameter::MIN_FILTER(v) => {
//                 gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
//             }
//             SamplerParameter::WRAP_S(v)
//             | SamplerParameter::WRAP_T(v)
//             | SamplerParameter::WRAP_R(v) => {
//                 gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
//             }
//             SamplerParameter::COMPARE_FUNC(v) => {
//                 gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
//             }
//             SamplerParameter::COMPARE_MODE(v) => {
//                 gl.sampler_parameteri(&sampler, self.gl_enum(), v.gl_enum() as i32)
//             }
//             SamplerParameter::MAX_LOD(v) | SamplerParameter::MIN_LOD(v) => {
//                 gl.sampler_parameterf(&sampler, self.gl_enum(), *v)
//             }
//         }
//     }
// }

pub enum TextureData<'a> {
    Uncompressed {
        data: TextureUncompressedData<'a>,
        format: TextureFormat,
        pixel_storages: SmallVec<[TexturePixelStorage; 6]>,
    },
    Compressed {
        data: TextureCompressedData,
        format: TextureCompressedFormat,
    },
}

pub enum TextureUncompressedData<'a> {
    Bytes {
        width: usize,
        height: usize,
        data: Box<dyn AsRef<[u8]>>,
        data_type: TextureDataType,
        src_element_offset: Option<usize>,
    },
    BytesBorrowed {
        width: usize,
        height: usize,
        data: &'a [u8],
        data_type: TextureDataType,
        src_element_offset: Option<usize>,
    },
    PixelBufferObject {
        width: usize,
        height: usize,
        buffer: WebGlBuffer,
        data_type: TextureDataType,
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
        /// Only [`TextureDataType::UNSIGNED_SHORT`],
        /// [`TextureDataType::UNSIGNED_SHORT_5_6_5`],
        /// [`TextureDataType::UNSIGNED_SHORT_4_4_4_4`],
        /// [`TextureDataType::UNSIGNED_SHORT_5_5_5_1`],
        /// [`TextureDataType::HALF_FLOAT`] are accepted.
        data_type: TextureDataType,
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
        /// Only [`TextureDataType::UNSIGNED_INT`],
        /// [`TextureDataType::UNSIGNED_INT_24_8`]
        /// are accepted.
        data_type: TextureDataType,
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
        data_type: TextureDataType,
    },
    HtmlImageElement {
        data: HtmlImageElement,
        data_type: TextureDataType,
    },
    HtmlVideoElement {
        data: HtmlVideoElement,
        data_type: TextureDataType,
    },
    ImageData {
        data: ImageData,
        data_type: TextureDataType,
    },
    ImageBitmap {
        data: ImageBitmap,
        data_type: TextureDataType,
    },
}

pub enum TextureCompressedData {
    // Bytes {
    //     width: usize,
    //     height: usize,
    //     data: Vec<u8>,
    //     src_element_offset: Option<usize>,
    //     src_element_length_override: Option<usize>,
    // },
    // BytesBorrowed {
    //     width: usize,
    //     height: usize,
    //     data: &'a [u8],
    //     src_element_offset: Option<usize>,
    //     src_element_length_override: Option<usize>,
    // },
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

pub trait TextureSource {
    fn byte_length(&self) -> usize;

    fn data(&self) -> TextureData;

    fn generate_mipmap(&self) -> bool;
}

pub enum TextureLayout {
    Texture2D {
        width: usize,
        height: usize,
        levels: usize,
        format: TextureInternalFormat,
    },
    Texrure2DArray {
        width: usize,
        height: usize,
        levels: usize,
        len: usize,
        format: TextureInternalFormat,
    },
    Texture3D {
        width: usize,
        height: usize,
        levels: usize,
        depth: usize,
        format: TextureInternalFormat,
    },
    TextureCubeMap {
        width: usize,
        height: usize,
        levels: usize,
        format: TextureInternalFormat,
    },
}

struct TextureRuntime {
    gl: WebGl2RenderingContext,
    texture: Option<WebGlTexture>,
    sampler: Option<WebGlSampler>,
    bindings: HashSet<TextureTarget>,
}

struct TextureShared {
    id: Uuid,
    layout: TextureLayout,
    // memory_policy: MemoryPolicy,
    queue: Vec<Box<dyn TextureSource>>,
    // registered: Option<BufferRegistered>,
    runtime: Option<TextureRuntime>,
}

pub struct Texture {
    name: Cow<'static, str>,
    shared: Rc<RefCell<TextureShared>>,
}
