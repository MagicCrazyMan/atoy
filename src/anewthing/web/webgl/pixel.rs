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
pub fn bytes_per_pixel(
    pixel_format: WebGlPixelFormat,
    pixel_data_type: WebGlPixelDataType,
) -> usize {
    let channels = pixel_format.channels();
    match pixel_data_type {
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

/// Calculates bytes length of a packed
pub fn bytes_length_of(
    pixel_format: WebGlPixelFormat,
    pixel_data_type: WebGlPixelDataType,
    pixel_pack_stores: WebGlPixelPackStores,
    width: usize,
    height: usize,
) -> usize {
    let rows = if pixel_pack_stores.row_length == 0 {
        pixel_pack_stores.skip_pixels + width
    } else {
        pixel_pack_stores.row_length
    };
    let height = pixel_pack_stores.skip_rows + height;

    // s
    let bytes_per_pixel = bytes_per_pixel(pixel_format, pixel_data_type);
    // nl
    let bytes_per_row = match pixel_pack_stores.alignment {
        WebGlPixelAlignment::One => bytes_per_pixel * rows,
        _ => {
            let nl = (bytes_per_pixel * rows) as f64;
            let a = match pixel_pack_stores.alignment {
                WebGlPixelAlignment::Two => 2.0,
                WebGlPixelAlignment::Four => 4.0,
                WebGlPixelAlignment::Eight => 8.0,
                _ => unreachable!(),
            };
            let s = bytes_per_pixel as f64;
            ((a / s) * ((s / a) * nl).ceil()) as usize
        }
    };

    rows * bytes_per_row * height
}

/// Available pixel alignment for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum WebGlPixelAlignment {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

/// Available pack pixel store for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlPixelPackStore {
    PackAlignment,
    PackRowLength,
    PackSkipPixels,
    PackSkipRows,
}

/// A collection of pixel pack store parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebGlPixelPackStores {
    pub alignment: WebGlPixelAlignment,
    pub row_length: usize,
    pub skip_pixels: usize,
    pub skip_rows: usize,
}

impl Default for WebGlPixelPackStores {
    fn default() -> Self {
        Self {
            alignment: WebGlPixelAlignment::Eight,
            row_length: 0,
            skip_pixels: 0,
            skip_rows: 0,
        }
    }
}

impl WebGlPixelPackStores {
    pub(crate) fn set_pixel_store(&self, gl: &WebGl2RenderingContext) {
        gl.pixel_storei(
            WebGl2RenderingContext::PACK_ALIGNMENT,
            self.alignment as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::PACK_ROW_LENGTH,
            self.row_length as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::PACK_SKIP_PIXELS,
            self.skip_pixels as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::PACK_SKIP_ROWS,
            self.skip_rows as i32,
        );
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

/// A collection of pixel unpack store parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebGlPixelUnpackStores {
    pub flip_y: bool,
    pub premultiply_alpha: bool,
    pub color_space_conversion: WebGlPixelUnpackColorSpaceConversion,
    pub image_height: usize,
    pub skip_pixels: usize,
    pub skip_rows: usize,
    pub skip_images: usize,
}

impl Default for WebGlPixelUnpackStores {
    fn default() -> Self {
        Self {
            flip_y: false,
            premultiply_alpha: false,
            color_space_conversion: WebGlPixelUnpackColorSpaceConversion::BrowserDefault,
            image_height: 0,
            skip_pixels: 0,
            skip_rows: 0,
            skip_images: 0,
        }
    }
}

impl WebGlPixelUnpackStores {
    pub(crate) fn set_pixel_store(&self, gl: &WebGl2RenderingContext) {
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
            if self.flip_y { 1 } else { 0 },
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
            if self.premultiply_alpha { 1 } else { 0 },
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
            self.color_space_conversion.to_gl_enum() as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT,
            self.image_height as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_SKIP_PIXELS,
            self.skip_pixels as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_SKIP_ROWS,
            self.skip_rows as i32,
        );
        gl.pixel_storei(
            WebGl2RenderingContext::UNPACK_SKIP_IMAGES,
            self.skip_images as i32,
        );
    }
}
