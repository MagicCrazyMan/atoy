use std::{cell::RefCell, collections::HashMap};

use uuid::Uuid;
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlTexture};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    RGB,
    RGBA,
    Luminance,
    LuminanceAlpha,
    Alpha,
    SRGB,
    SRGBA8,
    SRGB8_ALPHA8,
    R8,
    R16F,
    R32F,
    R8UI,
    RG8,
    RG16F,
    RG32F,
    RG8UI,
    RG16UI,
    RG32UI,
    SRGB8,
    RGB565,
    R11F_G11F_B10F,
    RGB9_E5,
    RGB16F,
    RGB32F,
    RGB8UI,
    RGBA8,
    RGB5_A1,
    RGB10_A2,
    RGBA4,
    RGBA16F,
    RGBA32F,
    RGBA8UI,
}

impl TextureFormat {
    pub fn to_gl_enum(&self) -> u32 {
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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureDataType {
    Float,
    HalfFloat,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    UnsignedShort_5_6_5,
    UnsignedShort_4_4_4_4,
    UnsignedShort_5_5_5_1,
    UnsignedInt_2_10_10_10_REV,
    UnsignedInt_10F_11F_11F_REV,
    UnsignedInt_5_9_9_9_REV,
    UnsignedInt_24_8,
    Float_32_UnsignedInt_24_8_REV,
}

impl TextureDataType {
    pub fn to_gl_enum(&self) -> u32 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureUnpackColorSpaceConversion {
    None,
    BrowserDefaultWebgl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexturePixelStorage {
    PackAlignment(i32),
    UnpackAlignment(i32),
    UnpackFlipYWebGL(bool),
    UnpackPremultiplyAlphaWebgl(bool),
    UnpackColorspaceConversionWebgl(TextureUnpackColorSpaceConversion),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
    UnpackRowLength(i32),
    UnpackImageHeight(i32),
    UnpackSkipPixels(i32),
    UnpackSkipRows(i32),
    UnpackSkipImages(i32),
}

impl TexturePixelStorage {
    pub fn key(&self) -> u32 {
        match self {
            TexturePixelStorage::PackAlignment(_) => WebGl2RenderingContext::PACK_ALIGNMENT,
            TexturePixelStorage::UnpackAlignment(_) => WebGl2RenderingContext::UNPACK_ALIGNMENT,
            TexturePixelStorage::UnpackFlipYWebGL(_) => WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
            TexturePixelStorage::UnpackPremultiplyAlphaWebgl(_) => {
                WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL
            }
            TexturePixelStorage::UnpackColorspaceConversionWebgl(_) => {
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

    pub fn value(&self) -> i32 {
        match self {
            TexturePixelStorage::UnpackFlipYWebGL(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            TexturePixelStorage::UnpackPremultiplyAlphaWebgl(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            TexturePixelStorage::UnpackColorspaceConversionWebgl(v) => match v {
                TextureUnpackColorSpaceConversion::None => WebGl2RenderingContext::NONE as i32,
                TextureUnpackColorSpaceConversion::BrowserDefaultWebgl => {
                    WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32
                }
            },
            TexturePixelStorage::PackAlignment(v)
            | TexturePixelStorage::UnpackAlignment(v)
            | TexturePixelStorage::PackRowLength(v)
            | TexturePixelStorage::PackSkipPixels(v)
            | TexturePixelStorage::PackSkipRows(v)
            | TexturePixelStorage::UnpackRowLength(v)
            | TexturePixelStorage::UnpackImageHeight(v)
            | TexturePixelStorage::UnpackSkipPixels(v)
            | TexturePixelStorage::UnpackSkipRows(v)
            | TexturePixelStorage::UnpackSkipImages(v) => *v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureMagnificationFilter {
    Linear,
    Nearest,
}

impl TextureMagnificationFilter {
    pub fn value(&self) -> i32 {
        match self {
            TextureMagnificationFilter::Linear => WebGl2RenderingContext::LINEAR as i32,
            TextureMagnificationFilter::Nearest => WebGl2RenderingContext::NEAREST as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureMinificationFilter {
    Linear,
    Nearest,
    NearestMipmapNearest,
    LinearMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapLinear,
}

impl TextureMinificationFilter {
    pub fn value(&self) -> i32 {
        match self {
            TextureMinificationFilter::Linear => WebGl2RenderingContext::LINEAR as i32,
            TextureMinificationFilter::Nearest => WebGl2RenderingContext::NEAREST as i32,
            TextureMinificationFilter::NearestMipmapNearest => {
                WebGl2RenderingContext::NEAREST_MIPMAP_NEAREST as i32
            }
            TextureMinificationFilter::LinearMipmapNearest => {
                WebGl2RenderingContext::LINEAR_MIPMAP_NEAREST as i32
            }
            TextureMinificationFilter::NearestMipmapLinear => {
                WebGl2RenderingContext::NEAREST_MIPMAP_LINEAR as i32
            }
            TextureMinificationFilter::LinearMipmapLinear => {
                WebGl2RenderingContext::LINEAR_MIPMAP_LINEAR as i32
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureWrapMethod {
    Repeat,
    ClampToEdge,
    MirroredRepeat,
}

impl TextureWrapMethod {
    pub fn value(&self) -> i32 {
        match self {
            TextureWrapMethod::Repeat => WebGl2RenderingContext::REPEAT as i32,
            TextureWrapMethod::ClampToEdge => WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            TextureWrapMethod::MirroredRepeat => WebGl2RenderingContext::MIRRORED_REPEAT as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureCompareFunction {
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    Equal,
    NotEqual,
    Always,
    Never,
}

impl TextureCompareFunction {
    pub fn value(&self) -> i32 {
        match self {
            TextureCompareFunction::LessEqual => WebGl2RenderingContext::LEQUAL as i32,
            TextureCompareFunction::GreaterEqual => WebGl2RenderingContext::GEQUAL as i32,
            TextureCompareFunction::Less => WebGl2RenderingContext::LESS as i32,
            TextureCompareFunction::Greater => WebGl2RenderingContext::GREATER as i32,
            TextureCompareFunction::Equal => WebGl2RenderingContext::EQUAL as i32,
            TextureCompareFunction::NotEqual => WebGl2RenderingContext::NOTEQUAL as i32,
            TextureCompareFunction::Always => WebGl2RenderingContext::ALWAYS as i32,
            TextureCompareFunction::Never => WebGl2RenderingContext::NEVER as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureCompareMode {
    None,
    CompareRefToTexture,
}

impl TextureCompareMode {
    pub fn value(&self) -> i32 {
        match self {
            TextureCompareMode::None => WebGl2RenderingContext::NONE as i32,
            TextureCompareMode::CompareRefToTexture => {
                WebGl2RenderingContext::COMPARE_REF_TO_TEXTURE as i32
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextureParameter {
    MagFilter(TextureMagnificationFilter),
    MinFilter(TextureMinificationFilter),
    WrapS(TextureWrapMethod),
    WrapT(TextureWrapMethod),
    WrapR(TextureWrapMethod),
    BaseLevel(i32),
    CompareFunc(TextureCompareFunction),
    CompareMode(TextureCompareMode),
    MaxLevel(i32),
    MaxLod(f32),
    MinLod(f32),
}

impl TextureParameter {
    pub(super) fn tex_parameteri(&self, gl: &WebGl2RenderingContext, target: u32) {
        match self {
            TextureParameter::MagFilter(v) => gl.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                v.value(),
            ),
            TextureParameter::MinFilter(v) => gl.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                v.value(),
            ),
            TextureParameter::WrapS(v) => {
                gl.tex_parameteri(target, WebGl2RenderingContext::TEXTURE_WRAP_S, v.value())
            }
            TextureParameter::WrapT(v) => {
                gl.tex_parameteri(target, WebGl2RenderingContext::TEXTURE_WRAP_T, v.value())
            }
            TextureParameter::WrapR(v) => {
                gl.tex_parameteri(target, WebGl2RenderingContext::TEXTURE_WRAP_R, v.value())
            }
            TextureParameter::BaseLevel(v) => {
                gl.tex_parameteri(target, WebGl2RenderingContext::TEXTURE_BASE_LEVEL, *v)
            }
            TextureParameter::CompareFunc(v) => gl.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_COMPARE_FUNC,
                v.value(),
            ),
            TextureParameter::CompareMode(v) => gl.tex_parameteri(
                target,
                WebGl2RenderingContext::TEXTURE_COMPARE_MODE,
                v.value(),
            ),
            TextureParameter::MaxLevel(v) => {
                gl.tex_parameteri(target, WebGl2RenderingContext::TEXTURE_MAX_LEVEL, *v)
            }
            TextureParameter::MaxLod(v) => {
                gl.tex_parameterf(target, WebGl2RenderingContext::TEXTURE_MAX_LOD, *v)
            }
            TextureParameter::MinLod(v) => {
                gl.tex_parameterf(target, WebGl2RenderingContext::TEXTURE_MIN_LOD, *v)
            }
        }
    }
}

enum TextureData {
    Preallocate {
        // target: TextureTarget,
        level: i32,
        internal_format: TextureFormat,
        width: i32,
        height: i32,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromBinary {
        // target: TextureTarget,
        level: i32,
        internal_format: TextureFormat,
        width: i32,
        height: i32,
        data: Box<dyn AsRef<[u8]>>,
        format: TextureFormat,
        data_type: TextureDataType,
        src_offset: u32,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlCanvasElement {
        // target: TextureTarget,
        level: i32,
        internal_format: TextureFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlCanvasElementWithSize {
        // target: TextureTarget,
        level: i32,
        internal_format: TextureFormat,
        width: i32,
        height: i32,
        format: TextureFormat,
        data_type: TextureDataType,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlImageElement {
        // target: TextureTarget,
        level: i32,
        internal_format: TextureFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        image: Box<dyn AsRef<HtmlImageElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlImageElementWithSize {
        // target: TextureTarget,
        level: i32,
        format: TextureFormat,
        width: i32,
        height: i32,
        internal_format: TextureFormat,
        data_type: TextureDataType,
        image: Box<dyn AsRef<HtmlImageElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
}

impl TextureData {
    fn pixel_storages(&self) -> &[TexturePixelStorage] {
        match self {
            TextureData::Preallocate { pixel_storages, .. } => &pixel_storages,
            TextureData::FromBinary { pixel_storages, .. } => &pixel_storages,
            TextureData::FromHtmlCanvasElement { pixel_storages, .. } => &pixel_storages,
            TextureData::FromHtmlCanvasElementWithSize { pixel_storages, .. } => &pixel_storages,
            TextureData::FromHtmlImageElement { pixel_storages, .. } => &pixel_storages,
            TextureData::FromHtmlImageElementWithSize { pixel_storages, .. } => &pixel_storages,
        }
    }

    fn tex_image(&self, gl: &WebGl2RenderingContext, tex_target: u32) -> Result<(), String> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image data
        let result = match self {
            TextureData::Preallocate {
                level,
                internal_format,
                width,
                height,
                format,
                data_type,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                *width,
                *height,
                0,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                None
            ),
            TextureData::FromBinary {
                level,
                internal_format,
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                *width,
                *height,
                0,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                data.as_ref().as_ref(),
                *src_offset
            ),
            TextureData::FromHtmlCanvasElement {
                level,
                internal_format,
                format,
                data_type,
                canvas,
                ..
            } => gl
            .tex_image_2d_with_u32_and_u32_and_html_canvas_element(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureData::FromHtmlCanvasElementWithSize {
                level,
                internal_format,
                width,
                height,
                format,
                data_type,
                canvas,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_canvas_element(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                *width,
                *height,
                0,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                canvas.as_ref().as_ref()
            ),
            TextureData::FromHtmlImageElement {
                level,
                internal_format,
                format,
                data_type,
                image,
                ..
            } => gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                image.as_ref().as_ref(),
            ),
            TextureData::FromHtmlImageElementWithSize {
                level,
                format,
                width,
                height,
                internal_format,
                data_type,
                image,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_image_element(
                tex_target,
                *level,
                internal_format.to_gl_enum() as i32,
                *width,
                *height,
                0,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                image.as_ref().as_ref()
            ),
        };

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                // should log error
                console_log!("{:?}", err);
                Err(err
                    .as_string()
                    .unwrap_or(String::from("unknown error during tex image 2d")))
            }
        }
    }

    fn tex_sub_image(&self, gl: &WebGl2RenderingContext, tex_target: u32) -> Result<(), String> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image data
        let result = match self {
            TextureData::Preallocate {
                level,
                width,
                height,
                format,
                data_type,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                None,
            ),
            TextureData::FromBinary {
                level,
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                data.as_ref().as_ref(),
                *src_offset,
            ),
            TextureData::FromHtmlCanvasElement {
                level,
                format,
                data_type,
                canvas,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_u32_and_u32_and_html_canvas_element(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureData::FromHtmlCanvasElementWithSize {
                level,
                width,
                height,
                format,
                data_type,
                canvas,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureData::FromHtmlImageElement {
                level,
                format,
                data_type,
                image,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                image.as_ref().as_ref(),
            ),
            TextureData::FromHtmlImageElementWithSize {
                level,
                format,
                width,
                height,
                data_type,
                image,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                tex_target,
                *level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.to_gl_enum(),
                data_type.to_gl_enum(),
                image.as_ref().as_ref(),
            ),
        };

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                // should log error
                console_log!("{:?}", err);
                Err(err
                    .as_string()
                    .unwrap_or(String::from("unknown error during tex image 2d")))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextureKind {
    Texture2D,
    TextureCubeMap,
}

impl TextureKind {
    pub fn target(&self) -> u32 {
        match self {
            TextureKind::Texture2D { .. } => WebGl2RenderingContext::TEXTURE_2D,
            TextureKind::TextureCubeMap { .. } => WebGl2RenderingContext::TEXTURE_CUBE_MAP,
        }
    }

    fn tex_image(
        &self,
        gl: &WebGl2RenderingContext,
        data: &[Option<TextureData>; 6],
    ) -> Result<(), String> {
        match self {
            TextureKind::Texture2D => {
                data[0]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_2D)?;
            }
            TextureKind::TextureCubeMap => {
                data[0]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X)?;
                data[1]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X)?;
                data[2]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y)?;
                data[3]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y)?;
                data[4]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z)?;
                data[5]
                    .as_ref()
                    .unwrap()
                    .tex_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z)?;
            }
        };

        Ok(())
    }

    fn tex_sub_image(
        &self,
        gl: &WebGl2RenderingContext,
        data: &[Option<TextureData>; 6],
    ) -> Result<(), String> {
        match self {
            TextureKind::Texture2D => {
                data[0]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_2D)?;
            }
            TextureKind::TextureCubeMap => {
                data[0]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X)?;
                data[1]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X)?;
                data[2]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y)?;
                data[3]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y)?;
                data[4]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z)?;
                data[5]
                    .as_ref()
                    .unwrap()
                    .tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z)?;
            }
        };

        Ok(())
    }
}

enum TextureStatus {
    Unchanged {
        id: Uuid,
        kind: TextureKind,
    },
    UpdateTexture {
        id: Option<Uuid>,
        kind: TextureKind,
        data: [Option<TextureData>; 6],
    },
    UpdateSubTexture {
        id: Uuid,
        kind: TextureKind,
        data: [Option<TextureData>; 6],
    },
}

pub struct TextureDescriptor {
    status: RefCell<TextureStatus>,
    generate_mipmap: bool,
}

impl TextureDescriptor {
    pub fn texture_2d_with_html_image_element<I: AsRef<HtmlImageElement> + 'static>(
        // target: TextureTarget,
        image: I,
        data_type: TextureDataType,
        internal_format: TextureFormat,
        format: TextureFormat,
        level: i32,
        pixel_storages: Vec<TexturePixelStorage>,
        generate_mipmap: bool,
    ) -> Self {
        Self {
            status: RefCell::new(TextureStatus::UpdateTexture {
                id: None,
                kind: TextureKind::Texture2D,
                data: [
                    Some(TextureData::FromHtmlImageElement {
                        image: Box::new(image),
                        format,
                        level,
                        internal_format,
                        data_type,
                        pixel_storages,
                        x_offset: 0,
                        y_offset: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            }),
            generate_mipmap,
        }
    }
}

pub struct TextureStore {
    gl: WebGl2RenderingContext,
    store: HashMap<Uuid, WebGlTexture>,
}

impl TextureStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: HashMap::new(),
        }
    }

    pub fn texture_or_create(
        &mut self,
        TextureDescriptor {
            status,
            generate_mipmap,
        }: &TextureDescriptor,
    ) -> Result<(u32, &WebGlTexture), String> {
        let mut status = status.borrow_mut();
        match &*status {
            TextureStatus::Unchanged { id, kind } => match self.store.get(id) {
                Some(texture) => Ok((kind.target(), texture)),
                None => Err(format!("failed to get texture with id {}", id)),
            },
            TextureStatus::UpdateTexture { id, kind, data } => {
                // delete old texture
                if let Some(texture) = id.as_ref().and_then(|id| self.store.remove(id)) {
                    self.gl.delete_texture(Some(&texture));
                }

                // create texture
                let Some(texture) = self.gl.create_texture() else {
                    return Err(String::from("failed to create texture"));
                };

                let kind = *kind;
                // binds texture
                self.gl.bind_texture(kind.target(), Some(&texture));
                // buffer images
                kind.tex_image(&self.gl, data)?;
                // generates mipmaps
                if *generate_mipmap {
                    self.gl.generate_mipmap(kind.target());
                }

                // unbinds for good practice
                self.gl.bind_texture(kind.target(), None);

                // stores it
                let id = Uuid::new_v4();
                let texture = self.store.entry(id.clone()).or_insert(texture);

                // updates status
                *status = TextureStatus::Unchanged { id, kind };

                Ok((kind.target(), texture))
            }
            TextureStatus::UpdateSubTexture { id, kind, data } => {
                let Some(texture) = self.store.get(id) else {
                    return Err(format!("failed to get texture with id {}", id));
                };

                let kind = *kind;
                // binds texture
                self.gl.bind_texture(kind.target(), Some(texture));
                // buffers images
                kind.tex_sub_image(&self.gl, data)?;
                // generates mipmaps
                if *generate_mipmap {
                    self.gl.generate_mipmap(kind.target());
                }
                // unbinds for good practice
                self.gl.bind_texture(kind.target(), None);

                // updates status
                *status = TextureStatus::Unchanged {
                    id: id.clone(),
                    kind,
                };

                Ok((kind.target(), texture))
            }
        }
    }
}
