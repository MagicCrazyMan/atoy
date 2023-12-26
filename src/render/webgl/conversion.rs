use web_sys::WebGl2RenderingContext;

use super::{
    buffer::{BufferDataType, BufferTarget, BufferUsage},
    draw::{CullFace, DrawElementType, DrawMode},
    offscreen::{FramebufferAttachment, FramebufferTarget, FramebufferSource},
    renderbuffer::RenderbufferInternalFormat,
    stencil::{StencilFunction, StencilOp},
    texture::{
        TextureCompareFunction, TextureCompareMode, TextureDataType, TextureFormat,
        TextureInternalFormat, TextureMagnificationFilter, TextureMinificationFilter,
        TextureParameter, TexturePixelStorage, TextureUnit, TextureUnpackColorSpaceConversion,
        TextureWrapMethod,
    },
};

/// Rust data type representing WebGL GLenum.
pub type GLenum = u32;
/// Rust data type representing WebGL GLint.
pub type GLint = i32;
/// Rust data type representing WebGL GLintptr.
pub type GLintptr = i32;
/// Rust data type representing WebGL GLsizeiptr.
pub type GLsizeiptr = i32;
/// Rust data type representing WebGL GLsizei.
pub type GLsizei = i32;
/// Rust data type representing WebGL GLuint.
pub type GLuint = u32;
/// Rust data type representing WebGL GLboolean.
pub type GLboolean = bool;
/// Rust data type representing WebGL GLfloat.
pub type GLfloat = f32;

/// A trait converts Rust data type to WebGL GLenum.
pub trait ToGlEnum {
    fn gl_enum(&self) -> GLenum;
}

impl ToGlEnum for BufferTarget {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            BufferTarget::ArrayBuffer => WebGl2RenderingContext::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            BufferTarget::CopyReadBuffer => WebGl2RenderingContext::COPY_READ_BUFFER,
            BufferTarget::CopyWriteBuffer => WebGl2RenderingContext::COPY_WRITE_BUFFER,
            BufferTarget::TransformFeedbackBuffer => {
                WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER
            }
            BufferTarget::UniformBuffer => WebGl2RenderingContext::UNIFORM_BUFFER,
            BufferTarget::PixelPackBuffer => WebGl2RenderingContext::PIXEL_PACK_BUFFER,
            BufferTarget::PixelUnpackBuffer => WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
        }
    }
}

impl ToGlEnum for BufferDataType {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            BufferDataType::Float => WebGl2RenderingContext::FLOAT,
            BufferDataType::Byte => WebGl2RenderingContext::BYTE,
            BufferDataType::Short => WebGl2RenderingContext::SHORT,
            BufferDataType::Int => WebGl2RenderingContext::INT,
            BufferDataType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            BufferDataType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            BufferDataType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
            BufferDataType::HalfFloat => WebGl2RenderingContext::HALF_FLOAT,
            BufferDataType::Int_2_10_10_10_Rev => WebGl2RenderingContext::INT_2_10_10_10_REV,
            BufferDataType::UnsignedInt_2_10_10_10_Rev => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
        }
    }
}

impl ToGlEnum for BufferUsage {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            BufferUsage::StaticDraw => WebGl2RenderingContext::STATIC_DRAW,
            BufferUsage::DynamicDraw => WebGl2RenderingContext::DYNAMIC_DRAW,
            BufferUsage::StreamDraw => WebGl2RenderingContext::STREAM_DRAW,
            BufferUsage::StaticRead => WebGl2RenderingContext::STATIC_READ,
            BufferUsage::DynamicRead => WebGl2RenderingContext::DYNAMIC_READ,
            BufferUsage::StreamRead => WebGl2RenderingContext::STREAM_READ,
            BufferUsage::StaticCopy => WebGl2RenderingContext::STATIC_COPY,
            BufferUsage::DynamicCopy => WebGl2RenderingContext::DYNAMIC_COPY,
            BufferUsage::StreamCopy => WebGl2RenderingContext::STATIC_COPY,
        }
    }
}

impl ToGlEnum for DrawElementType {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            DrawElementType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            DrawElementType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            DrawElementType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
        }
    }
}

impl ToGlEnum for DrawMode {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            DrawMode::Points => WebGl2RenderingContext::POINTS,
            DrawMode::Lines => WebGl2RenderingContext::LINES,
            DrawMode::LineLoop => WebGl2RenderingContext::LINE_LOOP,
            DrawMode::LineStrip => WebGl2RenderingContext::LINE_STRIP,
            DrawMode::Triangles => WebGl2RenderingContext::TRIANGLES,
            DrawMode::TriangleStrip => WebGl2RenderingContext::TRIANGLE_STRIP,
            DrawMode::TriangleFan => WebGl2RenderingContext::TRIANGLE_FAN,
        }
    }
}

impl ToGlEnum for CullFace {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            CullFace::Front => WebGl2RenderingContext::FRONT,
            CullFace::Back => WebGl2RenderingContext::BACK,
            CullFace::Both => WebGl2RenderingContext::FRONT_AND_BACK,
        }
    }
}

impl ToGlEnum for TextureInternalFormat {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureInternalFormat::RGB => WebGl2RenderingContext::RGB,
            TextureInternalFormat::RGBA => WebGl2RenderingContext::RGBA,
            TextureInternalFormat::LUMINANCE => WebGl2RenderingContext::LUMINANCE,
            TextureInternalFormat::LUMINANCE_ALPHA => WebGl2RenderingContext::LUMINANCE_ALPHA,
            TextureInternalFormat::ALPHA => WebGl2RenderingContext::ALPHA,
            TextureInternalFormat::SRGBA8 => WebGl2RenderingContext::SRGB8,
            TextureInternalFormat::SRGB8_ALPHA8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            TextureInternalFormat::R8 => WebGl2RenderingContext::R8,
            TextureInternalFormat::R16F => WebGl2RenderingContext::R16F,
            TextureInternalFormat::R32F => WebGl2RenderingContext::R32F,
            TextureInternalFormat::R8UI => WebGl2RenderingContext::R8UI,
            TextureInternalFormat::RG8 => WebGl2RenderingContext::RG8,
            TextureInternalFormat::RG16F => WebGl2RenderingContext::RG16F,
            TextureInternalFormat::RG32F => WebGl2RenderingContext::RG32F,
            TextureInternalFormat::RG8UI => WebGl2RenderingContext::RG8UI,
            TextureInternalFormat::RG16UI => WebGl2RenderingContext::RG16UI,
            TextureInternalFormat::RG32UI => WebGl2RenderingContext::RG32UI,
            TextureInternalFormat::SRGB8 => WebGl2RenderingContext::SRGB8,
            TextureInternalFormat::RGB565 => WebGl2RenderingContext::RGB565,
            TextureInternalFormat::R11F_G11F_B10F => WebGl2RenderingContext::R11F_G11F_B10F,
            TextureInternalFormat::RGB9_E5 => WebGl2RenderingContext::RGB9_E5,
            TextureInternalFormat::RGB16F => WebGl2RenderingContext::RGB16F,
            TextureInternalFormat::RGB32F => WebGl2RenderingContext::RGB32F,
            TextureInternalFormat::RGB8UI => WebGl2RenderingContext::RGB8UI,
            TextureInternalFormat::RGBA8 => WebGl2RenderingContext::RGBA8,
            TextureInternalFormat::RGB5_A1 => WebGl2RenderingContext::RGB5_A1,
            TextureInternalFormat::RGB10_A2 => WebGl2RenderingContext::RGB10_A2,
            TextureInternalFormat::RGBA4 => WebGl2RenderingContext::RGBA4,
            TextureInternalFormat::RGBA16F => WebGl2RenderingContext::RGBA16F,
            TextureInternalFormat::RGBA32F => WebGl2RenderingContext::RGBA32F,
            TextureInternalFormat::RGBA8UI => WebGl2RenderingContext::RGBA8UI,
            TextureInternalFormat::DEPTH_COMPONENT => WebGl2RenderingContext::DEPTH_COMPONENT,
            TextureInternalFormat::DEPTH_STENCIL => WebGl2RenderingContext::DEPTH_STENCIL,
            TextureInternalFormat::R8_SNORM => WebGl2RenderingContext::R8_SNORM,
            TextureInternalFormat::R32I => WebGl2RenderingContext::R32I,
            TextureInternalFormat::R32UI => WebGl2RenderingContext::R32UI,
            TextureInternalFormat::RG8_SNORM => WebGl2RenderingContext::RG8_SNORM,
            TextureInternalFormat::RG8I => WebGl2RenderingContext::RG8I,
            TextureInternalFormat::RG16I => WebGl2RenderingContext::RG16I,
            TextureInternalFormat::RG32I => WebGl2RenderingContext::RG32I,
            TextureInternalFormat::RGB8_SNORM => WebGl2RenderingContext::RGB8_SNORM,
            TextureInternalFormat::RGBA8_SNORM => WebGl2RenderingContext::RGBA8_SNORM,
            TextureInternalFormat::RGB8I => WebGl2RenderingContext::RGB8I,
            TextureInternalFormat::RGB16I => WebGl2RenderingContext::RGB16I,
            TextureInternalFormat::RGB16UI => WebGl2RenderingContext::RGB16UI,
            TextureInternalFormat::RGB32I => WebGl2RenderingContext::RGB32I,
            TextureInternalFormat::RGB32UI => WebGl2RenderingContext::RGB32UI,
            TextureInternalFormat::RGBA8I => WebGl2RenderingContext::RGBA8I,
            TextureInternalFormat::RGBA16I => WebGl2RenderingContext::RGBA16I,
            TextureInternalFormat::RGBA16UI => WebGl2RenderingContext::RGBA16UI,
            TextureInternalFormat::RGBA32I => WebGl2RenderingContext::RGBA32I,
            TextureInternalFormat::RGBA32UI => WebGl2RenderingContext::RGBA32UI,
            TextureInternalFormat::RGB10_A2UI => WebGl2RenderingContext::RGB10_A2UI,
        }
    }
}

impl ToGlEnum for TextureFormat {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureFormat::RED => WebGl2RenderingContext::RED,
            TextureFormat::RED_INTEGER => WebGl2RenderingContext::RED_INTEGER,
            TextureFormat::RG => WebGl2RenderingContext::RG,
            TextureFormat::RG_INTEGER => WebGl2RenderingContext::RG_INTEGER,
            TextureFormat::RGB => WebGl2RenderingContext::RGB,
            TextureFormat::RGB_INTEGER => WebGl2RenderingContext::RGB_INTEGER,
            TextureFormat::RGBA => WebGl2RenderingContext::RGBA,
            TextureFormat::RGBA_INTEGER => WebGl2RenderingContext::RGBA_INTEGER,
            TextureFormat::LUMINANCE => WebGl2RenderingContext::LUMINANCE,
            TextureFormat::LUMINANCE_ALPHA => WebGl2RenderingContext::LUMINANCE_ALPHA,
            TextureFormat::ALPHA => WebGl2RenderingContext::ALPHA,
        }
    }
}

impl ToGlEnum for TextureDataType {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureDataType::FLOAT => WebGl2RenderingContext::FLOAT,
            TextureDataType::HALF_FLOAT => WebGl2RenderingContext::HALF_FLOAT,
            TextureDataType::BYTE => WebGl2RenderingContext::BYTE,
            TextureDataType::SHORT => WebGl2RenderingContext::SHORT,
            TextureDataType::INT => WebGl2RenderingContext::INT,
            TextureDataType::UNSIGNED_BYTE => WebGl2RenderingContext::UNSIGNED_BYTE,
            TextureDataType::UNSIGNED_SHORT => WebGl2RenderingContext::UNSIGNED_SHORT,
            TextureDataType::UNSIGNED_INT => WebGl2RenderingContext::UNSIGNED_INT,
            TextureDataType::UNSIGNED_SHORT_5_6_5 => WebGl2RenderingContext::UNSIGNED_SHORT_5_6_5,
            TextureDataType::UNSIGNED_SHORT_4_4_4_4 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_4_4_4_4
            }
            TextureDataType::UNSIGNED_SHORT_5_5_5_1 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_5_5_5_1
            }
            TextureDataType::UNSIGNED_INT_2_10_10_10_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
            TextureDataType::UNSIGNED_INT_10F_11F_11F_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_10F_11F_11F_REV
            }
            TextureDataType::UNSIGNED_INT_5_9_9_9_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_5_9_9_9_REV
            }
            TextureDataType::UNSIGNED_INT_24_8 => WebGl2RenderingContext::UNSIGNED_INT_24_8,
            TextureDataType::FLOAT_32_UNSIGNED_INT_24_8_REV => {
                WebGl2RenderingContext::FLOAT_32_UNSIGNED_INT_24_8_REV
            }
        }
    }
}

impl ToGlEnum for TextureUnit {
    #[inline]
    fn gl_enum(&self) -> GLenum {
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
            TextureUnit::Custom(index) => WebGl2RenderingContext::TEXTURE0 + *index,
        }
    }
}

impl ToGlEnum for TextureUnpackColorSpaceConversion {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureUnpackColorSpaceConversion::None => WebGl2RenderingContext::NONE,
            TextureUnpackColorSpaceConversion::BrowserDefaultWebGL => {
                WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL
            }
        }
    }
}

impl ToGlEnum for TexturePixelStorage {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TexturePixelStorage::PackAlignment(_) => WebGl2RenderingContext::PACK_ALIGNMENT,
            TexturePixelStorage::UnpackAlignment(_) => WebGl2RenderingContext::UNPACK_ALIGNMENT,
            TexturePixelStorage::UnpackFlipYWebGL(_) => WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
            TexturePixelStorage::UnpackPremultiplyAlphaWebGL(_) => {
                WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL
            }
            TexturePixelStorage::UnpackColorSpaceConversionWebGL(_) => {
                WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL
            }
            TexturePixelStorage::PackRowLength(_) => WebGl2RenderingContext::PACK_ROW_LENGTH,
            TexturePixelStorage::PackSkipPixels(_) => WebGl2RenderingContext::PACK_SKIP_PIXELS,
            TexturePixelStorage::PackSkipRows(_) => WebGl2RenderingContext::PACK_SKIP_ROWS,
            TexturePixelStorage::UnpackRowLength(_) => WebGl2RenderingContext::UNPACK_ROW_LENGTH,
            TexturePixelStorage::UnpackImageHeight(_) => {
                WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT
            }
            TexturePixelStorage::UnpackSkipPixels(_) => WebGl2RenderingContext::UNPACK_SKIP_PIXELS,
            TexturePixelStorage::UnpackSkipRows(_) => WebGl2RenderingContext::UNPACK_SKIP_ROWS,
            TexturePixelStorage::UnpackSkipImages(_) => WebGl2RenderingContext::UNPACK_SKIP_IMAGES,
        }
    }
}

impl ToGlEnum for TextureMagnificationFilter {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureMagnificationFilter::Linear => WebGl2RenderingContext::LINEAR,
            TextureMagnificationFilter::Nearest => WebGl2RenderingContext::NEAREST,
        }
    }
}

impl ToGlEnum for TextureMinificationFilter {
    #[inline]
    fn gl_enum(&self) -> GLenum {
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
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureWrapMethod::Repeat => WebGl2RenderingContext::REPEAT,
            TextureWrapMethod::ClampToEdge => WebGl2RenderingContext::CLAMP_TO_EDGE,
            TextureWrapMethod::MirroredRepeat => WebGl2RenderingContext::MIRRORED_REPEAT,
        }
    }
}

impl ToGlEnum for TextureCompareFunction {
    #[inline]
    fn gl_enum(&self) -> GLenum {
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
    fn gl_enum(&self) -> GLenum {
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
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureParameter::MagFilter(_) => WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            TextureParameter::MinFilter(_) => WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            TextureParameter::WrapS(_) => WebGl2RenderingContext::TEXTURE_WRAP_S,
            TextureParameter::WrapT(_) => WebGl2RenderingContext::TEXTURE_WRAP_T,
            TextureParameter::WrapR(_) => WebGl2RenderingContext::TEXTURE_WRAP_R,
            TextureParameter::BaseLevel(_) => WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            TextureParameter::CompareFunc(_) => WebGl2RenderingContext::TEXTURE_COMPARE_FUNC,
            TextureParameter::CompareMode(_) => WebGl2RenderingContext::TEXTURE_COMPARE_MODE,
            TextureParameter::MaxLevel(_) => WebGl2RenderingContext::TEXTURE_MAX_LEVEL,
            TextureParameter::MaxLod(_) => WebGl2RenderingContext::TEXTURE_MAX_LOD,
            TextureParameter::MinLod(_) => WebGl2RenderingContext::TEXTURE_MIN_LOD,
        }
    }
}

impl ToGlEnum for StencilFunction {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            StencilFunction::Never => WebGl2RenderingContext::NEVER,
            StencilFunction::Less => WebGl2RenderingContext::LESS,
            StencilFunction::Equal => WebGl2RenderingContext::EQUAL,
            StencilFunction::LessEqual => WebGl2RenderingContext::LEQUAL,
            StencilFunction::Greater => WebGl2RenderingContext::GREATER,
            StencilFunction::NotEqual => WebGl2RenderingContext::NOTEQUAL,
            StencilFunction::GreaterEqual => WebGl2RenderingContext::GEQUAL,
            StencilFunction::Always => WebGl2RenderingContext::ALWAYS,
        }
    }
}

impl ToGlEnum for StencilOp {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            StencilOp::Keep => WebGl2RenderingContext::KEEP,
            StencilOp::Zero => WebGl2RenderingContext::ZERO,
            StencilOp::Replace => WebGl2RenderingContext::REPLACE,
            StencilOp::Increment => WebGl2RenderingContext::INCR,
            StencilOp::IncrementWrap => WebGl2RenderingContext::INCR_WRAP,
            StencilOp::Decrement => WebGl2RenderingContext::DECR,
            StencilOp::DecrementWrap => WebGl2RenderingContext::DECR_WRAP,
            StencilOp::Invert => WebGl2RenderingContext::INVERT,
        }
    }
}

impl ToGlEnum for RenderbufferInternalFormat {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            RenderbufferInternalFormat::R8 => WebGl2RenderingContext::R8,
            RenderbufferInternalFormat::R8UI => WebGl2RenderingContext::R8UI,
            RenderbufferInternalFormat::R8I => WebGl2RenderingContext::R8I,
            RenderbufferInternalFormat::R16UI => WebGl2RenderingContext::R16UI,
            RenderbufferInternalFormat::R16I => WebGl2RenderingContext::R16I,
            RenderbufferInternalFormat::R32UI => WebGl2RenderingContext::R32UI,
            RenderbufferInternalFormat::R32I => WebGl2RenderingContext::R32I,
            RenderbufferInternalFormat::RG8 => WebGl2RenderingContext::RG8,
            RenderbufferInternalFormat::RG8UI => WebGl2RenderingContext::RG8UI,
            RenderbufferInternalFormat::RG8I => WebGl2RenderingContext::RG8I,
            RenderbufferInternalFormat::RG16UI => WebGl2RenderingContext::RG16UI,
            RenderbufferInternalFormat::RG16I => WebGl2RenderingContext::RG16I,
            RenderbufferInternalFormat::RG32UI => WebGl2RenderingContext::RG32UI,
            RenderbufferInternalFormat::RG32I => WebGl2RenderingContext::RG32I,
            RenderbufferInternalFormat::RGB8 => WebGl2RenderingContext::RGB8,
            RenderbufferInternalFormat::RGBA8 => WebGl2RenderingContext::RGBA8,
            RenderbufferInternalFormat::RGB10_A2 => WebGl2RenderingContext::RGB10_A2,
            RenderbufferInternalFormat::RGBA8UI => WebGl2RenderingContext::RGBA8UI,
            RenderbufferInternalFormat::RGBA8I => WebGl2RenderingContext::RGBA8I,
            RenderbufferInternalFormat::RGB10_A2UI => WebGl2RenderingContext::RGB10_A2UI,
            RenderbufferInternalFormat::RGBA16UI => WebGl2RenderingContext::RGBA16UI,
            RenderbufferInternalFormat::RGBA16I => WebGl2RenderingContext::RGBA16I,
            RenderbufferInternalFormat::RGBA32UI => WebGl2RenderingContext::RGBA32UI,
            RenderbufferInternalFormat::RGBA32I => WebGl2RenderingContext::RGBA32I,
            RenderbufferInternalFormat::R16F => WebGl2RenderingContext::R16F,
            RenderbufferInternalFormat::RG16F => WebGl2RenderingContext::RG16F,
            RenderbufferInternalFormat::RGBA16F => WebGl2RenderingContext::RGBA16F,
            RenderbufferInternalFormat::R32F => WebGl2RenderingContext::R32F,
            RenderbufferInternalFormat::RG32F => WebGl2RenderingContext::RG32F,
            RenderbufferInternalFormat::RGBA32F => WebGl2RenderingContext::RGBA32F,
            RenderbufferInternalFormat::R11F_G11F_B10F => WebGl2RenderingContext::R11F_G11F_B10F,
            RenderbufferInternalFormat::SRGB => WebGl2RenderingContext::SRGB,
            RenderbufferInternalFormat::SRGB8 => WebGl2RenderingContext::SRGB8,
            RenderbufferInternalFormat::SRGB8_ALPHA8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            RenderbufferInternalFormat::RGBA4 => WebGl2RenderingContext::RGBA4,
            RenderbufferInternalFormat::RGB565 => WebGl2RenderingContext::RGB565,
            RenderbufferInternalFormat::RGB5_A1 => WebGl2RenderingContext::RGB5_A1,
            RenderbufferInternalFormat::DEPTH_COMPONENT16 => {
                WebGl2RenderingContext::DEPTH_COMPONENT16
            }
            RenderbufferInternalFormat::DEPTH_COMPONENT24 => {
                WebGl2RenderingContext::DEPTH_COMPONENT24
            }
            RenderbufferInternalFormat::DEPTH_COMPONENT32F => {
                WebGl2RenderingContext::DEPTH_COMPONENT32F
            }
            RenderbufferInternalFormat::STENCIL_INDEX8 => WebGl2RenderingContext::STENCIL_INDEX8,
            RenderbufferInternalFormat::DEPTH_STENCIL => WebGl2RenderingContext::DEPTH_STENCIL,
            RenderbufferInternalFormat::DEPTH24_STENCIL8 => {
                WebGl2RenderingContext::DEPTH24_STENCIL8
            }
            RenderbufferInternalFormat::DEPTH32F_STENCIL8 => {
                WebGl2RenderingContext::DEPTH32F_STENCIL8
            }
        }
    }
}

impl ToGlEnum for FramebufferTarget {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            FramebufferTarget::FRAMEBUFFER => WebGl2RenderingContext::FRAMEBUFFER,
            FramebufferTarget::READ_FRAMEBUFFER => WebGl2RenderingContext::READ_FRAMEBUFFER,
            FramebufferTarget::DRAW_FRAMEBUFFER => WebGl2RenderingContext::DRAW_FRAMEBUFFER,
        }
    }
}

impl ToGlEnum for FramebufferAttachment {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => WebGl2RenderingContext::COLOR_ATTACHMENT0,
            FramebufferAttachment::COLOR_ATTACHMENT1 => WebGl2RenderingContext::COLOR_ATTACHMENT1,
            FramebufferAttachment::COLOR_ATTACHMENT2 => WebGl2RenderingContext::COLOR_ATTACHMENT2,
            FramebufferAttachment::COLOR_ATTACHMENT3 => WebGl2RenderingContext::COLOR_ATTACHMENT3,
            FramebufferAttachment::COLOR_ATTACHMENT4 => WebGl2RenderingContext::COLOR_ATTACHMENT4,
            FramebufferAttachment::COLOR_ATTACHMENT5 => WebGl2RenderingContext::COLOR_ATTACHMENT5,
            FramebufferAttachment::COLOR_ATTACHMENT6 => WebGl2RenderingContext::COLOR_ATTACHMENT6,
            FramebufferAttachment::COLOR_ATTACHMENT7 => WebGl2RenderingContext::COLOR_ATTACHMENT7,
            FramebufferAttachment::COLOR_ATTACHMENT8 => WebGl2RenderingContext::COLOR_ATTACHMENT8,
            FramebufferAttachment::COLOR_ATTACHMENT9 => WebGl2RenderingContext::COLOR_ATTACHMENT9,
            FramebufferAttachment::COLOR_ATTACHMENT10 => WebGl2RenderingContext::COLOR_ATTACHMENT10,
            FramebufferAttachment::COLOR_ATTACHMENT11 => WebGl2RenderingContext::COLOR_ATTACHMENT11,
            FramebufferAttachment::COLOR_ATTACHMENT12 => WebGl2RenderingContext::COLOR_ATTACHMENT12,
            FramebufferAttachment::COLOR_ATTACHMENT13 => WebGl2RenderingContext::COLOR_ATTACHMENT13,
            FramebufferAttachment::COLOR_ATTACHMENT14 => WebGl2RenderingContext::COLOR_ATTACHMENT14,
            FramebufferAttachment::COLOR_ATTACHMENT15 => WebGl2RenderingContext::COLOR_ATTACHMENT15,
            FramebufferAttachment::DEPTH_ATTACHMENT => WebGl2RenderingContext::DEPTH_ATTACHMENT,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => {
                WebGl2RenderingContext::DEPTH_STENCIL_ATTACHMENT
            }
            FramebufferAttachment::STENCIL_ATTACHMENT => WebGl2RenderingContext::STENCIL_ATTACHMENT,
        }
    }
}

impl ToGlEnum for FramebufferSource {
    #[inline]
    fn gl_enum(&self) -> GLenum {
        match self {
            FramebufferSource::NONE => WebGl2RenderingContext::NONE,
            FramebufferSource::BACK => WebGl2RenderingContext::BACK,
            FramebufferSource::COLOR_ATTACHMENT0 => WebGl2RenderingContext::COLOR_ATTACHMENT0,
            FramebufferSource::COLOR_ATTACHMENT1 => WebGl2RenderingContext::COLOR_ATTACHMENT1,
            FramebufferSource::COLOR_ATTACHMENT2 => WebGl2RenderingContext::COLOR_ATTACHMENT2,
            FramebufferSource::COLOR_ATTACHMENT3 => WebGl2RenderingContext::COLOR_ATTACHMENT3,
            FramebufferSource::COLOR_ATTACHMENT4 => WebGl2RenderingContext::COLOR_ATTACHMENT4,
            FramebufferSource::COLOR_ATTACHMENT5 => WebGl2RenderingContext::COLOR_ATTACHMENT5,
            FramebufferSource::COLOR_ATTACHMENT6 => WebGl2RenderingContext::COLOR_ATTACHMENT6,
            FramebufferSource::COLOR_ATTACHMENT7 => WebGl2RenderingContext::COLOR_ATTACHMENT7,
            FramebufferSource::COLOR_ATTACHMENT8 => WebGl2RenderingContext::COLOR_ATTACHMENT8,
            FramebufferSource::COLOR_ATTACHMENT9 => WebGl2RenderingContext::COLOR_ATTACHMENT9,
            FramebufferSource::COLOR_ATTACHMENT10 => WebGl2RenderingContext::COLOR_ATTACHMENT10,
            FramebufferSource::COLOR_ATTACHMENT11 => WebGl2RenderingContext::COLOR_ATTACHMENT11,
            FramebufferSource::COLOR_ATTACHMENT12 => WebGl2RenderingContext::COLOR_ATTACHMENT12,
            FramebufferSource::COLOR_ATTACHMENT13 => WebGl2RenderingContext::COLOR_ATTACHMENT13,
            FramebufferSource::COLOR_ATTACHMENT14 => WebGl2RenderingContext::COLOR_ATTACHMENT14,
            FramebufferSource::COLOR_ATTACHMENT15 => WebGl2RenderingContext::COLOR_ATTACHMENT15,
        }
    }
}
