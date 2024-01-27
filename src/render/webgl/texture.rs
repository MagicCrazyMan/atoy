use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use hashbrown::HashMap;
use log::debug;
use serde::de;
use uuid::Uuid;
use web_sys::{
    js_sys::{Float32Array, Uint16Array, Uint32Array, Uint8Array},
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlBuffer, WebGlTexture,
};

use crate::lru::{Lru, LruNode};

use super::{
    abilities::Abilities, conversion::ToGlEnum, error::Error,
    utilities::pixel_unpack_buffer_binding,
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
    pub fn texel_channels(&self) -> usize {
        match self {
            TextureInternalFormat::RGBA32I => 4,
            TextureInternalFormat::RGBA32UI => 4,
            TextureInternalFormat::RGBA16I => 4,
            TextureInternalFormat::RGBA16UI => 4,
            TextureInternalFormat::RGBA8 => 4,
            TextureInternalFormat::RGBA8I => 4,
            TextureInternalFormat::RGBA8UI => 4,
            TextureInternalFormat::SRGB8_ALPHA8 => 4,
            TextureInternalFormat::RGB10_A2 => 4,
            TextureInternalFormat::RGB10_A2UI => 4,
            TextureInternalFormat::RGBA4 => 4,
            TextureInternalFormat::RGB5_A1 => 3,
            TextureInternalFormat::RGB8 => 3,
            TextureInternalFormat::RGB565 => 3,
            TextureInternalFormat::RG32I => 2,
            TextureInternalFormat::RG32UI => 2,
            TextureInternalFormat::RG16I => 2,
            TextureInternalFormat::RG16UI => 2,
            TextureInternalFormat::RG8 => 2,
            TextureInternalFormat::RG8I => 2,
            TextureInternalFormat::RG8UI => 2,
            TextureInternalFormat::R32I => 1,
            TextureInternalFormat::R32UI => 1,
            TextureInternalFormat::R16I => 1,
            TextureInternalFormat::R16UI => 1,
            TextureInternalFormat::R8 => 1,
            TextureInternalFormat::R8I => 1,
            TextureInternalFormat::R8UI => 1,
            TextureInternalFormat::RGBA32F => 4,
            TextureInternalFormat::RGBA16F => 4,
            TextureInternalFormat::RGBA8_SNORM => 4,
            TextureInternalFormat::RGB32F => 3,
            TextureInternalFormat::RGB32I => 3,
            TextureInternalFormat::RGB32UI => 3,
            TextureInternalFormat::RGB16F => 3,
            TextureInternalFormat::RGB16I => 3,
            TextureInternalFormat::RGB16UI => 3,
            TextureInternalFormat::RGB8_SNORM => 3,
            TextureInternalFormat::RGB8I => 3,
            TextureInternalFormat::RGB8UI => 3,
            TextureInternalFormat::SRGB8 => 3,
            TextureInternalFormat::R11F_G11F_B10F => 3,
            TextureInternalFormat::RGB9_E5 => 3,
            TextureInternalFormat::RG32F => 2,
            TextureInternalFormat::RG16F => 2,
            TextureInternalFormat::RG8_SNORM => 2,
            TextureInternalFormat::R32F => 1,
            TextureInternalFormat::R16F => 1,
            TextureInternalFormat::R8_SNORM => 1,
            TextureInternalFormat::DEPTH_COMPONENT32F => 1,
            TextureInternalFormat::DEPTH_COMPONENT24 => 1,
            TextureInternalFormat::DEPTH_COMPONENT16 => 1,
            TextureInternalFormat::DEPTH32F_STENCIL8 => 1,
            TextureInternalFormat::DEPTH24_STENCIL8 => 1,
        }
    }

    pub fn texel_bits(&self) -> usize {
        match self {
            TextureInternalFormat::RGBA32I => 128,
            TextureInternalFormat::RGBA32UI => 128,
            TextureInternalFormat::RGBA16I => 64,
            TextureInternalFormat::RGBA16UI => 64,
            TextureInternalFormat::RGBA8 => 32,
            TextureInternalFormat::RGBA8I => 32,
            TextureInternalFormat::RGBA8UI => 32,
            TextureInternalFormat::SRGB8_ALPHA8 => 32,
            TextureInternalFormat::RGB10_A2 => 32, // 10 + 10 + 10 + 2 in bits
            TextureInternalFormat::RGB10_A2UI => 32, // 10 + 10 + 10 + 2 in bits
            TextureInternalFormat::RGBA4 => 16,
            TextureInternalFormat::RGB5_A1 => 16, // 5 + 5 + 5 + 1 in bits
            TextureInternalFormat::RGB8 => 24,
            TextureInternalFormat::RGB565 => 16, // 5 + 6 + 5 in bits
            TextureInternalFormat::RG32I => 64,
            TextureInternalFormat::RG32UI => 64,
            TextureInternalFormat::RG16I => 32,
            TextureInternalFormat::RG16UI => 32,
            TextureInternalFormat::RG8 => 16,
            TextureInternalFormat::RG8I => 16,
            TextureInternalFormat::RG8UI => 16,
            TextureInternalFormat::R32I => 32,
            TextureInternalFormat::R32UI => 32,
            TextureInternalFormat::R16I => 16,
            TextureInternalFormat::R16UI => 16,
            TextureInternalFormat::R8 => 8,
            TextureInternalFormat::R8I => 8,
            TextureInternalFormat::R8UI => 8,
            TextureInternalFormat::RGBA32F => 128,
            TextureInternalFormat::RGBA16F => 64,
            TextureInternalFormat::RGBA8_SNORM => 32,
            TextureInternalFormat::RGB32F => 96,
            TextureInternalFormat::RGB32I => 96,
            TextureInternalFormat::RGB32UI => 96,
            TextureInternalFormat::RGB16F => 48,
            TextureInternalFormat::RGB16I => 48,
            TextureInternalFormat::RGB16UI => 48,
            TextureInternalFormat::RGB8_SNORM => 24,
            TextureInternalFormat::RGB8I => 24,
            TextureInternalFormat::RGB8UI => 24,
            TextureInternalFormat::SRGB8 => 24,
            TextureInternalFormat::R11F_G11F_B10F => 32, // 11 + 11 + 10 in bits
            TextureInternalFormat::RGB9_E5 => 32,        // 9 + 9 + 9 + 5 in bits
            TextureInternalFormat::RG32F => 64,
            TextureInternalFormat::RG16F => 32,
            TextureInternalFormat::RG8_SNORM => 16,
            TextureInternalFormat::R32F => 32,
            TextureInternalFormat::R16F => 16,
            TextureInternalFormat::R8_SNORM => 8,
            TextureInternalFormat::DEPTH_COMPONENT32F => 32,
            TextureInternalFormat::DEPTH_COMPONENT24 => 24,
            TextureInternalFormat::DEPTH_COMPONENT16 => 16,
            TextureInternalFormat::DEPTH32F_STENCIL8 => 40, // 32 + 8 in bits
            TextureInternalFormat::DEPTH24_STENCIL8 => 32,  // 24 + 8 in bits
        }
    }

    /// Returns the number of bytes per texel consumed.
    pub fn texel_bytes(&self) -> usize {
        self.texel_bits() / 8
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

/// Memory freeing policies.
pub enum MemoryPolicy {
    Unfree,
    Restorable(Rc<RefCell<dyn Fn() -> TextureSource>>),
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self::Unfree
    }
}

pub enum TextureSource {
    Preallocate {
        width: usize,
        height: usize,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
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
        data: Uint8Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Int8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
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
        data: Uint16Array,
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
        data: Uint32Array,
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

impl TextureSource {
    pub fn width(&self) -> usize {
        match self {
            TextureSource::Preallocate { width, .. }
            | TextureSource::PixelBufferObject { width, .. }
            | TextureSource::Binary { width, .. }
            | TextureSource::Uint8Array { width, .. }
            | TextureSource::Uint8ClampedArray { width, .. }
            | TextureSource::Int8Array { width, .. }
            | TextureSource::Uint16Array { width, .. }
            | TextureSource::Int16Array { width, .. }
            | TextureSource::Uint32Array { width, .. }
            | TextureSource::Int32Array { width, .. }
            | TextureSource::Float32Array { width, .. } => *width,
            TextureSource::HtmlCanvasElement {
                canvas,
                custom_size,
                ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(canvas.width() as usize),
            TextureSource::HtmlImageElement {
                image, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(image.natural_width() as usize),
            TextureSource::HtmlVideoElement {
                video, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(video.video_width() as usize),
            TextureSource::ImageData {
                data, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(data.width() as usize),
            TextureSource::ImageBitmap {
                bitmap,
                custom_size,
                ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(bitmap.width() as usize),
        }
    }

    pub fn height(&self) -> usize {
        match self {
            TextureSource::Preallocate { height, .. }
            | TextureSource::PixelBufferObject { height, .. }
            | TextureSource::Binary { height, .. }
            | TextureSource::Uint8Array { height, .. }
            | TextureSource::Uint8ClampedArray { height, .. }
            | TextureSource::Int8Array { height, .. }
            | TextureSource::Uint16Array { height, .. }
            | TextureSource::Int16Array { height, .. }
            | TextureSource::Uint32Array { height, .. }
            | TextureSource::Int32Array { height, .. }
            | TextureSource::Float32Array { height, .. } => *height,
            TextureSource::HtmlCanvasElement {
                canvas,
                custom_size,
                ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(canvas.height() as usize),
            TextureSource::HtmlImageElement {
                image, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(image.natural_height() as usize),
            TextureSource::HtmlVideoElement {
                video, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(video.video_height() as usize),
            TextureSource::ImageData {
                data, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(data.height() as usize),
            TextureSource::ImageBitmap {
                bitmap,
                custom_size,
                ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(bitmap.height() as usize),
        }
    }

    pub fn pixel_storages(&self) -> &[TexturePixelStorage] {
        match self {
            TextureSource::Preallocate { pixel_storages, .. }
            | TextureSource::PixelBufferObject { pixel_storages, .. }
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
            | TextureSource::ImageBitmap { pixel_storages, .. } => &pixel_storages,
        }
    }

    fn tex_sub_image_2d(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image sub data
        let result = match self {
            TextureSource::Preallocate {
                width,
                height,
                format,
                data_type,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                None
            ),
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
                    *pbo_offset as i32
                );
                gl.bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, current_buffer.as_ref());
                result
            },
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
                *src_offset  as u32
            ),
            TextureSource::Uint8Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                data,
                *src_offset  as u32
            ),
            TextureSource::Uint8ClampedArray {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                data,
                *src_offset  as u32
            ),
            TextureSource::Int8Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::BYTE,
                data,
                *src_offset  as u32
            ),
            TextureSource::Uint16Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data,
                *src_offset  as u32
            ),
            TextureSource::Int16Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::SHORT,
                data,
                *src_offset  as u32
            ),
            TextureSource::Uint32Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data,
                *src_offset  as u32
            ),
            TextureSource::Int32Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::INT,
                data,
                *src_offset  as u32
            ),
            TextureSource::Float32Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::FLOAT,
                data,
                *src_offset  as u32
            ),
            TextureSource::HtmlCanvasElement {
                format,
                data_type,
                canvas,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
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
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
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
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
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
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
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
                custom_size ,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
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
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
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
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
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
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
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
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image sub data
        let result = match self {
            TextureSource::Preallocate {
                width,
                height,
                format,
                data_type,
                ..
            } => gl.tex_sub_image_3d_with_opt_u8_array(
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
                None,
            ),
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
            TextureSource::Uint8Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Uint8ClampedArray {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Int8Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::BYTE,
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Uint16Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
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
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Int16Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::SHORT,
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Uint32Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
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
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Int32Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::INT,
                Some(data),
                *src_offset as u32,
            ),
            TextureSource::Float32Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_3d_with_opt_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                z_offset as i32,
                *width as i32,
                *height as i32,
                depth as i32,
                format.gl_enum(),
                WebGl2RenderingContext::FLOAT,
                Some(data),
                *src_offset as u32,
            ),
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

struct Runtime<T> {
    id: Uuid,
    gl: WebGl2RenderingContext,
    store_id: Uuid,
    texture: WebGlTexture,
    memory: usize,
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
            (*self.used_memory) -= self.memory;
            self.gl.delete_texture(Some(&self.texture));
        }
    }
}

pub struct Texture2D {
    width: usize,
    height: usize,
    queue: Vec<(TextureSource, usize, usize, usize)>,
}

pub struct Texture3D {
    width: usize,
    height: usize,
    depth: usize,
    queue: Vec<(TextureSource, usize, usize, usize, usize, usize)>,
}

pub struct Texture2DArray {
    width: usize,
    height: usize,
    array_length: usize,
    queue: Vec<(TextureSource, usize, usize, usize, usize, usize)>,
}

pub struct TextureCubeMap {
    width: usize,
    height: usize,
    positive_x: Vec<(TextureSource, usize, usize, usize)>,
    negative_x: Vec<(TextureSource, usize, usize, usize)>,
    positive_y: Vec<(TextureSource, usize, usize, usize)>,
    negative_y: Vec<(TextureSource, usize, usize, usize)>,
    positive_z: Vec<(TextureSource, usize, usize, usize)>,
    negative_z: Vec<(TextureSource, usize, usize, usize)>,
}

struct TextureDescriptorInner<T> {
    name: Option<Cow<'static, str>>,
    layout: T,
    internal_format: TextureInternalFormat,
    generate_mipmap: bool,
    memory_policy: MemoryPolicy,

    runtime: Option<Box<Runtime<T>>>,
}

impl TextureDescriptorInner<Texture2D> {
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

    fn used_memory(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        let texel_bytes = self.internal_format.texel_bytes();
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += width * height * texel_bytes;
        }
        used_memory
    }

    fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };
        let texel_bytes = self.internal_format.texel_bytes();

        Some(width * height * texel_bytes)
    }

    fn verify_size_tex_image(
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

    fn verify_size_tex_sub_image(
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

impl TextureDescriptorInner<Texture3D> {
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

    fn used_memory(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        let texel_bytes = self.internal_format.texel_bytes();
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            let depth = self.depth_of_level(level).unwrap();
            used_memory += width * height * depth * texel_bytes;
        }
        used_memory
    }

    fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };
        let Some(depth) = self.depth_of_level(level) else {
            return None;
        };
        let texel_bytes = self.internal_format.texel_bytes();

        Some(width * height * depth * texel_bytes)
    }

    fn verify_size_tex_image(
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

    fn verify_size_tex_sub_image(
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

    fn used_memory(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        let array_length = self.layout.array_length;
        let texel_bytes = self.internal_format.texel_bytes();
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += width * height * texel_bytes * array_length;
        }
        used_memory
    }

    fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };

        Some(width * height * self.internal_format.texel_bytes() * self.layout.array_length)
    }

    fn verify_size_tex_image(
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
        if depth > self.layout.array_length {
            return Err(Error::TextureSizeMismatched);
        }

        Ok(())
    }

    fn verify_size_tex_sub_image(
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
        if depth > self.layout.array_length {
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

    fn used_memory(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        let texel_bytes = self.internal_format.texel_bytes();
        for level in 0..=self.max_mipmap_level() {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += width * height * texel_bytes;
        }
        used_memory
    }

    fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };
        let texel_bytes = self.internal_format.texel_bytes();

        Some(width * height * texel_bytes * 6)
    }

    fn verify_size_tex_image(
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

    fn verify_size_tex_sub_image(
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

    pub fn memory_policy(&self) -> Ref<MemoryPolicy> {
        Ref::map(self.0.borrow(), |inner| &inner.memory_policy)
    }

    pub fn generate_mipmap(&self) -> bool {
        self.0.borrow().generate_mipmap
    }

    pub fn internal_format(&self) -> TextureInternalFormat {
        self.0.borrow().internal_format
    }
}

impl TextureDescriptor<Texture2D> {
    pub fn new(
        width: usize,
        height: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2D {
                width,
                height,
                queue: Vec::new(),
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2D {
                width: source.width(),
                height: source.height(),
                queue: vec![(source, 0, 0, 0)],
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
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

    pub fn used_memory(&self) -> usize {
        self.0.borrow().used_memory()
    }

    pub fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().used_memory_of_level(level)
    }

    pub fn tex_image(&mut self, source: TextureSource, level: usize) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_size_tex_image(level, width, height)?;

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
        inner.verify_size_tex_sub_image(level, width, height, x_offset, y_offset)?;

        inner.layout.queue.push((source, level, x_offset, y_offset));
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
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3D {
                width,
                height,
                depth,
                queue: Vec::new(),
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        depth: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture3D {
                width: source.width(),
                height: source.height(),
                depth,
                queue: vec![(source, 0, depth, 0, 0, 0)],
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
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

    pub fn used_memory(&self) -> usize {
        self.0.borrow().used_memory()
    }

    pub fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().used_memory_of_level(level)
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
        inner.verify_size_tex_image(level, width, height, depth)?;

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
        inner
            .verify_size_tex_sub_image(level, width, height, depth, x_offset, y_offset, z_offset)?;

        inner
            .layout
            .queue
            .push((source, level, depth, x_offset, y_offset, z_offset));
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
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DArray {
                width,
                height,
                array_length,
                queue: Vec::new(),
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        source_array_index: usize,
        array_length: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: Texture2DArray {
                width: source.width(),
                height: source.height(),
                array_length,
                queue: vec![(source, 0, source_array_index, 0, 0, 0)],
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        })))
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

    pub fn used_memory(&self) -> usize {
        self.0.borrow().used_memory()
    }

    pub fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().used_memory_of_level(level)
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
        inner.verify_size_tex_image(level, width, height, array_index)?;

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
        inner.verify_size_tex_sub_image(
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
        memory_policy: MemoryPolicy,
    ) -> Result<Self, Error> {
        if width != height {
            return Err(Error::TextureCubeMapWidthAndHeightNotEqual);
        }

        Ok(Self(Rc::new(RefCell::new(TextureDescriptorInner {
            name: None,
            layout: TextureCubeMap {
                width,
                height,
                positive_x: Vec::new(),
                negative_x: Vec::new(),
                positive_y: Vec::new(),
                negative_y: Vec::new(),
                positive_z: Vec::new(),
                negative_z: Vec::new(),
            },
            internal_format,
            generate_mipmap,
            memory_policy,

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
        memory_policy: MemoryPolicy,
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
                positive_x: vec![(positive_x, 0, 0, 0)],
                negative_x: vec![(negative_x, 0, 0, 0)],
                positive_y: vec![(positive_y, 0, 0, 0)],
                negative_y: vec![(negative_y, 0, 0, 0)],
                positive_z: vec![(positive_z, 0, 0, 0)],
                negative_z: vec![(negative_z, 0, 0, 0)],
            },
            internal_format,
            generate_mipmap,
            memory_policy,

            runtime: None,
        }))))
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

    pub fn used_memory(&self) -> usize {
        self.0.borrow().used_memory()
    }

    pub fn used_memory_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().used_memory_of_level(level)
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
                    inner.verify_size_tex_image(level, width, height)?;
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
                    inner.verify_size_tex_sub_image(level, width, height, x_offset, y_offset)?;
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
    descriptors_2d_array: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture2DArray>>>>,
    descriptors_3d: *mut HashMap<Uuid, Weak<RefCell<TextureDescriptorInner<Texture3D>>>>,
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
            descriptors_2d_array: Box::leak(Box::new(HashMap::new())),
            descriptors_3d: Box::leak(Box::new(HashMap::new())),
            descriptors_cube_map: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),
        }
    }
}

macro_rules! store_use_textures {
    ($((
        $layout:tt,
        $target:expr,
        $descriptors:ident,
        $tex_func:ident, $(($tex_queue:ident, $tex_target:expr, $($tex_params:ident),+)),+,
        $tex_storage_func:ident, ($($tex_storage_params:tt),+),
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
                                    inner.internal_format.gl_enum(),
                                    $(
                                        inner.layout.$tex_storage_params as i32
                                    ),+
                                );

                                let id = Uuid::new_v4();
                                let lru_node = LruNode::new(id);
                                let memory = inner.used_memory();
                                (*self.$descriptors).insert(id, Rc::downgrade(&descriptor.0));
                                (*self.lru).cache(lru_node);
                                (*self.used_memory) += memory;
                                inner.runtime = Some(Box::new(Runtime {
                                    id,
                                    gl: self.gl.clone(),
                                    store_id: self.id,
                                    texture: texture.clone(),
                                    memory,
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
                        $(
                            for (source, $($tex_params),+) in inner.layout.$tex_queue.drain(..) {
                                self.abilities
                                    .verify_texture_size(source.width(), source.height())?;
                                source.$tex_func(
                                    &self.gl,
                                    $tex_target,
                                    $(
                                        $tex_params
                                    ),+
                                )?;
                            }
                        )+

                        if is_new && inner.generate_mipmap {
                            self.gl.generate_mipmap($target);
                        }

                        self.gl
                            .bind_texture($target, None);
                        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

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
        tex_sub_image_2d,
            (queue, TextureTarget::TEXTURE_2D, level, x_offset, y_offset),
        tex_storage_2d,
            (width, height),
        use_texture_2d,
        unuse_texture_2d
    )
    (
        Texture3D,
        WebGl2RenderingContext::TEXTURE_3D,
        descriptors_3d,
        tex_sub_image_3d,
            (queue, TextureTarget::TEXTURE_3D, level, depth, x_offset, y_offset, z_offset),
        tex_storage_3d,
            (width, height, depth),
        use_texture_3d,
        unuse_texture_3d
    )
    (
        Texture2DArray,
        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
        descriptors_2d_array,
        tex_sub_image_3d,
            (queue, TextureTarget::TEXTURE_2D_ARRAY, level, array_index, x_offset, y_offset, array_index_offset),
        tex_storage_3d,
            (width, height, array_length),
        use_texture_2d_array,
        unuse_texture_2d_array
    )
    (
        TextureCubeMap,
        WebGl2RenderingContext::TEXTURE_CUBE_MAP,
        descriptors_cube_map,
        tex_sub_image_2d,
            (positive_x, TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_X, level, x_offset, y_offset),
            (negative_x, TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_X, level, x_offset, y_offset),
            (positive_y, TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_Y, level, x_offset, y_offset),
            (negative_y, TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_Y, level, x_offset, y_offset),
            (positive_z, TextureTarget::TEXTURE_CUBE_MAP_POSITIVE_Z, level, x_offset, y_offset),
            (negative_z, TextureTarget::TEXTURE_CUBE_MAP_NEGATIVE_Z, level, x_offset, y_offset),
        tex_storage_2d,
            (width, height),
        use_texture_cube_map,
        unuse_texture_cube_map
    )
}

macro_rules! store_drop {
    ($($d:ident)+) => {
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
    descriptors_2d
    descriptors_2d_array
    descriptors_3d
    descriptors_cube_map
);
