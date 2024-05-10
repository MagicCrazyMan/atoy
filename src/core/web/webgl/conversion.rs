use web_sys::{
    ExtTextureFilterAnisotropic, WebGl2RenderingContext, WebglCompressedTextureAstc,
    WebglCompressedTextureEtc, WebglCompressedTextureEtc1, WebglCompressedTexturePvrtc,
    WebglCompressedTextureS3tc, WebglCompressedTextureS3tcSrgb,
};

use super::{
    attribute::{ArrayBufferDataType, IndicesDataType},
    blit::{BlitFilter, BlitMask},
    buffer::{BufferTarget, BufferUsage},
    client_wait::{ClientWaitFlag, ClientWaitStatus, FenceSyncFlag},
    cullface::CullFace,
    depth::DepthFunction,
    draw::DrawMode,
    framebuffer::{FramebufferAttachment, FramebufferTarget, OperableBuffer},
    pixel::{
        PixelDataType, PixelFormat, PixelPackStorage, PixelUnpackStorage,
        UnpackColorSpaceConversion,
    },
    renderbuffer::{RenderbufferInternalFormat, RenderbufferTarget},
    stencil::{StencilFunction, StencilOp},
    texture::{
        SamplerParameter, TextureCompareFunction, TextureCompareMode, TextureCompressedFormat,
        TextureCubeMapFace, TextureMagnificationFilter, TextureMinificationFilter,
        TextureParameter, TextureTarget, TextureUncompressedInternalFormat, TextureUnit,
        TextureWrapMethod,
    },
};
/// A trait converts Rust data type to WebGL u32.
pub trait ToGlEnum {
    fn gl_enum(&self) -> u32;
}

impl ToGlEnum for TextureTarget {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureTarget::Texture2D => WebGl2RenderingContext::TEXTURE_2D,
            TextureTarget::TextureCubeMap => WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            TextureTarget::Texture2DArray => WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            TextureTarget::Texture3D => WebGl2RenderingContext::TEXTURE_3D,
        }
    }
}

impl ToGlEnum for TextureUncompressedInternalFormat {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUncompressedInternalFormat::RGBA32I => WebGl2RenderingContext::RGBA32I,
            TextureUncompressedInternalFormat::RGBA32UI => WebGl2RenderingContext::RGBA32UI,
            TextureUncompressedInternalFormat::RGBA16I => WebGl2RenderingContext::RGBA16I,
            TextureUncompressedInternalFormat::RGBA16UI => WebGl2RenderingContext::RGBA16UI,
            TextureUncompressedInternalFormat::RGBA8 => WebGl2RenderingContext::RGBA8,
            TextureUncompressedInternalFormat::RGBA8I => WebGl2RenderingContext::RGBA8I,
            TextureUncompressedInternalFormat::RGBA8UI => WebGl2RenderingContext::RGBA8UI,
            TextureUncompressedInternalFormat::SRGB8_ALPHA8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            TextureUncompressedInternalFormat::RGB10_A2 => WebGl2RenderingContext::RGB10_A2,
            TextureUncompressedInternalFormat::RGB10_A2UI => WebGl2RenderingContext::RGB10_A2UI,
            TextureUncompressedInternalFormat::RGBA4 => WebGl2RenderingContext::RGBA4,
            TextureUncompressedInternalFormat::RGB5_A1 => WebGl2RenderingContext::RGB5_A1,
            TextureUncompressedInternalFormat::RGB8 => WebGl2RenderingContext::RGB8,
            TextureUncompressedInternalFormat::RGB565 => WebGl2RenderingContext::RGB565,
            TextureUncompressedInternalFormat::RG32I => WebGl2RenderingContext::RG32I,
            TextureUncompressedInternalFormat::RG32UI => WebGl2RenderingContext::RG32UI,
            TextureUncompressedInternalFormat::RG16I => WebGl2RenderingContext::RG16I,
            TextureUncompressedInternalFormat::RG16UI => WebGl2RenderingContext::RG16UI,
            TextureUncompressedInternalFormat::RG8 => WebGl2RenderingContext::RG8,
            TextureUncompressedInternalFormat::RG8I => WebGl2RenderingContext::RG8I,
            TextureUncompressedInternalFormat::RG8UI => WebGl2RenderingContext::RG8UI,
            TextureUncompressedInternalFormat::R32I => WebGl2RenderingContext::R32I,
            TextureUncompressedInternalFormat::R32UI => WebGl2RenderingContext::R32UI,
            TextureUncompressedInternalFormat::R16I => WebGl2RenderingContext::R16I,
            TextureUncompressedInternalFormat::R16UI => WebGl2RenderingContext::R16UI,
            TextureUncompressedInternalFormat::R8 => WebGl2RenderingContext::R8,
            TextureUncompressedInternalFormat::R8I => WebGl2RenderingContext::R8I,
            TextureUncompressedInternalFormat::R8UI => WebGl2RenderingContext::R8UI,
            TextureUncompressedInternalFormat::RGBA32F => WebGl2RenderingContext::RGBA32F,
            TextureUncompressedInternalFormat::RGBA16F => WebGl2RenderingContext::RGBA16F,
            TextureUncompressedInternalFormat::RGBA8_SNORM => WebGl2RenderingContext::RGBA8_SNORM,
            TextureUncompressedInternalFormat::RGB32F => WebGl2RenderingContext::RGB32F,
            TextureUncompressedInternalFormat::RGB32I => WebGl2RenderingContext::RGB32I,
            TextureUncompressedInternalFormat::RGB32UI => WebGl2RenderingContext::RGB32UI,
            TextureUncompressedInternalFormat::RGB16F => WebGl2RenderingContext::RGB16F,
            TextureUncompressedInternalFormat::RGB16I => WebGl2RenderingContext::RGB16I,
            TextureUncompressedInternalFormat::RGB16UI => WebGl2RenderingContext::RGB16UI,
            TextureUncompressedInternalFormat::RGB8_SNORM => WebGl2RenderingContext::RGB8_SNORM,
            TextureUncompressedInternalFormat::RGB8I => WebGl2RenderingContext::RGB8I,
            TextureUncompressedInternalFormat::RGB8UI => WebGl2RenderingContext::RGB8UI,
            TextureUncompressedInternalFormat::SRGB8 => WebGl2RenderingContext::SRGB8,
            TextureUncompressedInternalFormat::R11F_G11F_B10F => {
                WebGl2RenderingContext::R11F_G11F_B10F
            }
            TextureUncompressedInternalFormat::RGB9_E5 => WebGl2RenderingContext::RGB9_E5,
            TextureUncompressedInternalFormat::RG32F => WebGl2RenderingContext::RG32F,
            TextureUncompressedInternalFormat::RG16F => WebGl2RenderingContext::RG16F,
            TextureUncompressedInternalFormat::RG8_SNORM => WebGl2RenderingContext::RG8_SNORM,
            TextureUncompressedInternalFormat::R32F => WebGl2RenderingContext::R32F,
            TextureUncompressedInternalFormat::R16F => WebGl2RenderingContext::R16F,
            TextureUncompressedInternalFormat::R8_SNORM => WebGl2RenderingContext::R8_SNORM,
            TextureUncompressedInternalFormat::DEPTH_COMPONENT32F => {
                WebGl2RenderingContext::DEPTH_COMPONENT32F
            }
            TextureUncompressedInternalFormat::DEPTH_COMPONENT24 => {
                WebGl2RenderingContext::DEPTH_COMPONENT24
            }
            TextureUncompressedInternalFormat::DEPTH_COMPONENT16 => {
                WebGl2RenderingContext::DEPTH_COMPONENT16
            }
            TextureUncompressedInternalFormat::DEPTH32F_STENCIL8 => {
                WebGl2RenderingContext::DEPTH32F_STENCIL8
            }
            TextureUncompressedInternalFormat::DEPTH24_STENCIL8 => {
                WebGl2RenderingContext::DEPTH24_STENCIL8
            }
        }
    }
}

impl ToGlEnum for TextureCompressedFormat {
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCompressedFormat::RGB_S3TC_DXT1 => {
                WebglCompressedTextureS3tc::COMPRESSED_RGB_S3TC_DXT1_EXT
            }
            TextureCompressedFormat::RGBA_S3TC_DXT1 => {
                WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT1_EXT
            }
            TextureCompressedFormat::RGBA_S3TC_DXT3 => {
                WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT3_EXT
            }
            TextureCompressedFormat::RGBA_S3TC_DXT5 => {
                WebglCompressedTextureS3tc::COMPRESSED_RGBA_S3TC_DXT5_EXT
            }
            TextureCompressedFormat::SRGB_S3TC_DXT1 => {
                WebglCompressedTextureS3tcSrgb::COMPRESSED_SRGB_S3TC_DXT1_EXT
            }
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1 => {
                WebglCompressedTextureS3tcSrgb::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT
            }
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3 => {
                WebglCompressedTextureS3tcSrgb::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT
            }
            TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5 => {
                WebglCompressedTextureS3tcSrgb::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT
            }
            TextureCompressedFormat::R11_EAC => WebglCompressedTextureEtc::COMPRESSED_R11_EAC,
            TextureCompressedFormat::SIGNED_R11_EAC => {
                WebglCompressedTextureEtc::COMPRESSED_SIGNED_R11_EAC
            }
            TextureCompressedFormat::RG11_EAC => WebglCompressedTextureEtc::COMPRESSED_RG11_EAC,
            TextureCompressedFormat::SIGNED_RG11_EAC => {
                WebglCompressedTextureEtc::COMPRESSED_SIGNED_RG11_EAC
            }
            TextureCompressedFormat::RGB8_ETC2 => WebglCompressedTextureEtc::COMPRESSED_RGB8_ETC2,
            TextureCompressedFormat::RGBA8_ETC2_EAC => {
                WebglCompressedTextureEtc::COMPRESSED_RGBA8_ETC2_EAC
            }
            TextureCompressedFormat::SRGB8_ETC2 => WebglCompressedTextureEtc::COMPRESSED_SRGB8_ETC2,
            TextureCompressedFormat::SRGB8_ALPHA8_ETC2_EAC => {
                WebglCompressedTextureEtc::COMPRESSED_SRGB8_ALPHA8_ETC2_EAC
            }
            TextureCompressedFormat::RGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                WebglCompressedTextureEtc::COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2
            }
            TextureCompressedFormat::SRGB8_PUNCHTHROUGH_ALPHA1_ETC2 => {
                WebglCompressedTextureEtc::COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2
            }
            TextureCompressedFormat::RGB_PVRTC_2BPPV1_IMG => {
                WebglCompressedTexturePvrtc::COMPRESSED_RGB_PVRTC_2BPPV1_IMG
            }
            TextureCompressedFormat::RGBA_PVRTC_2BPPV1_IMG => {
                WebglCompressedTexturePvrtc::COMPRESSED_RGBA_PVRTC_2BPPV1_IMG
            }
            TextureCompressedFormat::RGB_PVRTC_4BPPV1_IMG => {
                WebglCompressedTexturePvrtc::COMPRESSED_RGB_PVRTC_4BPPV1_IMG
            }
            TextureCompressedFormat::RGBA_PVRTC_4BPPV1_IMG => {
                WebglCompressedTexturePvrtc::COMPRESSED_RGBA_PVRTC_4BPPV1_IMG
            }
            TextureCompressedFormat::RGB_ETC1_WEBGL => {
                WebglCompressedTextureEtc1::COMPRESSED_RGB_ETC1_WEBGL
            }
            TextureCompressedFormat::RGBA_ASTC_4x4 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_4X4_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_4x4 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_5x4 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_5X4_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x4 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_5x5 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_5X5_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_5x5 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_6x5 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_6X5_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x5 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_6x6 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_6X6_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_6x6 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_8x5 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_8X5_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x5 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_8x6 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_8X6_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x6 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_8x8 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_8X8_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_8x8 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_10x5 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_10X5_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x5 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_10x6 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_10X6_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x6 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_10x10 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_10X10_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_10x10 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_10X10_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_12x10 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_12X10_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x10 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR
            }
            TextureCompressedFormat::RGBA_ASTC_12x12 => {
                WebglCompressedTextureAstc::COMPRESSED_RGBA_ASTC_12X12_KHR
            }
            TextureCompressedFormat::SRGB8_ALPHA8_ASTC_12x12 => {
                WebglCompressedTextureAstc::COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR
            }
            TextureCompressedFormat::RGBA_BPTC_UNORM => 36492,
            TextureCompressedFormat::SRGB_ALPHA_BPTC_UNORM => 36493,
            TextureCompressedFormat::RGB_BPTC_SIGNED_FLOAT => 36494,
            TextureCompressedFormat::RGB_BPTC_UNSIGNED_FLOAT => 36495,
            TextureCompressedFormat::RED_RGTC1 => 36283,
            TextureCompressedFormat::SIGNED_RED_RGTC1 => 36284,
            TextureCompressedFormat::RED_GREEN_RGTC2 => 36285,
            TextureCompressedFormat::SIGNED_RED_GREEN_RGTC2 => 36286,
        }
    }
}

impl ToGlEnum for TextureUnit {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUnit::Texture0 => WebGl2RenderingContext::TEXTURE0,
            TextureUnit::Texture1 => WebGl2RenderingContext::TEXTURE1,
            TextureUnit::Texture2 => WebGl2RenderingContext::TEXTURE2,
            TextureUnit::Texture3 => WebGl2RenderingContext::TEXTURE3,
            TextureUnit::Texture4 => WebGl2RenderingContext::TEXTURE4,
            TextureUnit::Texture5 => WebGl2RenderingContext::TEXTURE5,
            TextureUnit::Texture6 => WebGl2RenderingContext::TEXTURE6,
            TextureUnit::Texture7 => WebGl2RenderingContext::TEXTURE7,
            TextureUnit::Texture8 => WebGl2RenderingContext::TEXTURE8,
            TextureUnit::Texture9 => WebGl2RenderingContext::TEXTURE9,
            TextureUnit::Texture10 => WebGl2RenderingContext::TEXTURE10,
            TextureUnit::Texture11 => WebGl2RenderingContext::TEXTURE11,
            TextureUnit::Texture12 => WebGl2RenderingContext::TEXTURE12,
            TextureUnit::Texture13 => WebGl2RenderingContext::TEXTURE13,
            TextureUnit::Texture14 => WebGl2RenderingContext::TEXTURE14,
            TextureUnit::Texture15 => WebGl2RenderingContext::TEXTURE15,
            TextureUnit::Texture16 => WebGl2RenderingContext::TEXTURE16,
            TextureUnit::Texture17 => WebGl2RenderingContext::TEXTURE17,
            TextureUnit::Texture18 => WebGl2RenderingContext::TEXTURE18,
            TextureUnit::Texture19 => WebGl2RenderingContext::TEXTURE19,
            TextureUnit::Texture20 => WebGl2RenderingContext::TEXTURE20,
            TextureUnit::Texture21 => WebGl2RenderingContext::TEXTURE21,
            TextureUnit::Texture22 => WebGl2RenderingContext::TEXTURE22,
            TextureUnit::Texture23 => WebGl2RenderingContext::TEXTURE23,
            TextureUnit::Texture24 => WebGl2RenderingContext::TEXTURE24,
            TextureUnit::Texture25 => WebGl2RenderingContext::TEXTURE25,
            TextureUnit::Texture26 => WebGl2RenderingContext::TEXTURE26,
            TextureUnit::Texture27 => WebGl2RenderingContext::TEXTURE27,
            TextureUnit::Texture28 => WebGl2RenderingContext::TEXTURE28,
            TextureUnit::Texture29 => WebGl2RenderingContext::TEXTURE29,
            TextureUnit::Texture30 => WebGl2RenderingContext::TEXTURE30,
            TextureUnit::Texture31 => WebGl2RenderingContext::TEXTURE31,
        }
    }
}

impl ToGlEnum for TextureMagnificationFilter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureMagnificationFilter::Linear => WebGl2RenderingContext::LINEAR,
            TextureMagnificationFilter::Nearest => WebGl2RenderingContext::NEAREST,
        }
    }
}

impl ToGlEnum for TextureMinificationFilter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureMinificationFilter::Linear => WebGl2RenderingContext::LINEAR,
            TextureMinificationFilter::Nearest => WebGl2RenderingContext::NEAREST,
            TextureMinificationFilter::NearestMipmapNearest => {
                WebGl2RenderingContext::NEAREST_MIPMAP_NEAREST
            }
            TextureMinificationFilter::LinearMipmapNearest => {
                WebGl2RenderingContext::LINEAR_MIPMAP_NEAREST
            }
            TextureMinificationFilter::NearestMipmapLinear => {
                WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR
            }
            TextureMinificationFilter::LinearMipmapLinear => {
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR
            }
        }
    }
}

impl ToGlEnum for TextureWrapMethod {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureWrapMethod::Repeat => WebGl2RenderingContext::REPEAT,
            TextureWrapMethod::ClampToEdge => WebGl2RenderingContext::CLAMP_TO_EDGE,
            TextureWrapMethod::MirroredRepeat => WebGl2RenderingContext::MIRRORED_REPEAT,
        }
    }
}

impl ToGlEnum for TextureCompareFunction {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCompareFunction::LessEqual => WebGl2RenderingContext::LEQUAL,
            TextureCompareFunction::GreaterEqual => WebGl2RenderingContext::GEQUAL,
            TextureCompareFunction::Less => WebGl2RenderingContext::LESS,
            TextureCompareFunction::Greater => WebGl2RenderingContext::GREATER,
            TextureCompareFunction::Equal => WebGl2RenderingContext::EQUAL,
            TextureCompareFunction::NotEqual => WebGl2RenderingContext::NOTEQUAL,
            TextureCompareFunction::Always => WebGl2RenderingContext::ALWAYS,
            TextureCompareFunction::Never => WebGl2RenderingContext::NEVER,
        }
    }
}

impl ToGlEnum for TextureCompareMode {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCompareMode::None => WebGl2RenderingContext::NONE,
            TextureCompareMode::CompareRefToTexture => {
                WebGl2RenderingContext::COMPARE_REF_TO_TEXTURE
            }
        }
    }
}

impl ToGlEnum for TextureParameter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureParameter::BaseLevel(_) => WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            TextureParameter::MaxLevel(_) => WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            TextureParameter::MaxAnisotropy(_) => {
                ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT
            }
        }
    }
}

impl ToGlEnum for SamplerParameter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            SamplerParameter::MagnificationFilter(_) => WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            SamplerParameter::MinificationFilter(_) => WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            SamplerParameter::WrapS(_) => WebGl2RenderingContext::TEXTURE_WRAP_S,
            SamplerParameter::WrapT(_) => WebGl2RenderingContext::TEXTURE_WRAP_T,
            SamplerParameter::WrapR(_) => WebGl2RenderingContext::TEXTURE_WRAP_R,
            SamplerParameter::CompareFunction(_) => WebGl2RenderingContext::TEXTURE_COMPARE_FUNC,
            SamplerParameter::CompareMode(_) => WebGl2RenderingContext::TEXTURE_COMPARE_MODE,
            SamplerParameter::MaxLod(_) => WebGl2RenderingContext::TEXTURE_MAX_LOD,
            SamplerParameter::MinLod(_) => WebGl2RenderingContext::TEXTURE_MIN_LOD,
        }
    }
}

impl ToGlEnum for TextureCubeMapFace {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCubeMapFace::PositiveX => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
            TextureCubeMapFace::NegativeX => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
            TextureCubeMapFace::PositiveY => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
            TextureCubeMapFace::NegativeY => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            TextureCubeMapFace::PositiveZ => WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
            TextureCubeMapFace::NegativeZ => WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        }
    }
}

