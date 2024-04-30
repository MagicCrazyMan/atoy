use web_sys::WebGl2RenderingContext;

/// Available pixel data types mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelDataType {
    Float,
    HalfFloat,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    UnsignedShort5_6_5,
    UnsignedShort4_4_4_4,
    UnsignedShort5_5_5_1,
    UnsignedInt2_10_10_10Rev,
    #[allow(non_camel_case_types)]
    UnsignedInt10F_11F_11F_Rev,
    UnsignedInt5_9_9_9Rev,
    UnsignedInt24_8,
    Float32UnsignedInt24_8Rev,
}

/// Available unpack color space conversions for [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnpackColorSpaceConversion {
    None,
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
