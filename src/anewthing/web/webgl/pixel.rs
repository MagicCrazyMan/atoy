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

/// Available texture pack pixel store for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelPackStore {
    PackAlignment,
    PackRowLength,
    PackSkipPixels,
    PackSkipRows,
}

/// Available texture unpack pixel stores with value for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelPackStoreWithValue {
    PackAlignment(i32),
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

/// Available texture unpack color space conversions for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelUnpackColorSpaceConversion {
    None,
    #[gl_enum(BROWSER_DEFAULT_WEBGL)]
    BrowserDefault,
}

/// Available texture unpack pixel store for [`WebGl2RenderingContext`].
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

/// Available texture unpack pixel stores with value for [`WebGl2RenderingContext`].
///
/// [`WebGl2RenderingContext::UNPACK_ALIGNMENT`] and [`WebGl2RenderingContext::UNPACK_ROW_LENGTH`] are ignored in WebGL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlPixelUnpackStoreWithValue {
    // UnpackAlignment(i32),
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
