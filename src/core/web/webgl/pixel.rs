use proc::GlEnum;
use web_sys::WebGl2RenderingContext;

/// Available pixel formats mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum PixelFormat {
    #[gl_enum(RED)]
    Red,
    #[gl_enum(RED_INTEGER)]
    RedInteger,
    #[gl_enum(RG)]
    Rg,
    #[gl_enum(RG_INTEGER)]
    RgInteger,
    #[gl_enum(RGB)]
    Rgb,
    #[gl_enum(RGB_INTEGER)]
    RgbInteger,
    #[gl_enum(RGBA)]
    Rgba,
    #[gl_enum(RGBA_INTEGER)]
    RgbaInteger,
    #[gl_enum(LUMINANCE)]
    Luminance,
    #[gl_enum(LUMINANCE_ALPHA)]
    LuminanceAlpha,
    #[gl_enum(ALPHA)]
    Alpha,
    #[gl_enum(DEPTH_COMPONENT)]
    DepthComponent,
    #[gl_enum(DEPTH_STENCIL)]
    DepthStencil,
}

impl PixelFormat {
    pub(crate) fn channels_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Red => 1,
            PixelFormat::RedInteger => 1,
            PixelFormat::Rg => 2,
            PixelFormat::RgInteger => 2,
            PixelFormat::Rgb => 3,
            PixelFormat::RgbInteger => 3,
            PixelFormat::Rgba => 4,
            PixelFormat::RgbaInteger => 4,
            PixelFormat::Luminance => 1,
            PixelFormat::LuminanceAlpha => 2,
            PixelFormat::Alpha => 1,
            PixelFormat::DepthComponent => 1,
            PixelFormat::DepthStencil => 1,
        }
    }
}

/// Available pixel data types mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum PixelDataType {
    #[gl_enum(FLOAT)]
    Float,
    #[gl_enum(HALF_FLOAT)]
    HalfFloat,
    #[gl_enum(BYTE)]
    Byte,
    #[gl_enum(SHORT)]
    Short,
    #[gl_enum(INT)]
    Int,
    #[gl_enum(UNSIGNED_BYTE)]
    UnsignedByte,
    #[gl_enum(UNSIGNED_SHORT)]
    UnsignedShort,
    #[gl_enum(UNSIGNED_INT)]
    UnsignedInt,
    #[gl_enum(UNSIGNED_SHORT_5_6_5)]
    UnsignedShort5_6_5,
    #[gl_enum(UNSIGNED_SHORT_4_4_4_4)]
    UnsignedShort4_4_4_4,
    #[gl_enum(UNSIGNED_SHORT_5_5_5_1)]
    UnsignedShort5_5_5_1,
    #[gl_enum(UNSIGNED_INT_2_10_10_10_REV)]
    UnsignedInt2_10_10_10Rev,
    #[gl_enum(UNSIGNED_INT_10F_11F_11F_REV)]
    #[allow(non_camel_case_types)]
    UnsignedInt10F_11F_11F_Rev,
    #[gl_enum(UNSIGNED_INT_5_9_9_9_REV)]
    UnsignedInt5_9_9_9Rev,
    #[gl_enum(UNSIGNED_INT_24_8)]
    UnsignedInt24_8,
    #[gl_enum(FLOAT_32_UNSIGNED_INT_24_8_REV)]
    Float32UnsignedInt24_8Rev,
}

impl PixelDataType {
    pub(crate) fn bytes_per_channel(&self) -> usize {
        match self {
            PixelDataType::Float => 4,
            PixelDataType::HalfFloat => 2,
            PixelDataType::Byte => 1,
            PixelDataType::Short => 2,
            PixelDataType::Int => 4,
            PixelDataType::UnsignedByte => 1,
            PixelDataType::UnsignedShort => 2,
            PixelDataType::UnsignedInt => 4,
            PixelDataType::UnsignedShort5_6_5 => 2,
            PixelDataType::UnsignedShort4_4_4_4 => 2,
            PixelDataType::UnsignedShort5_5_5_1 => 2,
            PixelDataType::UnsignedInt2_10_10_10Rev => 4,
            PixelDataType::UnsignedInt10F_11F_11F_Rev => 4,
            PixelDataType::UnsignedInt5_9_9_9Rev => 4,
            PixelDataType::UnsignedInt24_8 => 4,
            PixelDataType::Float32UnsignedInt24_8Rev => 4,
        }
    }
}

/// Available unpack color space conversions for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum UnpackColorSpaceConversion {
    #[gl_enum(NONE)]
    None,
    #[gl_enum(BROWSER_DEFAULT_WEBGL)]
    BrowserDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PixelAlignment {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

/// Available pixel pack storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelPackStorage {
    PackAlignment(PixelAlignment),
    PackRowLength(i32),
    PackSkipPixels(i32),
    PackSkipRows(i32),
}

impl PixelPackStorage {
    pub(super) fn pixel_store(&self, gl: &WebGl2RenderingContext) -> PixelPackStorage {
        match self {
            PixelPackStorage::PackAlignment(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ALIGNMENT, *v as i32);
                PixelPackStorage::PackAlignment(PixelAlignment::Four)
            }
            PixelPackStorage::PackRowLength(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_ROW_LENGTH, *v);
                PixelPackStorage::PackRowLength(0)
            }
            PixelPackStorage::PackSkipPixels(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_PIXELS, *v);
                PixelPackStorage::PackSkipPixels(0)
            }
            PixelPackStorage::PackSkipRows(v) => {
                gl.pixel_storei(WebGl2RenderingContext::PACK_SKIP_ROWS, *v);
                PixelPackStorage::PackSkipRows(0)
            }
        }
    }
}

/// Available pixel unpack storages for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelUnpackStorage {
    UnpackAlignment(PixelAlignment),
    UnpackFlipY(bool),
    UnpackPremultiplyAlpha(bool),
    UnpackColorSpaceConversion(UnpackColorSpaceConversion),
    UnpackRowLength(i32),
    UnpackImageHeight(i32),
    UnpackSkipPixels(i32),
    UnpackSkipRows(i32),
    UnpackSkipImages(i32),
}

impl PixelUnpackStorage {
    pub(super) fn pixel_store(&self, gl: &WebGl2RenderingContext) -> PixelUnpackStorage {
        match self {
            PixelUnpackStorage::UnpackAlignment(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ALIGNMENT, *v as i32);
                PixelUnpackStorage::UnpackAlignment(PixelAlignment::Four)
            }
            PixelUnpackStorage::UnpackFlipY(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL,
                    if *v { 1 } else { 0 },
                );
                PixelUnpackStorage::UnpackFlipY(false)
            }
            PixelUnpackStorage::UnpackPremultiplyAlpha(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
                    if *v { 1 } else { 0 },
                );
                PixelUnpackStorage::UnpackPremultiplyAlpha(false)
            }
            PixelUnpackStorage::UnpackColorSpaceConversion(v) => {
                gl.pixel_storei(
                    WebGl2RenderingContext::UNPACK_COLORSPACE_CONVERSION_WEBGL,
                    match v {
                        UnpackColorSpaceConversion::None => WebGl2RenderingContext::NONE as i32,
                        UnpackColorSpaceConversion::BrowserDefault => {
                            WebGl2RenderingContext::BROWSER_DEFAULT_WEBGL as i32
                        }
                    },
                );
                PixelUnpackStorage::UnpackColorSpaceConversion(
                    UnpackColorSpaceConversion::BrowserDefault,
                )
            }
            PixelUnpackStorage::UnpackRowLength(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_ROW_LENGTH, *v);
                PixelUnpackStorage::UnpackRowLength(0)
            }
            PixelUnpackStorage::UnpackImageHeight(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_IMAGE_HEIGHT, *v);
                PixelUnpackStorage::UnpackImageHeight(0)
            }
            PixelUnpackStorage::UnpackSkipPixels(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_PIXELS, *v);
                PixelUnpackStorage::UnpackSkipPixels(0)
            }
            PixelUnpackStorage::UnpackSkipRows(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_ROWS, *v);
                PixelUnpackStorage::UnpackSkipRows(0)
            }
            PixelUnpackStorage::UnpackSkipImages(v) => {
                gl.pixel_storei(WebGl2RenderingContext::UNPACK_SKIP_IMAGES, *v);
                PixelUnpackStorage::UnpackSkipImages(0)
            }
        }
    }
}
