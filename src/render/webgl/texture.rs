use std::{cell::RefCell, collections::HashMap};

use uuid::Uuid;
use wasm_bindgen_test::console_log;
use web_sys::{
    js_sys::Object, HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlTexture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureTarget {
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
}

impl TextureTarget {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            TextureTarget::Texture2D => WebGl2RenderingContext::TEXTURE_2D,
            TextureTarget::TextureCubeMapPositiveX => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X
            }
            TextureTarget::TextureCubeMapNegativeX => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X
            }
            TextureTarget::TextureCubeMapPositiveY => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y
            }
            TextureTarget::TextureCubeMapNegativeY => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y
            }
            TextureTarget::TextureCubeMapPositiveZ => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z
            }
            TextureTarget::TextureCubeMapNegativeZ => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z
            }
        }
    }
}

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
    pub fn key(&self) -> u32 {
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

enum TextureData {
    Preallocate {
        target: TextureTarget,
        width: i32,
        height: i32,
        format: TextureFormat,
        level: i32,
    },
    FromBinary {
        target: TextureTarget,
        width: i32,
        height: i32,
        data: Box<dyn AsRef<[u8]>>,
        format: TextureFormat,
        src_offset: u32,
        level: i32,
    },
    FromHtmlCanvasElement {
        target: TextureTarget,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromHtmlCanvasElementWithSize {
        target: TextureTarget,
        width: i32,
        height: i32,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromHtmlImageElement {
        target: TextureTarget,
        image: Box<dyn AsRef<HtmlImageElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromHtmlImageElementWithSize {
        target: TextureTarget,
        width: i32,
        height: i32,
        image: Box<dyn AsRef<HtmlImageElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromObject {
        target: TextureTarget,
        width: i32,
        height: i32,
        object: Box<dyn AsRef<Object>>,
        format: TextureFormat,
        src_offset: u32,
        level: i32,
    },
}

enum TextureSubData {
    Clear {
        target: TextureTarget,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: TextureFormat,
        level: i32,
    },
    FromBinary {
        target: TextureTarget,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        data: Box<dyn AsRef<[u8]>>,
        src_offset: u32,
        format: TextureFormat,
        level: i32,
    },
    FromHtmlCanvasElement {
        target: TextureTarget,
        x_offset: i32,
        y_offset: i32,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromHtmlCanvasElementWithSize {
        target: TextureTarget,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        format: TextureFormat,
        level: i32,
    },
    FromObject {
        target: TextureTarget,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        canvas: Box<dyn AsRef<Object>>,
        src_offset: u32,
        format: TextureFormat,
        level: i32,
    },
}

enum TextureStatus {
    Unchanged {
        id: Uuid,
    },
    UpdateTexture {
        id: Option<Uuid>,
        list: Vec<TextureData>,
    },
    UpdateSubTexture {
        id: Uuid,
        list: Vec<TextureSubData>,
    },
}

pub struct TextureDescriptor {
    status: RefCell<TextureStatus>,
    internal_format: TextureFormat,
    data_type: TextureDataType,
    pixel_storages: Vec<TexturePixelStorage>,
    generate_mipmap: bool,
}

impl TextureDescriptor {
    // pub fn preallocate<P: Into<Vec<TexturePixelStorageParam>>>(
    //     data_type: TextureDataType,
    //     width: i32,
    //     height: i32,
    //     internal_format: TextureFormat,
    //     format: TextureFormat,
    //     level: i32,
    //     generate_mipmap: bool,
    //     params: P,
    // ) -> Self {
    //     Self {
    //         status: RefCell::new(TextureStatus::UpdateTexture {
    //             id: None,
    //             data: TextureData::Preallocate {
    //                 width,
    //                 height,
    //                 format,
    //                 level,
    //             },
    //         }),
    //         internal_format,
    //         data_type,
    //         params: params.into(),
    //         generate_mipmap,
    //     }
    // }

    // pub fn with_binary<D: AsRef<[u8]> + 'static, P: Into<Vec<TexturePixelStorageParam>>>(
    //     data: D,
    //     data_type: TextureDataType,
    //     width: i32,
    //     height: i32,
    //     internal_format: TextureFormat,
    //     format: TextureFormat,
    //     level: i32,
    //     src_offset: u32,
    //     generate_mipmap: bool,
    //     params: P,
    // ) -> Self {
    //     Self {
    //         status: RefCell::new(TextureStatus::UpdateTexture {
    //             id: None,
    //             data: TextureData::FromBinary {
    //                 width,
    //                 height,
    //                 data: Box::new(data),
    //                 format,
    //                 src_offset,
    //                 level,
    //             },
    //         }),
    //         internal_format,
    //         data_type,
    //         params: params.into(),
    //         generate_mipmap,
    //     }
    // }

    pub fn with_html_image_element(
        target: TextureTarget,
        image: HtmlImageElement,
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
                list: vec![TextureData::FromHtmlImageElement {
                    target,
                    image: Box::new(image),
                    format,
                    level,
                }],
            }),
            internal_format,
            data_type,
            pixel_storages,
            generate_mipmap,
        }
    }

    // pub fn with_canvas<
    //     C: AsRef<HtmlCanvasElement> + 'static,
    //     P: Into<Vec<TexturePixelStorageParam>>,
    // >(
    //     canvas: C,
    //     data_type: TextureDataType,
    //     internal_format: TextureFormat,
    //     format: TextureFormat,
    //     level: i32,
    //     generate_mipmap: bool,
    //     params: P,
    // ) -> Self {
    //     Self {
    //         status: RefCell::new(TextureStatus::UpdateTexture {
    //             id: None,
    //             data: TextureData::FromCanvas {
    //                 canvas: Box::new(canvas),
    //                 format,
    //                 level,
    //             },
    //         }),
    //         internal_format,
    //         data_type,
    //         params: params.into(),
    //         generate_mipmap,
    //     }
    // }

    // pub fn with_canvas_size<
    //     C: AsRef<HtmlCanvasElement> + 'static,
    //     P: Into<Vec<TexturePixelStorageParam>>,
    // >(
    //     canvas: C,
    //     width: i32,
    //     height: i32,
    //     data_type: TextureDataType,
    //     internal_format: TextureFormat,
    //     format: TextureFormat,
    //     level: i32,
    //     generate_mipmap: bool,
    //     params: P,
    // ) -> Self {
    //     Self {
    //         status: RefCell::new(TextureStatus::UpdateTexture {
    //             id: None,
    //             data: TextureData::FromCanvasWithSize {
    //                 canvas: Box::new(canvas),
    //                 width,
    //                 height,
    //                 format,
    //                 level,
    //             },
    //         }),
    //         internal_format,
    //         data_type,
    //         params: params.into(),
    //         generate_mipmap,
    //     }
    // }

    // pub fn with_js_object<O: AsRef<Object> + 'static, P: Into<Vec<TexturePixelStorageParam>>>(
    //     object: O,
    //     width: i32,
    //     height: i32,
    //     data_type: TextureDataType,
    //     internal_format: TextureFormat,
    //     format: TextureFormat,
    //     level: i32,
    //     src_offset: u32,
    //     generate_mipmap: bool,
    //     params: P,
    // ) -> Self {
    //     Self {
    //         status: RefCell::new(TextureStatus::UpdateTexture {
    //             id: None,
    //             data: TextureData::FromObject {
    //                 canvas: Box::new(object),
    //                 width,
    //                 height,
    //                 format,
    //                 src_offset,
    //                 level,
    //             },
    //         }),
    //         internal_format,
    //         data_type,
    //         params: params.into(),
    //         generate_mipmap,
    //     }
    // }

    // pub fn buffer_texture(&mut self) {
    //     let status = self.status.borrow_mut();
    //     match &*status {
    //         TextureStatus::Unchanged { id } => todo!(),
    //         TextureStatus::UpdateTexture { id, data } => todo!(),
    //         TextureStatus::UpdateSubTexture { id, data } => todo!(),
    //     }
    // }
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
            internal_format,
            data_type,
            pixel_storages,
            generate_mipmap,
        }: &TextureDescriptor,
    ) -> Result<&WebGlTexture, String> {
        let mut status = status.borrow_mut();
        match &*status {
            TextureStatus::Unchanged { id } => match self.store.get(id) {
                Some(texture) => Ok(texture),
                None => Err(format!("failed to get texture with id {}", id)),
            },
            TextureStatus::UpdateTexture { id, list } => {
                // delete old texture
                if let Some(texture) = id.as_ref().and_then(|id| self.store.remove(id)) {
                    self.gl.delete_texture(Some(&texture));
                }

                // create texture
                let Some(texture) = self.gl.create_texture() else {
                    return Err(String::from("failed to create texture"));
                };

                // set pixel storage params
                pixel_storages.iter().for_each(|param| {
                    self.gl.pixel_storei(param.key(), param.value());
                });

                // active texture
                self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
                // buffer every image
                for data in list {

                    let result = match data {
                        TextureData::Preallocate { target, width, height, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                *width,
                                *height,
                                0,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                None)
                        }
                        TextureData::FromBinary {
                            target,
                            width,
                            height,
                            data,
                            src_offset,
                            format,
                            level
                        } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                *width,
                                *height,
                                0,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                data.as_ref().as_ref(),
                                *src_offset
                            )
                        }
                        TextureData::FromHtmlCanvasElement { target, canvas, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                canvas.as_ref().as_ref()
                            )
                        },
                        TextureData::FromHtmlCanvasElementWithSize { target, width, height, canvas, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_canvas_element(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                *width,
                                *height,
                                0,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                canvas.as_ref().as_ref()
                            )
                        },
                        TextureData::FromHtmlImageElement { target, image, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                image.as_ref().as_ref()
                            )
                        },
                        TextureData::FromHtmlImageElementWithSize { target, width, height, image, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_image_element(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                *width,
                                *height,
                                0,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                image.as_ref().as_ref()
                            )
                        },
                        TextureData::FromObject { target, width, height, object, src_offset, format, level } => {
                            self.gl.bind_texture(target.to_gl_enum(), Some(&texture));
                            self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                                target.to_gl_enum(),
                                *level,
                                internal_format.to_gl_enum() as i32,
                                *width,
                                *height,
                                0,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                object.as_ref().as_ref(),
                                *src_offset
                            )
                        },
                    };
                    if let Err(err) = result {
                        // should log error
                        console_log!("{:?}", err);
                        return Err(err
                            .as_string()
                            .unwrap_or(String::from("unknown error during tex image 2d")));
                    }
                }

                // generates mipmaps
                if *generate_mipmap {
                    match target {
                        TextureTarget::Texture2D => {
                            self.gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D)
                        }
                        _ => self
                            .gl
                            .generate_mipmap(WebGl2RenderingContext::TEXTURE_CUBE_MAP),
                    }
                }

                // unbinds for good practice
                self.gl.bind_texture(target.to_gl_enum(), None);

                // stores it
                let id = Uuid::new_v4();
                let texture = self.store.entry(id.clone()).or_insert(texture);

                // updates status
                *status = TextureStatus::Unchanged { id };

                Ok(texture)
            }
            TextureStatus::UpdateSubTexture { id, list } => {
                let Some(texture) = self.store.get(id) else {
                    return Err(format!("failed to get texture with id {}", id));
                };

                // buffer sub image
                self.gl.bind_texture(target.to_gl_enum(), Some(texture));
                for data in list {
                    let result = match data {
                        TextureSubData::Clear { target, width, height, format, x_offset, y_offset, level } => {
                            self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                                target.to_gl_enum(),
                                *level,
                                *x_offset,
                                *y_offset,
                                *width,
                                *height,
                                format.to_gl_enum(),
                                data_type.to_gl_enum(),
                                None
                            )
                        }
                        TextureSubData::FromBinary { target, x_offset, y_offset, width, height, data, src_offset, format, level } => self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                            target.to_gl_enum(),
                            *level,
                            *x_offset,
                            *y_offset,
                            *width,
                            *height,
                            format.to_gl_enum(),
                            data_type.to_gl_enum(),
                            data.as_ref().as_ref(),
                            *src_offset
                        ),
                        TextureSubData::FromHtmlCanvasElement { target, x_offset, y_offset, canvas, format, level } => self.gl.tex_sub_image_2d_with_u32_and_u32_and_html_canvas_element(
                            target.to_gl_enum(),
                            *level,
                            *x_offset,
                            *y_offset,
                            format.to_gl_enum(),
                            data_type.to_gl_enum(),
                            canvas.as_ref().as_ref()
                        ),
                        TextureSubData::FromHtmlCanvasElementWithSize { target, x_offset, y_offset, width, height, canvas, format, level } => self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                            target.to_gl_enum(),
                            *level,
                            *x_offset,
                            *y_offset,
                            *width,
                            *height,
                            format.to_gl_enum(),
                            data_type.to_gl_enum(),
                            canvas.as_ref().as_ref()
                        ),
                        TextureSubData::FromObject { target, x_offset, y_offset, width, height, canvas, src_offset, format, level } => self.gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                            target.to_gl_enum(),
                            *level,
                            *x_offset,
                            *y_offset,
                            *width,
                            *height,
                            format.to_gl_enum(),
                            data_type.to_gl_enum(),
                            canvas.as_ref().as_ref(),
                            *src_offset
                        ),
                    };
                    if let Err(err) = result {
                        // should log error
                        console_log!("{:?}", err);
                        return Err(err
                            .as_string()
                            .unwrap_or(String::from("unknown error during tex sub image 2d")));
                    }
                }

                // generates mipmaps
                if *generate_mipmap {
                    match target {
                        TextureTarget::Texture2D => {
                            self.gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D)
                        }
                        _ => self
                            .gl
                            .generate_mipmap(WebGl2RenderingContext::TEXTURE_CUBE_MAP),
                    }
                }

                // unbinds for good practice
                self.gl.bind_texture(target.to_gl_enum(), None);

                Ok(texture)
            }
        }
    }
}
