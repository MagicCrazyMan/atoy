use web_sys::WebGl2RenderingContext;

use super::{
    buffer::{BufferDataType, BufferTarget, BufferUsage},
    draw::{CullFace, DrawElementType, DrawMode},
    texture::{
        TextureCompareFunction, TextureCompareMode, TextureDataType, TextureFormat,
        TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
        TexturePixelStorage, TextureUnit, TextureUnpackColorSpaceConversion, TextureWrapMethod,
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
    fn gl_enum(&self) -> GLenum {
        match self {
            DrawElementType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            DrawElementType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            DrawElementType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
        }
    }
}

impl ToGlEnum for DrawMode {
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
    fn gl_enum(&self) -> GLenum {
        match self {
            CullFace::Front => WebGl2RenderingContext::FRONT,
            CullFace::Back => WebGl2RenderingContext::BACK,
            CullFace::Both => WebGl2RenderingContext::FRONT_AND_BACK,
        }
    }
}

impl ToGlEnum for TextureFormat {
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureFormat::RGB => WebGl2RenderingContext::RGB,
            TextureFormat::RGBA => WebGl2RenderingContext::RGBA,
            TextureFormat::Luminance => WebGl2RenderingContext::LUMINANCE,
            TextureFormat::LuminanceAlpha => WebGl2RenderingContext::LUMINANCE_ALPHA,
            TextureFormat::Alpha => WebGl2RenderingContext::ALPHA,
            TextureFormat::SRGB => WebGl2RenderingContext::SRGB,
            TextureFormat::SRGBA8 => WebGl2RenderingContext::SRGB8,
            TextureFormat::SRGB8_ALPHA8 => WebGl2RenderingContext::SRGB8_ALPHA8,
            TextureFormat::R8 => WebGl2RenderingContext::R8,
            TextureFormat::R16F => WebGl2RenderingContext::R16F,
            TextureFormat::R32F => WebGl2RenderingContext::R32F,
            TextureFormat::R8UI => WebGl2RenderingContext::R8UI,
            TextureFormat::RG8 => WebGl2RenderingContext::RG8,
            TextureFormat::RG16F => WebGl2RenderingContext::RG16F,
            TextureFormat::RG32F => WebGl2RenderingContext::RG32F,
            TextureFormat::RG8UI => WebGl2RenderingContext::RG8UI,
            TextureFormat::RG16UI => WebGl2RenderingContext::RG16UI,
            TextureFormat::RG32UI => WebGl2RenderingContext::RG32UI,
            TextureFormat::SRGB8 => WebGl2RenderingContext::SRGB8,
            TextureFormat::RGB565 => WebGl2RenderingContext::RGB565,
            TextureFormat::R11F_G11F_B10F => WebGl2RenderingContext::R11F_G11F_B10F,
            TextureFormat::RGB9_E5 => WebGl2RenderingContext::RGB9_E5,
            TextureFormat::RGB16F => WebGl2RenderingContext::RGB16F,
            TextureFormat::RGB32F => WebGl2RenderingContext::RGB32F,
            TextureFormat::RGB8UI => WebGl2RenderingContext::RGB8UI,
            TextureFormat::RGBA8 => WebGl2RenderingContext::RGBA8,
            TextureFormat::RGB5_A1 => WebGl2RenderingContext::RGB5_A1,
            TextureFormat::RGB10_A2 => WebGl2RenderingContext::RGB10_A2,
            TextureFormat::RGBA4 => WebGl2RenderingContext::RGBA4,
            TextureFormat::RGBA16F => WebGl2RenderingContext::RGBA16F,
            TextureFormat::RGBA32F => WebGl2RenderingContext::RGBA32F,
            TextureFormat::RGBA8UI => WebGl2RenderingContext::RGBA8UI,
        }
    }
}

impl ToGlEnum for TextureDataType {
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureDataType::Float => WebGl2RenderingContext::FLOAT,
            TextureDataType::HalfFloat => WebGl2RenderingContext::HALF_FLOAT,
            TextureDataType::Byte => WebGl2RenderingContext::BYTE,
            TextureDataType::Short => WebGl2RenderingContext::SHORT,
            TextureDataType::Int => WebGl2RenderingContext::INT,
            TextureDataType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            TextureDataType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            TextureDataType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
            TextureDataType::UnsignedShort_5_6_5 => WebGl2RenderingContext::UNSIGNED_SHORT_5_6_5,
            TextureDataType::UnsignedShort_4_4_4_4 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_4_4_4_4
            }
            TextureDataType::UnsignedShort_5_5_5_1 => {
                WebGl2RenderingContext::UNSIGNED_SHORT_5_5_5_1
            }
            TextureDataType::UnsignedInt_2_10_10_10_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
            TextureDataType::UnsignedInt_10F_11F_11F_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_10F_11F_11F_REV
            }
            TextureDataType::UnsignedInt_5_9_9_9_REV => {
                WebGl2RenderingContext::UNSIGNED_INT_5_9_9_9_REV
            }
            TextureDataType::UnsignedInt_24_8 => WebGl2RenderingContext::UNSIGNED_INT_24_8,
            TextureDataType::Float_32_UnsignedInt_24_8_REV => {
                WebGl2RenderingContext::FLOAT_32_UNSIGNED_INT_24_8_REV
            }
        }
    }
}

impl ToGlEnum for TextureUnit {
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
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureMagnificationFilter::Linear => WebGl2RenderingContext::LINEAR,
            TextureMagnificationFilter::Nearest => WebGl2RenderingContext::NEAREST,
        }
    }
}

impl ToGlEnum for TextureMinificationFilter {
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
    fn gl_enum(&self) -> GLenum {
        match self {
            TextureWrapMethod::Repeat => WebGl2RenderingContext::REPEAT,
            TextureWrapMethod::ClampToEdge => WebGl2RenderingContext::CLAMP_TO_EDGE,
            TextureWrapMethod::MirroredRepeat => WebGl2RenderingContext::MIRRORED_REPEAT,
        }
    }
}

impl ToGlEnum for TextureCompareFunction {
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
