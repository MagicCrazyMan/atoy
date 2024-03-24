use web_sys::{
    ExtTextureFilterAnisotropic, WebGl2RenderingContext, WebglCompressedTextureAstc,
    WebglCompressedTextureEtc, WebglCompressedTextureEtc1, WebglCompressedTexturePvrtc,
    WebglCompressedTextureS3tc, WebglCompressedTextureS3tcSrgb,
};

use super::{
    buffer::{BufferDataType, BufferTarget, BufferUsage},
    client_wait::ClientWaitFlags,
    draw::{CullFace, DrawMode, ElementIndicesDataType},
    framebuffer::{
        BlitFlilter, BlitMask, FramebufferAttachmentTarget, FramebufferTarget, OperableBuffer,
    },
    renderbuffer::RenderbufferInternalFormat,
    stencil::{StencilFunction, StencilOp},
    texture::{
        SamplerParameter, TextureCompareFunction, TextureCompareMode, TextureCompressedFormat,
        TextureInternalFormat, TextureMagnificationFilter, TextureMinificationFilter,
        TextureParameter, TexturePixelStorage, TextureTarget, TextureUncompressedInternalFormat,
        TextureUncompressedPixelDataType, TextureUncompressedPixelFormat, TextureUnit,
        TextureUnpackColorSpaceConversion, TextureWrapMethod,
    },
};

/// A trait converts Rust data type to WebGL u32.
pub trait ToGlEnum {
    fn gl_enum(&self) -> u32;
}

impl ToGlEnum for BufferTarget {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            BufferTarget::ARRAY_BUFFER => WebGl2RenderingContext::ARRAY_BUFFER,
            BufferTarget::ELEMENT_ARRAY_BUFFER => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            BufferTarget::COPY_READ_BUFFER => WebGl2RenderingContext::COPY_READ_BUFFER,
            BufferTarget::COPY_WRITE_BUFFER => WebGl2RenderingContext::COPY_WRITE_BUFFER,
            BufferTarget::TRANSFORM_FEEDBACK_BUFFER => {
                WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER
            }
            BufferTarget::UNIFORM_BUFFER => WebGl2RenderingContext::UNIFORM_BUFFER,
            BufferTarget::PIXEL_PACK_BUFFER => WebGl2RenderingContext::PIXEL_PACK_BUFFER,
            BufferTarget::PIXEL_UNPACK_BUFFER => WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
        }
    }
}

impl ToGlEnum for BufferDataType {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            BufferDataType::FLOAT => WebGl2RenderingContext::FLOAT,
            BufferDataType::BYTE => WebGl2RenderingContext::BYTE,
            BufferDataType::SHORT => WebGl2RenderingContext::SHORT,
            BufferDataType::INT => WebGl2RenderingContext::INT,
            BufferDataType::UNSIGNED_BYTE => WebGl2RenderingContext::UNSIGNED_BYTE,
            BufferDataType::UNSIGNED_SHORT => WebGl2RenderingContext::UNSIGNED_SHORT,
            BufferDataType::UNSIGNED_INT => WebGl2RenderingContext::UNSIGNED_INT,
            BufferDataType::HALF_FLOAT => WebGl2RenderingContext::HALF_FLOAT,
            BufferDataType::INT_2_10_10_10_REV => WebGl2RenderingContext::INT_2_10_10_10_REV,
            BufferDataType::UNSIGNED_INT_2_10_10_10_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
        }
    }
}

impl ToGlEnum for BufferUsage {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            BufferUsage::STATIC_DRAW => WebGl2RenderingContext::STATIC_DRAW,
            BufferUsage::DYNAMIC_DRAW => WebGl2RenderingContext::DYNAMIC_DRAW,
            BufferUsage::STREAM_DRAW => WebGl2RenderingContext::STREAM_DRAW,
            BufferUsage::STATIC_READ => WebGl2RenderingContext::STATIC_READ,
            BufferUsage::DYNAMIC_READ => WebGl2RenderingContext::DYNAMIC_READ,
            BufferUsage::STREAM_READ => WebGl2RenderingContext::STREAM_READ,
            BufferUsage::STATIC_COPY => WebGl2RenderingContext::STATIC_COPY,
            BufferUsage::DYNAMIC_COPY => WebGl2RenderingContext::DYNAMIC_COPY,
            BufferUsage::STREAM_COPY => WebGl2RenderingContext::STREAM_COPY,
        }
    }
}

impl ToGlEnum for ElementIndicesDataType {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            ElementIndicesDataType::UNSIGNED_BYTE => WebGl2RenderingContext::UNSIGNED_BYTE,
            ElementIndicesDataType::UNSIGNED_SHORT => WebGl2RenderingContext::UNSIGNED_SHORT,
            ElementIndicesDataType::UNSIGNED_INT => WebGl2RenderingContext::UNSIGNED_INT,
        }
    }
}

impl ToGlEnum for DrawMode {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            DrawMode::POINTS => WebGl2RenderingContext::POINTS,
            DrawMode::LINES => WebGl2RenderingContext::LINES,
            DrawMode::LINE_LOOP => WebGl2RenderingContext::LINE_LOOP,
            DrawMode::LINE_STRIP => WebGl2RenderingContext::LINE_STRIP,
            DrawMode::TRIANGLES => WebGl2RenderingContext::TRIANGLES,
            DrawMode::TRIANGLE_STRIP => WebGl2RenderingContext::TRIANGLE_STRIP,
            DrawMode::TRIANGLE_FAN => WebGl2RenderingContext::TRIANGLE_FAN,
        }
    }
}

impl ToGlEnum for CullFace {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            CullFace::FRONT => WebGl2RenderingContext::FRONT,
            CullFace::BACK => WebGl2RenderingContext::BACK,
            CullFace::FRONT_AND_BACK => WebGl2RenderingContext::FRONT_AND_BACK,
        }
    }
}

impl ToGlEnum for TextureTarget {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureTarget::TEXTURE_2D => WebGl2RenderingContext::TEXTURE_2D,
            TextureTarget::TEXTURE_CUBE_MAP => WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            TextureTarget::TEXTURE_2D_ARRAY => WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            TextureTarget::TEXTURE_3D => WebGl2RenderingContext::TEXTURE_3D,
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

impl ToGlEnum for TextureUncompressedPixelFormat {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUncompressedPixelFormat::RED => WebGl2RenderingContext::RED,
            TextureUncompressedPixelFormat::RED_INTEGER => WebGl2RenderingContext::RED_INTEGER,
            TextureUncompressedPixelFormat::RG => WebGl2RenderingContext::RG,
            TextureUncompressedPixelFormat::RG_INTEGER => WebGl2RenderingContext::RG_INTEGER,
            TextureUncompressedPixelFormat::RGB => WebGl2RenderingContext::RGB,
            TextureUncompressedPixelFormat::RGB_INTEGER => WebGl2RenderingContext::RGB_INTEGER,
            TextureUncompressedPixelFormat::RGBA => WebGl2RenderingContext::RGBA,
            TextureUncompressedPixelFormat::RGBA_INTEGER => WebGl2RenderingContext::RGBA_INTEGER,
            TextureUncompressedPixelFormat::LUMINANCE => WebGl2RenderingContext::LUMINANCE,
            TextureUncompressedPixelFormat::LUMINANCE_ALPHA => {
                WebGl2RenderingContext::LUMINANCE_ALPHA
            }
            TextureUncompressedPixelFormat::ALPHA => WebGl2RenderingContext::ALPHA,
            TextureUncompressedPixelFormat::DEPTH_COMPONENT => {
                WebGl2RenderingContext::DEPTH_COMPONENT
            }
            TextureUncompressedPixelFormat::DEPTH_STENCIL => WebGl2RenderingContext::DEPTH_STENCIL,
        }
    }
}

impl ToGlEnum for TextureInternalFormat {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureInternalFormat::Uncompressed(f) => f.gl_enum(),
            TextureInternalFormat::Compressed(f) => f.gl_enum(),
        }
    }
}

impl ToGlEnum for TextureUncompressedPixelDataType {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUncompressedPixelDataType::FLOAT => WebGl2RenderingContext::FLOAT,
            TextureUncompressedPixelDataType::HALF_FLOAT => WebGl2RenderingContext::HALF_FLOAT,
            TextureUncompressedPixelDataType::BYTE => WebGl2RenderingContext::BYTE,
            TextureUncompressedPixelDataType::SHORT => WebGl2RenderingContext::SHORT,
            TextureUncompressedPixelDataType::INT => WebGl2RenderingContext::INT,
            TextureUncompressedPixelDataType::UNSIGNED_BYTE => {
                WebGl2RenderingContext::UNSIGNED_BYTE
            }
            TextureUncompressedPixelDataType::UNSIGNED_SHORT => {
                WebGl2RenderingContext::UNSIGNED_SHORT
            }
            TextureUncompressedPixelDataType::UNSIGNED_INT => WebGl2RenderingContext::UNSIGNED_INT,
            TextureUncompressedPixelDataType::UNSIGNED_SHORT_5_6_5 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_5_6_5
            }
            TextureUncompressedPixelDataType::UNSIGNED_SHORT_4_4_4_4 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_4_4_4_4
            }
            TextureUncompressedPixelDataType::UNSIGNED_SHORT_5_5_5_1 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_5_5_5_1
            }
            TextureUncompressedPixelDataType::UNSIGNED_INT_2_10_10_10_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
            TextureUncompressedPixelDataType::UNSIGNED_INT_10F_11F_11F_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_10F_11F_11F_REV
            }
            TextureUncompressedPixelDataType::UNSIGNED_INT_5_9_9_9_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_5_9_9_9_REV
            }
            TextureUncompressedPixelDataType::UNSIGNED_INT_24_8 => {
                WebGl2RenderingContext::UNSIGNED_INT_24_8
            }
            TextureUncompressedPixelDataType::FLOAT_32_UNSIGNED_INT_24_8_REV => {
                WebGl2RenderingContext::FLOAT_32_UNSIGNED_INT_24_8_REV
            }
        }
    }
}

impl ToGlEnum for TextureUnit {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUnit::TEXTURE0 => WebGl2RenderingContext::TEXTURE0,
            TextureUnit::TEXTURE1 => WebGl2RenderingContext::TEXTURE1,
            TextureUnit::TEXTURE2 => WebGl2RenderingContext::TEXTURE2,
            TextureUnit::TEXTURE3 => WebGl2RenderingContext::TEXTURE3,
            TextureUnit::TEXTURE4 => WebGl2RenderingContext::TEXTURE4,
            TextureUnit::TEXTURE5 => WebGl2RenderingContext::TEXTURE5,
            TextureUnit::TEXTURE6 => WebGl2RenderingContext::TEXTURE6,
            TextureUnit::TEXTURE7 => WebGl2RenderingContext::TEXTURE7,
            TextureUnit::TEXTURE8 => WebGl2RenderingContext::TEXTURE8,
            TextureUnit::TEXTURE9 => WebGl2RenderingContext::TEXTURE9,
            TextureUnit::TEXTURE10 => WebGl2RenderingContext::TEXTURE10,
            TextureUnit::TEXTURE11 => WebGl2RenderingContext::TEXTURE11,
            TextureUnit::TEXTURE12 => WebGl2RenderingContext::TEXTURE12,
            TextureUnit::TEXTURE13 => WebGl2RenderingContext::TEXTURE13,
            TextureUnit::TEXTURE14 => WebGl2RenderingContext::TEXTURE14,
            TextureUnit::TEXTURE15 => WebGl2RenderingContext::TEXTURE15,
            TextureUnit::TEXTURE16 => WebGl2RenderingContext::TEXTURE16,
            TextureUnit::TEXTURE17 => WebGl2RenderingContext::TEXTURE17,
            TextureUnit::TEXTURE18 => WebGl2RenderingContext::TEXTURE18,
            TextureUnit::TEXTURE19 => WebGl2RenderingContext::TEXTURE19,
            TextureUnit::TEXTURE20 => WebGl2RenderingContext::TEXTURE20,
            TextureUnit::TEXTURE21 => WebGl2RenderingContext::TEXTURE21,
            TextureUnit::TEXTURE22 => WebGl2RenderingContext::TEXTURE22,
            TextureUnit::TEXTURE23 => WebGl2RenderingContext::TEXTURE23,
            TextureUnit::TEXTURE24 => WebGl2RenderingContext::TEXTURE24,
            TextureUnit::TEXTURE25 => WebGl2RenderingContext::TEXTURE25,
            TextureUnit::TEXTURE26 => WebGl2RenderingContext::TEXTURE26,
            TextureUnit::TEXTURE27 => WebGl2RenderingContext::TEXTURE27,
            TextureUnit::TEXTURE28 => WebGl2RenderingContext::TEXTURE28,
            TextureUnit::TEXTURE29 => WebGl2RenderingContext::TEXTURE29,
            TextureUnit::TEXTURE30 => WebGl2RenderingContext::TEXTURE30,
            TextureUnit::TEXTURE31 => WebGl2RenderingContext::TEXTURE31,
        }
    }
}

impl ToGlEnum for TextureUnpackColorSpaceConversion {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureUnpackColorSpaceConversion::NONE => WebGl2RenderingContext::NONE,
            TextureUnpackColorSpaceConversion::BROWSER_DEFAULT_WEBGL => {
                WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL
            }
        }
    }
}

impl ToGlEnum for TexturePixelStorage {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TexturePixelStorage::PACK_ALIGNMENT(_) => WebGl2RenderingContext::PACK_ALIGNMENT,
            TexturePixelStorage::UNPACK_ALIGNMENT(_) => WebGl2RenderingContext::UNPACK_ALIGNMENT,
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(_) => {
                WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL
            }
            TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(_) => {
                WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL
            }
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(_) => {
                WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL
            }
            TexturePixelStorage::PACK_ROW_LENGTH(_) => WebGl2RenderingContext::PACK_ROW_LENGTH,
            TexturePixelStorage::PACK_SKIP_PIXELS(_) => WebGl2RenderingContext::PACK_SKIP_PIXELS,
            TexturePixelStorage::PACK_SKIP_ROWS(_) => WebGl2RenderingContext::PACK_SKIP_ROWS,
            TexturePixelStorage::UNPACK_ROW_LENGTH(_) => WebGl2RenderingContext::UNPACK_ROW_LENGTH,
            TexturePixelStorage::UNPACK_IMAGE_HEIGHT(_) => {
                WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT
            }
            TexturePixelStorage::UNPACK_SKIP_PIXELS(_) => {
                WebGl2RenderingContext::UNPACK_SKIP_PIXELS
            }
            TexturePixelStorage::UNPACK_SKIP_ROWS(_) => WebGl2RenderingContext::UNPACK_SKIP_ROWS,
            TexturePixelStorage::UNPACK_SKIP_IMAGES(_) => {
                WebGl2RenderingContext::UNPACK_SKIP_IMAGES
            }
        }
    }
}

impl ToGlEnum for TextureMagnificationFilter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureMagnificationFilter::LINEAR => WebGl2RenderingContext::LINEAR,
            TextureMagnificationFilter::NEAREST => WebGl2RenderingContext::NEAREST,
        }
    }
}

impl ToGlEnum for TextureMinificationFilter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureMinificationFilter::LINEAR => WebGl2RenderingContext::LINEAR,
            TextureMinificationFilter::NEAREST => WebGl2RenderingContext::NEAREST,
            TextureMinificationFilter::NEAREST_MIPMAP_NEAREST => {
                WebGl2RenderingContext::NEAREST_MIPMAP_NEAREST
            }
            TextureMinificationFilter::LINEAR_MIPMAP_NEAREST => {
                WebGl2RenderingContext::LINEAR_MIPMAP_NEAREST
            }
            TextureMinificationFilter::NEAREST_MIPMAP_LINEAR => {
                WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR
            }
            TextureMinificationFilter::LINEAR_MIPMAP_LINEAR => {
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR
            }
        }
    }
}

impl ToGlEnum for TextureWrapMethod {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureWrapMethod::REPEAT => WebGl2RenderingContext::REPEAT,
            TextureWrapMethod::CLAMP_TO_EDGE => WebGl2RenderingContext::CLAMP_TO_EDGE,
            TextureWrapMethod::MIRRORED_REPEAT => WebGl2RenderingContext::MIRRORED_REPEAT,
        }
    }
}

impl ToGlEnum for TextureCompareFunction {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCompareFunction::LEQUAL => WebGl2RenderingContext::LEQUAL,
            TextureCompareFunction::GEQUAL => WebGl2RenderingContext::GEQUAL,
            TextureCompareFunction::LESS => WebGl2RenderingContext::LESS,
            TextureCompareFunction::GREATER => WebGl2RenderingContext::GREATER,
            TextureCompareFunction::EQUAL => WebGl2RenderingContext::EQUAL,
            TextureCompareFunction::NOTEQUAL => WebGl2RenderingContext::NOTEQUAL,
            TextureCompareFunction::ALWAYS => WebGl2RenderingContext::ALWAYS,
            TextureCompareFunction::NEVER => WebGl2RenderingContext::NEVER,
        }
    }
}

impl ToGlEnum for TextureCompareMode {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureCompareMode::NONE => WebGl2RenderingContext::NONE,
            TextureCompareMode::COMPARE_REF_TO_TEXTURE => {
                WebGl2RenderingContext::COMPARE_REF_TO_TEXTURE
            }
        }
    }
}

impl ToGlEnum for TextureParameter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            TextureParameter::BASE_LEVEL(_) => WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            TextureParameter::MAX_LEVEL(_) => WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            TextureParameter::MAX_ANISOTROPY(_) => {
                ExtTextureFilterAnisotropic::TEXTURE_MAX_ANISOTROPY_EXT
            }
        }
    }
}

impl ToGlEnum for SamplerParameter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            SamplerParameter::MAG_FILTER(_) => WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            SamplerParameter::MIN_FILTER(_) => WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            SamplerParameter::WRAP_S(_) => WebGl2RenderingContext::TEXTURE_WRAP_S,
            SamplerParameter::WRAP_T(_) => WebGl2RenderingContext::TEXTURE_WRAP_T,
            SamplerParameter::WRAP_R(_) => WebGl2RenderingContext::TEXTURE_WRAP_R,
            SamplerParameter::COMPARE_FUNC(_) => WebGl2RenderingContext::TEXTURE_COMPARE_FUNC,
            SamplerParameter::COMPARE_MODE(_) => WebGl2RenderingContext::TEXTURE_COMPARE_MODE,
            SamplerParameter::MAX_LOD(_) => WebGl2RenderingContext::TEXTURE_MAX_LOD,
            SamplerParameter::MIN_LOD(_) => WebGl2RenderingContext::TEXTURE_MIN_LOD,
        }
    }
}

impl ToGlEnum for StencilFunction {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            StencilFunction::NEVER => WebGl2RenderingContext::NEVER,
            StencilFunction::LESS => WebGl2RenderingContext::LESS,
            StencilFunction::EQUAL => WebGl2RenderingContext::EQUAL,
            StencilFunction::LEQUAL => WebGl2RenderingContext::LEQUAL,
            StencilFunction::GREATER => WebGl2RenderingContext::GREATER,
            StencilFunction::NOTEQUAL => WebGl2RenderingContext::NOTEQUAL,
            StencilFunction::GEQUAL => WebGl2RenderingContext::GEQUAL,
            StencilFunction::ALWAYS => WebGl2RenderingContext::ALWAYS,
        }
    }
}

impl ToGlEnum for StencilOp {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            StencilOp::KEEP => WebGl2RenderingContext::KEEP,
            StencilOp::ZERO => WebGl2RenderingContext::ZERO,
            StencilOp::REPLACE => WebGl2RenderingContext::REPLACE,
            StencilOp::INCR => WebGl2RenderingContext::INCR,
            StencilOp::INCR_WRAP => WebGl2RenderingContext::INCR_WRAP,
            StencilOp::DECR => WebGl2RenderingContext::DECR,
            StencilOp::DECR_WRAP => WebGl2RenderingContext::DECR_WRAP,
            StencilOp::INVERT => WebGl2RenderingContext::INVERT,
        }
    }
}

impl ToGlEnum for RenderbufferInternalFormat {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            RenderbufferInternalFormat::RGBA32I => WebGl2RenderingContext::RGBA32I,
            RenderbufferInternalFormat::RGBA32UI => WebGl2RenderingContext::RGBA32UI,
            RenderbufferInternalFormat::RGBA16I => WebGl2RenderingContext::RGBA16I,
            RenderbufferInternalFormat::RGBA16UI => WebGl2RenderingContext::RGBA16UI,
            RenderbufferInternalFormat::RGBA8 => WebGl2RenderingContext::RGBA8,
            RenderbufferInternalFormat::RGBA8I => WebGl2RenderingContext::RGBA8I,
            RenderbufferInternalFormat::RGBA8UI => WebGl2RenderingContext::RGBA8UI,
            RenderbufferInternalFormat::SRGB8_ALPHA8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            RenderbufferInternalFormat::RGB10_A2 => WebGl2RenderingContext::RGB10_A2,
            RenderbufferInternalFormat::RGB10_A2UI => WebGl2RenderingContext::RGB10_A2UI,
            RenderbufferInternalFormat::RGBA4 => WebGl2RenderingContext::RGBA4,
            RenderbufferInternalFormat::RGB5_A1 => WebGl2RenderingContext::RGB5_A1,
            RenderbufferInternalFormat::RGB8 => WebGl2RenderingContext::RGB8,
            RenderbufferInternalFormat::RGB565 => WebGl2RenderingContext::RGB565,
            RenderbufferInternalFormat::RG32I => WebGl2RenderingContext::RG32I,
            RenderbufferInternalFormat::RG32UI => WebGl2RenderingContext::RG32UI,
            RenderbufferInternalFormat::RG16I => WebGl2RenderingContext::RG16I,
            RenderbufferInternalFormat::RG16UI => WebGl2RenderingContext::RG16UI,
            RenderbufferInternalFormat::RG8 => WebGl2RenderingContext::RG8,
            RenderbufferInternalFormat::RG8I => WebGl2RenderingContext::RG8I,
            RenderbufferInternalFormat::RG8UI => WebGl2RenderingContext::RG8UI,
            RenderbufferInternalFormat::R32I => WebGl2RenderingContext::R32I,
            RenderbufferInternalFormat::R32UI => WebGl2RenderingContext::R32UI,
            RenderbufferInternalFormat::R16I => WebGl2RenderingContext::R16I,
            RenderbufferInternalFormat::R16UI => WebGl2RenderingContext::R16UI,
            RenderbufferInternalFormat::R8 => WebGl2RenderingContext::R8,
            RenderbufferInternalFormat::R8I => WebGl2RenderingContext::R8I,
            RenderbufferInternalFormat::R8UI => WebGl2RenderingContext::R8UI,
            RenderbufferInternalFormat::DEPTH_COMPONENT32F => {
                WebGl2RenderingContext::DEPTH_COMPONENT32F
            }
            RenderbufferInternalFormat::DEPTH_COMPONENT24 => {
                WebGl2RenderingContext::DEPTH_COMPONENT24
            }
            RenderbufferInternalFormat::DEPTH_COMPONENT16 => {
                WebGl2RenderingContext::DEPTH_COMPONENT16
            }
            RenderbufferInternalFormat::DEPTH32F_STENCIL8 => {
                WebGl2RenderingContext::DEPTH32F_STENCIL8
            }
            RenderbufferInternalFormat::DEPTH24_STENCIL8 => {
                WebGl2RenderingContext::DEPTH24_STENCIL8
            }
            RenderbufferInternalFormat::R16F => WebGl2RenderingContext::R16F,
            RenderbufferInternalFormat::RG16F => WebGl2RenderingContext::RG16F,
            RenderbufferInternalFormat::RGBA16F => WebGl2RenderingContext::RGBA16F,
            RenderbufferInternalFormat::R32F => WebGl2RenderingContext::R32F,
            RenderbufferInternalFormat::RG32F => WebGl2RenderingContext::RG32F,
            RenderbufferInternalFormat::RGBA32F => WebGl2RenderingContext::RGBA32F,
            RenderbufferInternalFormat::R11F_G11F_B10F => WebGl2RenderingContext::R11F_G11F_B10F,
        }
    }
}

impl ToGlEnum for FramebufferTarget {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            FramebufferTarget::READ_FRAMEBUFFER => WebGl2RenderingContext::READ_FRAMEBUFFER,
            FramebufferTarget::DRAW_FRAMEBUFFER => WebGl2RenderingContext::DRAW_FRAMEBUFFER,
        }
    }
}

impl ToGlEnum for FramebufferAttachmentTarget {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            FramebufferAttachmentTarget::COLOR_ATTACHMENT0 => WebGl2RenderingContext::COLOR_ATTACHMENT0,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT1 => WebGl2RenderingContext::COLOR_ATTACHMENT1,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT2 => WebGl2RenderingContext::COLOR_ATTACHMENT2,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT3 => WebGl2RenderingContext::COLOR_ATTACHMENT3,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT4 => WebGl2RenderingContext::COLOR_ATTACHMENT4,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT5 => WebGl2RenderingContext::COLOR_ATTACHMENT5,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT6 => WebGl2RenderingContext::COLOR_ATTACHMENT6,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT7 => WebGl2RenderingContext::COLOR_ATTACHMENT7,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT8 => WebGl2RenderingContext::COLOR_ATTACHMENT8,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT9 => WebGl2RenderingContext::COLOR_ATTACHMENT9,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT10 => WebGl2RenderingContext::COLOR_ATTACHMENT10,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT11 => WebGl2RenderingContext::COLOR_ATTACHMENT11,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT12 => WebGl2RenderingContext::COLOR_ATTACHMENT12,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT13 => WebGl2RenderingContext::COLOR_ATTACHMENT13,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT14 => WebGl2RenderingContext::COLOR_ATTACHMENT14,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT15 => WebGl2RenderingContext::COLOR_ATTACHMENT15,
            FramebufferAttachmentTarget::DEPTH_ATTACHMENT => WebGl2RenderingContext::DEPTH_ATTACHMENT,
            FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT => {
                WebGl2RenderingContext::DEPTH_STENCIL_ATTACHMENT
            }
            FramebufferAttachmentTarget::STENCIL_ATTACHMENT => WebGl2RenderingContext::STENCIL_ATTACHMENT,
        }
    }
}

impl ToGlEnum for OperableBuffer {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            OperableBuffer::NONE => WebGl2RenderingContext::NONE,
            OperableBuffer::BACK => WebGl2RenderingContext::BACK,
            OperableBuffer::COLOR_ATTACHMENT0 => WebGl2RenderingContext::COLOR_ATTACHMENT0,
            OperableBuffer::COLOR_ATTACHMENT1 => WebGl2RenderingContext::COLOR_ATTACHMENT1,
            OperableBuffer::COLOR_ATTACHMENT2 => WebGl2RenderingContext::COLOR_ATTACHMENT2,
            OperableBuffer::COLOR_ATTACHMENT3 => WebGl2RenderingContext::COLOR_ATTACHMENT3,
            OperableBuffer::COLOR_ATTACHMENT4 => WebGl2RenderingContext::COLOR_ATTACHMENT4,
            OperableBuffer::COLOR_ATTACHMENT5 => WebGl2RenderingContext::COLOR_ATTACHMENT5,
            OperableBuffer::COLOR_ATTACHMENT6 => WebGl2RenderingContext::COLOR_ATTACHMENT6,
            OperableBuffer::COLOR_ATTACHMENT7 => WebGl2RenderingContext::COLOR_ATTACHMENT7,
            OperableBuffer::COLOR_ATTACHMENT8 => WebGl2RenderingContext::COLOR_ATTACHMENT8,
            OperableBuffer::COLOR_ATTACHMENT9 => WebGl2RenderingContext::COLOR_ATTACHMENT9,
            OperableBuffer::COLOR_ATTACHMENT10 => WebGl2RenderingContext::COLOR_ATTACHMENT10,
            OperableBuffer::COLOR_ATTACHMENT11 => WebGl2RenderingContext::COLOR_ATTACHMENT11,
            OperableBuffer::COLOR_ATTACHMENT12 => WebGl2RenderingContext::COLOR_ATTACHMENT12,
            OperableBuffer::COLOR_ATTACHMENT13 => WebGl2RenderingContext::COLOR_ATTACHMENT13,
            OperableBuffer::COLOR_ATTACHMENT14 => WebGl2RenderingContext::COLOR_ATTACHMENT14,
            OperableBuffer::COLOR_ATTACHMENT15 => WebGl2RenderingContext::COLOR_ATTACHMENT15,
        }
    }
}

impl ToGlEnum for BlitMask {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            BlitMask::COLOR_BUFFER_BIT => WebGl2RenderingContext::COLOR_BUFFER_BIT,
            BlitMask::DEPTH_BUFFER_BIT => WebGl2RenderingContext::DEPTH_BUFFER_BIT,
            BlitMask::STENCIL_BUFFER_BIT => WebGl2RenderingContext::STENCIL_BUFFER_BIT,
        }
    }
}

impl ToGlEnum for BlitFlilter {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            BlitFlilter::NEAREST => WebGl2RenderingContext::NEAREST,
            BlitFlilter::LINEAR => WebGl2RenderingContext::LINEAR,
        }
    }
}

impl ToGlEnum for ClientWaitFlags {
    #[inline]
    fn gl_enum(&self) -> u32 {
        match self {
            ClientWaitFlags::SYNC_FLUSH_COMMANDS_BIT => {
                WebGl2RenderingContext::SYNC_FLUSH_COMMANDS_BIT
            }
        }
    }
}
