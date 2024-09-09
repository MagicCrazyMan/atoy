use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;
use proc::GlEnum;
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlSampler, WebGlTexture};

use crate::anewthing::channel::Channel;

use super::capabilities::WebGlCapabilities;

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

impl WebGlTextureUnit {
    pub fn unit_index(&self) -> u32 {
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

/// Available texture magnification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureMagnificationFilter {
    Linear,
    Nearest,
}

/// Available texture minification filters for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureMinificationFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

/// Available texture wrap methods for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureWrapMethod {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

/// Available texture compare function for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureCompareFunction {
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
pub enum WebGlTextureCompareMode {
    None,
    CompareRefToTexture,
}

/// Available texture color internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureInternalFormat {
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

// impl TextureInternalFormat for TextureUncompressedInternalFormat {
//     fn byte_length(&self, width: usize, height: usize) -> usize {
//         match self {
//             TextureUncompressedInternalFormat::RGBA32I => width * height * 16,
//             TextureUncompressedInternalFormat::RGBA32UI => width * height * 16,
//             TextureUncompressedInternalFormat::RGBA16I => width * height * 4,
//             TextureUncompressedInternalFormat::RGBA16UI => width * height * 4,
//             TextureUncompressedInternalFormat::RGBA8 => width * height * 4,
//             TextureUncompressedInternalFormat::RGBA8I => width * height * 4,
//             TextureUncompressedInternalFormat::RGBA8UI => width * height * 4,
//             TextureUncompressedInternalFormat::SRGB8_ALPHA8 => width * height * 4,
//             TextureUncompressedInternalFormat::RGB10_A2 => width * height * 4, // 10 + 10 + 10 + 2 in bits
//             TextureUncompressedInternalFormat::RGB10_A2UI => width * height * 4, // 10 + 10 + 10 + 2 in bits
//             TextureUncompressedInternalFormat::RGBA4 => width * height * 2,
//             TextureUncompressedInternalFormat::RGB5_A1 => width * height * 2, // 5 + 5 + 5 + 1 in bits
//             TextureUncompressedInternalFormat::RGB8 => width * height * 3,
//             TextureUncompressedInternalFormat::RGB565 => width * height * 2, // 5 + 6 + 5 in bits
//             TextureUncompressedInternalFormat::RG32I => width * height * 4,
//             TextureUncompressedInternalFormat::RG32UI => width * height * 4,
//             TextureUncompressedInternalFormat::RG16I => width * height * 4,
//             TextureUncompressedInternalFormat::RG16UI => width * height * 4,
//             TextureUncompressedInternalFormat::RG8 => width * height * 2,
//             TextureUncompressedInternalFormat::RG8I => width * height * 2,
//             TextureUncompressedInternalFormat::RG8UI => width * height * 2,
//             TextureUncompressedInternalFormat::R32I => width * height * 4,
//             TextureUncompressedInternalFormat::R32UI => width * height * 4,
//             TextureUncompressedInternalFormat::R16I => width * height * 2,
//             TextureUncompressedInternalFormat::R16UI => width * height * 2,
//             TextureUncompressedInternalFormat::R8 => width * height * 1,
//             TextureUncompressedInternalFormat::R8I => width * height * 1,
//             TextureUncompressedInternalFormat::R8UI => width * height * 1,
//             TextureUncompressedInternalFormat::RGBA32F => width * height * 16,
//             TextureUncompressedInternalFormat::RGBA16F => width * height * 4,
//             TextureUncompressedInternalFormat::RGBA8_SNORM => width * height * 4,
//             TextureUncompressedInternalFormat::RGB32F => width * height * 12,
//             TextureUncompressedInternalFormat::RGB32I => width * height * 12,
//             TextureUncompressedInternalFormat::RGB32UI => width * height * 12,
//             TextureUncompressedInternalFormat::RGB16F => width * height * 6,
//             TextureUncompressedInternalFormat::RGB16I => width * height * 6,
//             TextureUncompressedInternalFormat::RGB16UI => width * height * 6,
//             TextureUncompressedInternalFormat::RGB8_SNORM => width * height * 3,
//             TextureUncompressedInternalFormat::RGB8I => width * height * 3,
//             TextureUncompressedInternalFormat::RGB8UI => width * height * 3,
//             TextureUncompressedInternalFormat::SRGB8 => width * height * 3,
//             TextureUncompressedInternalFormat::R11F_G11F_B10F => width * height * 4, // 11 + 11 + 10 in bits
//             TextureUncompressedInternalFormat::RGB9_E5 => width * height * 4, // 9 + 9 + 9 + 5 in bits
//             TextureUncompressedInternalFormat::RG32F => width * height * 4,
//             TextureUncompressedInternalFormat::RG16F => width * height * 4,
//             TextureUncompressedInternalFormat::RG8_SNORM => width * height * 2,
//             TextureUncompressedInternalFormat::R32F => width * height * 4,
//             TextureUncompressedInternalFormat::R16F => width * height * 2,
//             TextureUncompressedInternalFormat::R8_SNORM => width * height * 1,
//             TextureUncompressedInternalFormat::DEPTH_COMPONENT32F => width * height * 4,
//             TextureUncompressedInternalFormat::DEPTH_COMPONENT24 => width * height * 3,
//             TextureUncompressedInternalFormat::DEPTH_COMPONENT16 => width * height * 2,
//             TextureUncompressedInternalFormat::DEPTH32F_STENCIL8 => width * height * 5, // 32 + 8 in bits
//             TextureUncompressedInternalFormat::DEPTH24_STENCIL8 => width * height * 4,
//         }
//     }
// }

/// Available texture compressed internal and upload formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlTextureCompressedFormat {
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

// impl TextureInternalFormat for TextureCompressedFormat {
//     fn byte_length(&self, width: usize, height: usize) -> usize {
//         match self {
//             // for S3TC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc/ for more details
//             TextureCompressedFormat::RGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::RGBA_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::RGBA_S3TC_DXT3 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::RGBA_S3TC_DXT5 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             // for S3TC RGBA, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_s3tc_srgb/ for more details
//             TextureCompressedFormat::SRGB_S3TC_DXT1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 8
//             }
//             TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             // for ETC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc/ for more details
//             TextureCompressedFormat::R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::SIGNED_R11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SIGNED_RG11_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::RGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::SRGB8_ETC2 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::RGBA8_ETC2_EAC => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ETC2_EAC => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::RGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 8
//             }
//             TextureCompressedFormat::SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 8
//             }
//             // for PVRTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_pvrtc/ for more details
//             TextureCompressedFormat::RGB_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
//             TextureCompressedFormat::RGBA_PVRTC_2BPPV1_IMG => width.max(16) * height.max(8) / 4,
//             TextureCompressedFormat::RGB_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
//             TextureCompressedFormat::RGBA_PVRTC_4BPPV1_IMG => width.max(8) * height.max(8) / 2,
//             // for ETC1, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_etc1/ for more details
//             TextureCompressedFormat::RGB_ETC1_WEBGL => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             // for ASTC, checks https://registry.khronos.org/webgl/extensions/WEBGL_compressed_texture_astc/ for more details
//             TextureCompressedFormat::RGBA_ASTC_4x4 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_4x4 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_5x4 => ((width + 4) / 5) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x4 => {
//                 ((width + 4) / 5) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_5x5 => ((width + 4) / 5) * ((height + 4) / 5) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x5 => {
//                 ((width + 4) / 5) * ((height + 4) / 5) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_6x5 => ((width + 5) / 6) * ((height + 4) / 5) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x5 => {
//                 ((width + 5) / 6) * ((height + 4) / 5) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_6x6 => ((width + 5) / 6) * ((height + 5) / 6) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x6 => {
//                 ((width + 5) / 6) * ((height + 5) / 6) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_8x5 => ((width + 7) / 8) * ((height + 4) / 5) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x5 => {
//                 ((width + 7) / 8) * ((height + 4) / 5) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_8x6 => ((width + 7) / 8) * ((height + 5) / 6) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x6 => {
//                 ((width + 7) / 8) * ((height + 5) / 6) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_8x8 => ((width + 7) / 8) * ((height + 7) / 8) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x8 => {
//                 ((width + 7) / 8) * ((height + 7) / 8) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_10x5 => ((width + 9) / 10) * ((height + 4) / 5) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x5 => {
//                 ((width + 9) / 10) * ((height + 4) / 5) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_10x6 => ((width + 9) / 10) * ((height + 5) / 6) * 16,
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x6 => {
//                 ((width + 9) / 10) * ((height + 5) / 6) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_10x10 => {
//                 ((width + 9) / 10) * ((height + 9) / 10) * 16
//             }
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x10 => {
//                 ((width + 9) / 10) * ((height + 9) / 10) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_12x10 => {
//                 ((width + 11) / 12) * ((height + 9) / 10) * 16
//             }
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x10 => {
//                 ((width + 11) / 12) * ((height + 9) / 10) * 16
//             }
//             TextureCompressedFormat::RGBA_ASTC_12x12 => {
//                 ((width + 11) / 12) * ((height + 11) / 12) * 16
//             }
//             TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x12 => {
//                 ((width + 11) / 12) * ((height + 11) / 12) * 16
//             }
//             // for BPTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_bptc/ for more details
//             TextureCompressedFormat::RGBA_BPTC_UNORM => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SRGB_ALPHA_BPTC_UNORM => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::RGB_BPTC_SIGNED_FLOAT => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             TextureCompressedFormat::RGB_BPTC_UNSIGNED_FLOAT => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//             // for RGTC, checks https://registry.khronos.org/webgl/extensions/EXT_texture_compression_rgtc/ for more details
//             TextureCompressedFormat::RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::SIGNED_RED_RGTC1 => ((width + 3) / 4) * ((height + 3) / 4) * 8,
//             TextureCompressedFormat::RED_GREEN_RGTC2 => ((width + 3) / 4) * ((height + 3) / 4) * 16,
//             TextureCompressedFormat::SIGNED_RED_GREEN_RGTC2 => {
//                 ((width + 3) / 4) * ((height + 3) / 4) * 16
//             }
//         }
//     }
// }

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

#[derive(Clone)]
pub struct WebGlTextureItem {
    gl_texture: WebGlTexture,
    gl_sampler: WebGlSampler,
    layout: WebGlTextureLayout,
    internal_format: WebGlTextureInternalFormat,
    width: usize,
    height: usize,
    levels: usize,
    depth: usize,
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

    /// Returns [`WebGlTextureLayout`].
    pub fn layout(&self) -> WebGlTextureLayout {
        self.layout
    }

    /// Returns [`WebGlTextureInternalFormat`].
    pub fn internal_format(&self) -> WebGlTextureInternalFormat {
        self.internal_format
    }

    /// Returns width of the texture.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns height of the texture.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns levels of the texture.
    pub fn levels(&self) -> usize {
        self.levels
    }

    /// Returns depth of the texture.
    /// Useless when texture layout is not [`WebGlTextureLayout::Texture3D`] or [`WebGlTextureLayout::Texture2DArray`].
    pub fn depth(&self) -> usize {
        self.depth
    }
}

pub struct WebGlTextureManager {
    id: Uuid,
    gl: WebGl2RenderingContext,
    capabilities: WebGlCapabilities,
    channel: Channel,
    textures: Rc<RefCell<HashMap<Uuid, WebGlTextureItem>>>,
}

impl WebGlTextureManager {
    pub fn new(
        gl: WebGl2RenderingContext,
        capabilities: WebGlCapabilities,
        channel: Channel,
    ) -> Self {
        let textures = Rc::new(RefCell::new(HashMap::new()));
        // channel.on::<TextureDropped>(TextureDroppedHandler::new(Rc::clone(&textures)));

        Self {
            id: Uuid::new_v4(),
            gl,
            capabilities,
            channel,
            textures,
        }
    }

    /// Returns texture manager id.
    pub fn id(&self) -> &Uuid {
        &self.id
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

// impl Handler<TextureDropped> for TextureDroppedHandler {
//     fn handle(&mut self, evt: &mut Event<'_, TextureDropped>) {
//         self.textures.borrow_mut().remove(evt.id());
//     }
// }
