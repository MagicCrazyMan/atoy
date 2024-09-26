use proc::GlEnum;
use web_sys::WebGl2RenderingContext;

/// Available image pixel formats mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelFormat {
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

impl WebGlPixelFormat {
    /// Returns the number of channels.
    ///
    /// [`DepthComponent`](WebGlPixelFormat::DepthComponent) and [`DepthStencil`](WebGlPixelFormat::DepthStencil) returns `1`.
    pub fn channels(&self) -> usize {
        match self {
            WebGlPixelFormat::Red => 1,
            WebGlPixelFormat::RedInteger => 1,
            WebGlPixelFormat::Rg => 2,
            WebGlPixelFormat::RgInteger => 2,
            WebGlPixelFormat::Rgb => 3,
            WebGlPixelFormat::RgbInteger => 3,
            WebGlPixelFormat::Rgba => 4,
            WebGlPixelFormat::RgbaInteger => 4,
            WebGlPixelFormat::Luminance => 1,
            WebGlPixelFormat::LuminanceAlpha => 2,
            WebGlPixelFormat::Alpha => 1,
            WebGlPixelFormat::DepthComponent => 1,
            WebGlPixelFormat::DepthStencil => 2,
        }
    }
}

/// Available image pixel data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelDataType {
    Float,
    HalfFloat,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    #[gl_enum(UNSIGNED_SHORT_5_6_5)]
    UnsignedShort_5_6_5,
    #[gl_enum(UNSIGNED_SHORT_4_4_4_4)]
    UnsignedShort_4_4_4_4,
    #[gl_enum(UNSIGNED_SHORT_5_5_5_1)]
    UnsignedShort_5_5_5_1,
    #[gl_enum(UNSIGNED_INT_2_10_10_10_REV)]
    UnsignedInt_2_10_10_10Rev,
    #[gl_enum(UNSIGNED_INT_10F_11F_11F_REV)]
    UnsignedInt_10F_11F_11F_Rev,
    #[gl_enum(UNSIGNED_INT_5_9_9_9_REV)]
    UnsignedInt_5_9_9_9Rev,
    #[gl_enum(UNSIGNED_INT_24_8)]
    UnsignedInt_24_8,
    #[gl_enum(FLOAT_32_UNSIGNED_INT_24_8_REV)]
    Float_32_UnsignedInt_24_8_Rev,
}

/// Calculates bytes per pixel by pixel format and pixel data type.
///
/// Incompatible format and data type combination does not returns `Err`,
/// checks the source code for details.
pub fn bytes_per_pixel(format: WebGlPixelFormat, data_type: WebGlPixelDataType) -> usize {
    let channels = format.channels();
    match data_type {
        WebGlPixelDataType::Float => channels * 4,
        WebGlPixelDataType::HalfFloat => channels * 2,
        WebGlPixelDataType::Byte => channels * 1,
        WebGlPixelDataType::Short => channels * 2,
        WebGlPixelDataType::Int => channels * 4,
        WebGlPixelDataType::UnsignedByte => channels * 1,
        WebGlPixelDataType::UnsignedShort => channels * 2,
        WebGlPixelDataType::UnsignedInt => channels * 4,
        WebGlPixelDataType::UnsignedShort_5_6_5 => 2,
        WebGlPixelDataType::UnsignedShort_4_4_4_4 => 2,
        WebGlPixelDataType::UnsignedShort_5_5_5_1 => 2,
        WebGlPixelDataType::UnsignedInt_2_10_10_10Rev => 4,
        WebGlPixelDataType::UnsignedInt_10F_11F_11F_Rev => 4,
        WebGlPixelDataType::UnsignedInt_5_9_9_9Rev => 4,
        WebGlPixelDataType::UnsignedInt_24_8 => 4,
        WebGlPixelDataType::Float_32_UnsignedInt_24_8_Rev => 8,
    }
}

pub fn size_of(
    format: WebGlPixelFormat,
    data_type: WebGlPixelDataType,
    stores: &[WebGlPixelPackStoreWithValue],
    width: usize,
    height: usize,
) -> usize {
     let bytes_per_pixel = bytes_per_pixel(format, data_type);
    //  let width = 
    todo!()
}

/// Available pixel alignment for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum WebGlPixelAlignment {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

/// Available pack pixel store for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelPackStore {
    PackAlignment,
    PackRowLength,
    PackSkipPixels,
    PackSkipRows,
}

/// Available unpack pixel stores with value for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelPackStoreWithValue {
    PackAlignment(WebGlPixelAlignment),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
}

impl From<WebGlPixelPackStoreWithValue> for WebGlPixelPackStore {
    #[inline]
    fn from(value: WebGlPixelPackStoreWithValue) -> Self {
        match value {
            WebGlPixelPackStoreWithValue::PackAlignment(_) => WebGlPixelPackStore::PackAlignment,
            WebGlPixelPackStoreWithValue::PackRowLength(_) => WebGlPixelPackStore::PackRowLength,
            WebGlPixelPackStoreWithValue::PackSkipPixels(_) => WebGlPixelPackStore::PackSkipPixels,
            WebGlPixelPackStoreWithValue::PackSkipRows(_) => WebGlPixelPackStore::PackSkipRows,
        }
    }
}

impl WebGlPixelPackStoreWithValue {
    /// Returns as [`WebGlPixelPackStore`].
    #[inline]
    pub fn as_pack_pixel_store(&self) -> WebGlPixelPackStore {
        WebGlPixelPackStore::from(*self)
    }

    #[inline]
    pub fn to_gl_enum(&self) -> u32 {
        WebGlPixelPackStore::from(*self).to_gl_enum()
    }
}

/// Available unpack color space conversions for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelUnpackColorSpaceConversion {
    None,
    #[gl_enum(BROWSER_DEFAULT_WEBGL)]
    BrowserDefault,
}

/// Available unpack pixel store for [`WebGl2RenderingContext`].
///
/// [`WebGl2RenderingContext::UNPACK_ALIGNMENT`] and [`WebGl2RenderingContext::UNPACK_ROW_LENGTH`] are ignored in WebGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelUnpackStore {
    // UnpackAlignment,
    #[gl_enum(UNPACK_FLIP_Y_WEBGL)]
    UnpackFlipY,
    #[gl_enum(UNPACK_PREMULTIPLY_ALPHA_WEBGL)]
    UnpackPremultiplyAlpha,
    #[gl_enum(UNPACK_COLORSPACE_CONVERSION_WEBGL)]
    UnpackColorSpaceConversion,
    // UnpackRowLength,
    UnpackImageHeight,
    UnpackSkipPixels,
    UnpackSkipRows,
    UnpackSkipImages,
}

/// Available unpack pixel stores with value for [`WebGl2RenderingContext`].
///
/// [`WebGl2RenderingContext::UNPACK_ALIGNMENT`] and [`WebGl2RenderingContext::UNPACK_ROW_LENGTH`] are ignored in WebGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelUnpackStoreWithValue {
    // UnpackAlignment(WebGlPixelAlignment),
    UnpackFlipY(bool),
    UnpackPremultiplyAlpha(bool),
    UnpackColorSpaceConversion(WebGlPixelUnpackColorSpaceConversion),
    // UnpackRowLength(i32),
    UnpackImageHeight(i32),
    UnpackSkipPixels(i32),
    UnpackSkipRows(i32),
    UnpackSkipImages(i32),
}

impl From<WebGlPixelUnpackStoreWithValue> for WebGlPixelUnpackStore {
    #[inline]
    fn from(value: WebGlPixelUnpackStoreWithValue) -> Self {
        match value {
            // WebGlPixelUnpackStoreWithValue::UnpackAlignment(_) => {
            //     WebGlPixelUnpackStore::UnpackAlignment
            // }
            WebGlPixelUnpackStoreWithValue::UnpackFlipY(_) => WebGlPixelUnpackStore::UnpackFlipY,
            WebGlPixelUnpackStoreWithValue::UnpackPremultiplyAlpha(_) => {
                WebGlPixelUnpackStore::UnpackPremultiplyAlpha
            }
            WebGlPixelUnpackStoreWithValue::UnpackColorSpaceConversion(_) => {
                WebGlPixelUnpackStore::UnpackColorSpaceConversion
            }
            // WebGlPixelUnpackStoreWithValue::UnpackRowLength(_) => {
            //     WebGlPixelUnpackStore::UnpackRowLength
            // }
            WebGlPixelUnpackStoreWithValue::UnpackImageHeight(_) => {
                WebGlPixelUnpackStore::UnpackImageHeight
            }
            WebGlPixelUnpackStoreWithValue::UnpackSkipPixels(_) => {
                WebGlPixelUnpackStore::UnpackSkipPixels
            }
            WebGlPixelUnpackStoreWithValue::UnpackSkipRows(_) => {
                WebGlPixelUnpackStore::UnpackSkipRows
            }
            WebGlPixelUnpackStoreWithValue::UnpackSkipImages(_) => {
                WebGlPixelUnpackStore::UnpackSkipImages
            }
        }
    }
}

impl WebGlPixelUnpackStoreWithValue {
    /// Returns as [`WebGlPixelUnpackStore`].
    #[inline]
    pub fn as_pixel_store(&self) -> WebGlPixelUnpackStore {
        WebGlPixelUnpackStore::from(*self)
    }

    #[inline]
    pub fn to_gl_enum(&self) -> u32 {
        WebGlPixelUnpackStore::from(*self).to_gl_enum()
    }

    /// Returns default value of a specified [`WebGlPixelUnpackStore`].
    pub fn default_of(store: WebGlPixelUnpackStore) -> WebGlPixelUnpackStoreWithValue {
        match store {
            WebGlPixelUnpackStore::UnpackFlipY => {
                WebGlPixelUnpackStoreWithValue::UnpackFlipY(false)
            }
            WebGlPixelUnpackStore::UnpackPremultiplyAlpha => {
                WebGlPixelUnpackStoreWithValue::UnpackPremultiplyAlpha(false)
            }
            WebGlPixelUnpackStore::UnpackColorSpaceConversion => {
                WebGlPixelUnpackStoreWithValue::UnpackColorSpaceConversion(
                    WebGlPixelUnpackColorSpaceConversion::BrowserDefault,
                )
            }
            WebGlPixelUnpackStore::UnpackImageHeight => {
                WebGlPixelUnpackStoreWithValue::UnpackImageHeight(0)
            }
            WebGlPixelUnpackStore::UnpackSkipPixels => {
                WebGlPixelUnpackStoreWithValue::UnpackSkipPixels(0)
            }
            WebGlPixelUnpackStore::UnpackSkipRows => {
                WebGlPixelUnpackStoreWithValue::UnpackSkipRows(0)
            }
            WebGlPixelUnpackStore::UnpackSkipImages => {
                WebGlPixelUnpackStoreWithValue::UnpackSkipImages(0)
            }
        }
    }

    pub(crate) fn set_pixel_store(&self, gl: &WebGl2RenderingContext) {
        let pname = self.to_gl_enum();
        match self {
            WebGlPixelUnpackStoreWithValue::UnpackFlipY(v) => {
                gl.pixel_storei(pname, if *v { 1 } else { 0 })
            }
            WebGlPixelUnpackStoreWithValue::UnpackPremultiplyAlpha(v) => {
                gl.pixel_storei(pname, if *v { 1 } else { 0 })
            }
            WebGlPixelUnpackStoreWithValue::UnpackColorSpaceConversion(v) => {
                gl.pixel_storei(pname, v.to_gl_enum() as i32)
            }
            WebGlPixelUnpackStoreWithValue::UnpackImageHeight(v) => gl.pixel_storei(pname, *v),
            WebGlPixelUnpackStoreWithValue::UnpackSkipPixels(v) => gl.pixel_storei(pname, *v),
            WebGlPixelUnpackStoreWithValue::UnpackSkipRows(v) => gl.pixel_storei(pname, *v),
            WebGlPixelUnpackStoreWithValue::UnpackSkipImages(v) => gl.pixel_storei(pname, *v),
        }
    }
}
