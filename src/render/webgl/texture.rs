use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
    fmt::Debug,
    rc::{Rc, Weak},
};

use hashbrown::{HashMap, HashSet};
use log::debug;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{
    js_sys::{Float32Array, Uint16Array, Uint32Array, Uint8Array},
    HtmlCanvasElement, HtmlImageElement, HtmlVideoElement, ImageBitmap, ImageData,
    WebGl2RenderingContext, WebGlTexture,
};

use crate::lru::{Lru, LruNode};

use super::{conversion::ToGlEnum, error::Error};

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureTarget {
    TEXTURE_2D,
    TEXTURE_CUBE_MAP_POSITIVE_X,
    TEXTURE_CUBE_MAP_POSITIVE_Y,
    TEXTURE_CUBE_MAP_POSITIVE_Z,
    TEXTURE_CUBE_MAP_NEGATIVE_X,
    TEXTURE_CUBE_MAP_NEGATIVE_Y,
    TEXTURE_CUBE_MAP_NEGATIVE_Z,
    TEXTURE_2D_ARRAY,
    TEXTURE_3D,
}

/// Available texture formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    RED,
    RED_INTEGER,
    RG,
    RG_INTEGER,
    RGB,
    RGB_INTEGER,
    RGBA,
    RGBA_INTEGER,
    LUMINANCE,
    LUMINANCE_ALPHA,
    ALPHA,
    DEPTH_COMPONENT,
    DEPTH_STENCIL,
}

/// Available texture internal formats mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureInternalFormat {
    RGBA32I,
    RGBA32UI,
    RGBA16I,
    RGBA16UI,
    RGBA8,
    RGBA8I,
    RGBA8UI,
    SRGB8_ALPHA8,
    RGB10_A2,
    RGB10_A2UI,
    RGBA4,
    RGB5_A1,
    RGB8,
    RGB565,
    RG32I,
    RG32UI,
    RG16I,
    RG16UI,
    RG8,
    RG8I,
    RG8UI,
    R32I,
    R32UI,
    R16I,
    R16UI,
    R8,
    R8I,
    R8UI,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RGBA32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RGBA16F,
    RGBA8_SNORM,
    RGB32F,
    RGB32I,
    RGB32UI,
    RGB16F,
    RGB16I,
    RGB16UI,
    RGB8_SNORM,
    RGB8I,
    RGB8UI,
    SRGB8,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R11F_G11F_B10F,
    RGB9_E5,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RG32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    RG16F,
    RG8_SNORM,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled.
    R16F,
    R8_SNORM,
    DEPTH_COMPONENT32F,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT16,
    DEPTH32F_STENCIL8,
    DEPTH24_STENCIL8,
}

impl TextureInternalFormat {
    /// Estimates texture memory usage in bytes in WebGL runtime by texture size and whether mipmap enabled.
    pub fn estimate_memory_size(&self, width: usize, height: usize, mipmap: bool) -> usize {
        todo!()
    }
}

/// Available texture data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDataType {
    FLOAT,
    HALF_FLOAT,
    BYTE,
    SHORT,
    INT,
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
    UNSIGNED_SHORT_5_6_5,
    UNSIGNED_SHORT_4_4_4_4,
    UNSIGNED_SHORT_5_5_5_1,
    UNSIGNED_INT_2_10_10_10_REV,
    UNSIGNED_INT_10F_11F_11F_REV,
    UNSIGNED_INT_5_9_9_9_REV,
    UNSIGNED_INT_24_8,
    FLOAT_32_UNSIGNED_INT_24_8_REV,
}

/// Available texture units mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnit {
    TEXTURE0,
    TEXTURE1,
    TEXTURE2,
    TEXTURE3,
    TEXTURE4,
    TEXTURE5,
    TEXTURE6,
    TEXTURE7,
    TEXTURE8,
    TEXTURE9,
    TEXTURE10,
    TEXTURE11,
    TEXTURE12,
    TEXTURE13,
    TEXTURE14,
    TEXTURE15,
    TEXTURE16,
    TEXTURE17,
    TEXTURE18,
    TEXTURE19,
    TEXTURE20,
    TEXTURE21,
    TEXTURE22,
    TEXTURE23,
    TEXTURE24,
    TEXTURE25,
    TEXTURE26,
    TEXTURE27,
    TEXTURE28,
    TEXTURE29,
    TEXTURE30,
    TEXTURE31,
}

impl TextureUnit {
    pub fn unit_index(&self) -> i32 {
        match self {
            TextureUnit::TEXTURE0 => 0,
            TextureUnit::TEXTURE1 => 1,
            TextureUnit::TEXTURE2 => 2,
            TextureUnit::TEXTURE3 => 3,
            TextureUnit::TEXTURE4 => 4,
            TextureUnit::TEXTURE5 => 5,
            TextureUnit::TEXTURE6 => 6,
            TextureUnit::TEXTURE7 => 7,
            TextureUnit::TEXTURE8 => 8,
            TextureUnit::TEXTURE9 => 9,
            TextureUnit::TEXTURE10 => 10,
            TextureUnit::TEXTURE11 => 11,
            TextureUnit::TEXTURE12 => 12,
            TextureUnit::TEXTURE13 => 13,
            TextureUnit::TEXTURE14 => 14,
            TextureUnit::TEXTURE15 => 15,
            TextureUnit::TEXTURE16 => 16,
            TextureUnit::TEXTURE17 => 17,
            TextureUnit::TEXTURE18 => 18,
            TextureUnit::TEXTURE19 => 19,
            TextureUnit::TEXTURE20 => 20,
            TextureUnit::TEXTURE21 => 21,
            TextureUnit::TEXTURE22 => 22,
            TextureUnit::TEXTURE23 => 23,
            TextureUnit::TEXTURE24 => 24,
            TextureUnit::TEXTURE25 => 25,
            TextureUnit::TEXTURE26 => 26,
            TextureUnit::TEXTURE27 => 27,
            TextureUnit::TEXTURE28 => 28,
            TextureUnit::TEXTURE29 => 29,
            TextureUnit::TEXTURE30 => 30,
            TextureUnit::TEXTURE31 => 31,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureUnpackColorSpaceConversion {
    NONE,
    BROWSER_DEFAULT_WEBGL,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePixelStorage {
    PACK_ALIGNMENT(i32),
    UNPACK_ALIGNMENT(i32),
    UNPACK_FLIP_Y_WEBGL(bool),
    UNPACK_PREMULTIPLY_ALPHA_WEBGL(bool),
    UNPACK_COLORSPACE_CONVERSION_WEBGL(TextureUnpackColorSpaceConversion),
    PACK_ROW_LENGTH(i32),
    PACK_SKIP_PIXELS(i32),
    PACK_SKIP_ROWS(i32),
    UNPACK_ROW_LENGTH(i32),
    UNPACK_IMAGE_HEIGHT(i32),
    UNPACK_SKIP_PIXELS(i32),
    UNPACK_SKIP_ROWS(i32),
    UNPACK_SKIP_IMAGES(i32),
}

impl TexturePixelStorage {
    pub fn key(&self) -> u32 {
        self.gl_enum()
    }

    pub fn value(&self) -> i32 {
        match self {
            TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(v)
            | TexturePixelStorage::UNPACK_PREMULTIPLY_ALPHA_WEBGL(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            TexturePixelStorage::UNPACK_COLORSPACE_CONVERSION_WEBGL(v) => v.gl_enum() as i32,
            TexturePixelStorage::PACK_ALIGNMENT(v)
            | TexturePixelStorage::UNPACK_ALIGNMENT(v)
            | TexturePixelStorage::PACK_ROW_LENGTH(v)
            | TexturePixelStorage::PACK_SKIP_PIXELS(v)
            | TexturePixelStorage::PACK_SKIP_ROWS(v)
            | TexturePixelStorage::UNPACK_ROW_LENGTH(v)
            | TexturePixelStorage::UNPACK_IMAGE_HEIGHT(v)
            | TexturePixelStorage::UNPACK_SKIP_PIXELS(v)
            | TexturePixelStorage::UNPACK_SKIP_ROWS(v)
            | TexturePixelStorage::UNPACK_SKIP_IMAGES(v) => *v,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMagnificationFilter {
    LINEAR,
    NEAREST,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMinificationFilter {
    LINEAR,
    NEAREST,
    NEAREST_MIPMAP_NEAREST,
    LINEAR_MIPMAP_NEAREST,
    NEAREST_MIPMAP_LINEAR,
    LINEAR_MIPMAP_LINEAR,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapMethod {
    REPEAT,
    CLAMP_TO_EDGE,
    MIRRORED_REPEAT,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareFunction {
    LEQUAL,
    GEQUAL,
    LESS,
    GREATER,
    EQUAL,
    NOTEQUAL,
    ALWAYS,
    NEVER,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompareMode {
    NONE,
    COMPARE_REF_TO_TEXTURE,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextureParameter {
    MAG_FILTER(TextureMagnificationFilter),
    MIN_FILTER(TextureMinificationFilter),
    WRAP_S(TextureWrapMethod),
    WRAP_T(TextureWrapMethod),
    WRAP_R(TextureWrapMethod),
    BASE_LEVEL(i32),
    COMPARE_FUNC(TextureCompareFunction),
    COMPARE_MODE(TextureCompareMode),
    MAX_LEVEL(i32),
    MAX_LOD(f32),
    MIN_LOD(f32),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompressedFormat {
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc` enabled.
    RGBA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT1,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT3,
    /// Available when extension `WEBGL_compressed_texture_s3tc_srgb` enabled.
    SRGB_ALPHA_S3TC_DXT5,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SIGNED_R11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SIGNED_RG11_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGBA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_ALPHA8_ETC2_EAC,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    RGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_etc` enabled.
    SRGB8_PUNCHTHROUGH_ALPHA1_ETC2,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGB_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGBA_PVRTC_2BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGB_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_pvrtc` enabled.
    RGBA_PVRTC_4BPPV1_IMG,
    /// Available when extension `WEBGL_compressed_texture_etc1` enabled.
    RGB_ETC1_WEBGL,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_4x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_5x4,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_5x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_6x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_6x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_8x8,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x5,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x6,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_10x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_12x10,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    RGBA_ASTC_12x12,
    /// Available when extension `WEBGL_compressed_texture_astc` enabled.
    SRGB8_ALPHA8_ASTC_12x12,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGBA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    SRGB_ALPHA_BPTC_UNORM,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGB_BPTC_SIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_bptc` enabled.
    RGB_BPTC_UNSIGNED_FLOAT,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    SIGNED_RED_RGTC1,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    RED_GREEN_RGTC2,
    /// Available when extension `EXT_texture_compression_rgtc` enabled.
    SIGNED_RED_GREEN_RGTC2,
}

/// Memory freeing policies.
pub enum MemoryPolicy {
    Default,
    Restorable(Rc<RefCell<dyn Fn() -> TextureSource>>),
    Unfree,
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self::Default
    }
}

pub enum TextureSource {
    Preallocate {
        width: usize,
        height: usize,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Binary {
        width: usize,
        height: usize,
        data: Box<dyn AsRef<[u8]>>,
        format: TextureFormat,
        data_type: TextureDataType,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Uint8Array {
        width: usize,
        height: usize,
        data: Uint8Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Uint16Array {
        width: usize,
        height: usize,
        data: Uint16Array,
        format: TextureFormat,
        /// Only [`TextureDataType::UNSIGNED_SHORT`],
        /// [`TextureDataType::UNSIGNED_SHORT_5_6_5`],
        /// [`TextureDataType::UNSIGNED_SHORT_4_4_4_4`],
        /// [`TextureDataType::UNSIGNED_SHORT_5_5_5_1`],
        /// [`TextureDataType::HALF_FLOAT`] are accepted.
        data_type: TextureDataType,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Uint32Array {
        width: usize,
        height: usize,
        data: Uint32Array,
        format: TextureFormat,
        /// Only [`TextureDataType::UNSIGNED_INT`],
        /// [`TextureDataType::UNSIGNED_INT_24_8`]
        /// are accepted.
        data_type: TextureDataType,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    Float32Array {
        width: usize,
        height: usize,
        data: Float32Array,
        format: TextureFormat,
        src_offset: usize,
        pixel_storages: Vec<TexturePixelStorage>,
    },
    HtmlCanvasElement {
        canvas: HtmlCanvasElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    HtmlImageElement {
        image: HtmlImageElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    HtmlVideoElement {
        video: HtmlVideoElement,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    ImageData {
        data: ImageData,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
    ImageBitmap {
        bitmap: ImageBitmap,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        custom_size: Option<(usize, usize)>,
    },
}

impl TextureSource {
    pub fn width(&self) -> usize {
        match self {
            TextureSource::Preallocate { width, .. }
            | TextureSource::Binary { width, .. }
            | TextureSource::Uint8Array { width, .. }
            | TextureSource::Uint16Array { width, .. }
            | TextureSource::Uint32Array { width, .. }
            | TextureSource::Float32Array { width, .. } => *width,
            TextureSource::HtmlCanvasElement {
                canvas,
                custom_size,
                ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(canvas.width() as usize),
            TextureSource::HtmlImageElement {
                image, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(image.natural_width() as usize),
            TextureSource::HtmlVideoElement {
                video, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(video.video_width() as usize),
            TextureSource::ImageData {
                data, custom_size, ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(data.width() as usize),
            TextureSource::ImageBitmap {
                bitmap,
                custom_size,
                ..
            } => custom_size
                .map(|(width, _)| width)
                .unwrap_or(bitmap.width() as usize),
        }
    }

    pub fn height(&self) -> usize {
        match self {
            TextureSource::Preallocate { height, .. }
            | TextureSource::Binary { height, .. }
            | TextureSource::Uint8Array { height, .. }
            | TextureSource::Uint16Array { height, .. }
            | TextureSource::Uint32Array { height, .. }
            | TextureSource::Float32Array { height, .. } => *height,
            TextureSource::HtmlCanvasElement {
                canvas,
                custom_size,
                ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(canvas.height() as usize),
            TextureSource::HtmlImageElement {
                image, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(image.natural_height() as usize),
            TextureSource::HtmlVideoElement {
                video, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(video.video_height() as usize),
            TextureSource::ImageData {
                data, custom_size, ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(data.height() as usize),
            TextureSource::ImageBitmap {
                bitmap,
                custom_size,
                ..
            } => custom_size
                .map(|(_, height)| height)
                .unwrap_or(bitmap.height() as usize),
        }
    }

    pub fn pixel_storages(&self) -> &[TexturePixelStorage] {
        match self {
            TextureSource::Preallocate { pixel_storages, .. }
            | TextureSource::Binary { pixel_storages, .. }
            | TextureSource::Uint8Array { pixel_storages, .. }
            | TextureSource::Uint16Array { pixel_storages, .. }
            | TextureSource::Uint32Array { pixel_storages, .. }
            | TextureSource::Float32Array { pixel_storages, .. }
            | TextureSource::HtmlCanvasElement { pixel_storages, .. }
            | TextureSource::HtmlImageElement { pixel_storages, .. }
            | TextureSource::HtmlVideoElement { pixel_storages, .. }
            | TextureSource::ImageData { pixel_storages, .. }
            | TextureSource::ImageBitmap { pixel_storages, .. } => &pixel_storages,
        }
    }

    // Under WebGL2, we use `tex_storage` + `tex_sub_image` instead of `tex_image`.
    // fn tex_image(
    //     &self,
    //     gl: &WebGl2RenderingContext,
    //     target: TextureTarget,
    //     internal_format: TextureInternalFormat,
    //     level: usize,
    // ) -> Result<(), Error> {
    //     // setups pixel storage parameters
    //     self.pixel_storages()
    //         .iter()
    //         .for_each(|param| gl.pixel_storei(param.key(), param.value()));

    //     // buffers image data
    //     let result = match self {
    //         TextureSource::Preallocate {
    //             width,
    //             height,
    //             format,
    //             data_type,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             data_type.gl_enum(),
    //             None
    //         ),
    //         TextureSource::FromBinary {
    //             width,
    //             height,
    //             data,
    //             format,
    //             data_type,
    //             src_offset,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             data_type.gl_enum(),
    //             data.as_ref().as_ref(),
    //             *src_offset  as u32
    //         ),
    //         TextureSource::FromUint8Array {
    //             width,
    //             height,
    //             data,
    //             format,
    //             src_offset,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             WebGl2RenderingContext::UNSIGNED_BYTE,
    //             data,
    //             *src_offset  as u32
    //         ),
    //         TextureSource::FromUint16Array {
    //             width,
    //             height,
    //             data,
    //             format,
    //             data_type,
    //             src_offset,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             data_type.gl_enum(),
    //             data,
    //             *src_offset  as u32
    //         ),
    //         TextureSource::FromUint32Array {
    //             width,
    //             height,
    //             data,
    //             format,
    //             data_type,
    //             src_offset,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             data_type.gl_enum(),
    //             data,
    //             *src_offset  as u32
    //         ),
    //         TextureSource::FromFloat32Array {
    //             width,
    //             height,
    //             data,
    //             format,
    //             src_offset,
    //             ..
    //         } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
    //             target.gl_enum(),
    //             level as i32,
    //             internal_format.gl_enum() as i32,
    //             *width as i32,
    //             *height as i32,
    //             0,
    //             format.gl_enum(),
    //             WebGl2RenderingContext::FLOAT,
    //             data,
    //             *src_offset  as u32
    //         ),
    //         TextureSource::FromHtmlCanvasElement {
    //             format,
    //             data_type,
    //             canvas,
    //             custom_size,
    //             ..
    //         } => match custom_size {
    //             Some((width, height)) => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_canvas_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 *width as i32,
    //                 *height as i32,
    //                 0,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 canvas,
    //             ),
    //             None => gl.tex_image_2d_with_u32_and_u32_and_html_canvas_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 canvas,
    //             ),
    //         },
    //         TextureSource::FromHtmlImageElement {
    //             format,
    //             data_type,
    //             image,
    //             custom_size,
    //             ..
    //         } => match custom_size {
    //             Some((width, height)) => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_image_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 *width as i32,
    //                 *height as i32,
    //                 0,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 image,
    //             ),
    //             None => gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 image,
    //             ),
    //         },
    //         TextureSource::FromHtmlVideoElement {
    //             video,
    //             format,
    //             data_type,
    //             custom_size ,
    //             ..
    //         } => match custom_size {
    //             Some((width, height)) => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_video_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 *width as i32,
    //                 *height as i32,
    //                 0,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 video,
    //             ),
    //             None => gl.tex_image_2d_with_u32_and_u32_and_html_video_element(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 video,
    //             ),
    //         },
    //         TextureSource::FromImageData {
    //             data,
    //             format,
    //             data_type,
    //             custom_size,
    //             ..
    //         } => match custom_size {
    //             Some((width, height)) => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_image_data(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 *width as i32,
    //                 *height as i32,
    //                 0,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 data,
    //             ),
    //             None => gl.tex_image_2d_with_u32_and_u32_and_image_data(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 data,
    //             ),
    //         },
    //         TextureSource::FromImageBitmap {
    //             bitmap,
    //             format,
    //             data_type,
    //             custom_size,
    //             ..
    //         } => match custom_size {
    //             Some((width, height)) => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_image_bitmap(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 *width as i32,
    //                 *height as i32,
    //                 0,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 bitmap,
    //             ),
    //             None => gl.tex_image_2d_with_u32_and_u32_and_image_bitmap(
    //                 target.gl_enum(),
    //                 level as i32,
    //                 internal_format.gl_enum() as i32,
    //                 format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 bitmap,
    //             ),
    //         }
    //     };

    //     result.map_err(|err| Error::TexImageFailure(err.as_string()))
    // }

    fn tex_sub_image(
        &self,
        gl: &WebGl2RenderingContext,
        target: TextureTarget,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image sub data
        let result = match self {
            TextureSource::Preallocate {
                width,
                height,
                format,
                data_type,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                None
            ),
            TextureSource::Binary {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_u8_array_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data.as_ref().as_ref(),
                *src_offset  as u32
            ),

            TextureSource::Uint8Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::UNSIGNED_BYTE,
                data,
                *src_offset  as u32
            ),
            TextureSource::Uint16Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data,
                *src_offset  as u32
            ),
            TextureSource::Uint32Array {
                width,
                height,
                data,
                format,
                data_type,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                data,
                *src_offset  as u32
            ),
            TextureSource::Float32Array {
                width,
                height,
                data,
                format,
                src_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_array_buffer_view_and_src_offset(
                target.gl_enum(),
                level as i32,
                x_offset as i32,
                y_offset as i32,
                *width as i32,
                *height as i32,
                format.gl_enum(),
                WebGl2RenderingContext::FLOAT,
                data,
                *src_offset  as u32
            ),
            TextureSource::HtmlCanvasElement {
                format,
                data_type,
                canvas,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    canvas,
                ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_canvas_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    canvas.width() as i32,
                    canvas.height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    canvas,
                ),
            },
            TextureSource::HtmlImageElement {
                format,
                data_type,
                image,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    image,
                ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    image.natural_width() as i32,
                    image.natural_height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    image,
                ),
            },
            TextureSource::HtmlVideoElement {
                video,
                format,
                data_type,
                custom_size ,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    video,
                ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_video_element(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    video.video_width() as i32,
                    video.video_height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    video,
                ),
            },
            TextureSource::ImageData {
                data,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    data,
                ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_data(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    data.width() as i32,
                    data.height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    data,
                ),
            },
            TextureSource::ImageBitmap {
                bitmap,
                format,
                data_type,
                custom_size,
                ..
            } => match custom_size {
                Some((width, height)) => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    *width as i32,
                    *height as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    bitmap,
                ),
                None => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_image_bitmap(
                    target.gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    bitmap.width() as i32,
                    bitmap.height() as i32,
                    format.gl_enum(),
                    data_type.gl_enum(),
                    bitmap,
                ),
            }
        };

        result.map_err(|err| Error::TexImageFailure(err.as_string()))
    }
}

struct Runtime2D {
    id: usize,
    gl: WebGl2RenderingContext,
    store_id: Uuid,
    texture: WebGlTexture,
    lru_node: *mut LruNode<usize>,
    using: bool,

    used_memory: *mut usize,
    descriptors: *mut HashMap<usize, Weak<RefCell<TextureDescriptor2DInner>>>,
    lru: *mut Lru<usize>,
}

impl Drop for Runtime2D {
    fn drop(&mut self) {
        unsafe {
            (*self.descriptors).remove(&self.id);
            (*self.lru).remove(self.lru_node);
            self.gl.delete_texture(Some(&self.texture));
        }
    }
}

struct TextureDescriptor2DInner {
    name: Option<Cow<'static, str>>,
    max_width: usize,
    max_height: usize,
    internal_format: TextureInternalFormat,
    generate_mipmap: bool,
    memory_policy: MemoryPolicy,

    queue: Vec<(TextureSource, usize, usize, usize)>,

    runtime: Option<Box<Runtime2D>>,
}

impl TextureDescriptor2DInner {
    #[inline]
    fn max_mipmap_level(&self) -> usize {
        (self.max_width as f64)
            .max(self.max_height as f64)
            .log2()
            .floor() as usize
    }

    #[inline]
    fn width(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.max_width >> level).max(1))
    }

    #[inline]
    fn height(&self, level: usize) -> Option<usize> {
        let max_level = self.max_mipmap_level();
        if level > max_level {
            return None;
        }

        Some((self.max_height >> level).max(1))
    }

    fn verify_size_tex_image(
        &self,
        level: usize,
        width: usize,
        height: usize,
    ) -> Result<(), Error> {
        if self.width(level).map(|w| w != width).unwrap_or(true) {
            return Err(Error::TexImageSizeMismatched);
        }
        if self.height(level).map(|h| h != height).unwrap_or(true) {
            return Err(Error::TexImageSizeMismatched);
        }

        Ok(())
    }

    fn verify_size_tex_sub_image(
        &self,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        if self
            .width(level)
            .map(|w| width + x_offset > w)
            .unwrap_or(true)
        {
            return Err(Error::TexImageSizeMismatched);
        }
        if self
            .height(level)
            .map(|h| height + y_offset > h)
            .unwrap_or(true)
        {
            return Err(Error::TexImageSizeMismatched);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct TextureDescriptor2D(Rc<RefCell<TextureDescriptor2DInner>>);

impl TextureDescriptor2D {
    pub fn new(
        max_width: usize,
        max_height: usize,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptor2DInner {
            name: None,
            max_width,
            max_height,
            internal_format,
            generate_mipmap,
            memory_policy,

            queue: Vec::new(),

            runtime: None,
        })))
    }

    pub fn with_source(
        source: TextureSource,
        internal_format: TextureInternalFormat,
        generate_mipmap: bool,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(TextureDescriptor2DInner {
            name: None,
            max_width: source.width(),
            max_height: source.height(),
            internal_format,
            generate_mipmap,
            memory_policy,

            queue: vec![(source, 0, 0, 0)],

            runtime: None,
        })))
    }

    #[inline]
    pub fn max_mipmap_level(&self) -> usize {
        self.0.borrow().max_mipmap_level()
    }

    pub fn max_width(&self) -> usize {
        self.0.borrow().max_width
    }

    pub fn max_height(&self) -> usize {
        self.0.borrow().max_height
    }

    pub fn width(&self, level: usize) -> Option<usize> {
        self.0.borrow().width(level)
    }

    pub fn height(&self, level: usize) -> Option<usize> {
        self.0.borrow().height(level)
    }

    /// Returns buffer descriptor name.
    pub fn name(&self) -> Ref<Option<Cow<'static, str>>> {
        Ref::map(self.0.borrow(), |inner| &inner.name)
    }

    /// Sets buffer descriptor name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.0.borrow_mut().name.replace(Cow::Owned(name.into()));
    }

    /// Sets buffer descriptor name.
    pub fn set_name_str(&mut self, name: &'static str) {
        self.0.borrow_mut().name.replace(Cow::Borrowed(name));
    }

    pub fn memory_policy(&self) -> Ref<MemoryPolicy> {
        Ref::map(self.0.borrow(), |inner| &inner.memory_policy)
    }

    pub fn generate_mipmap(&self) -> bool {
        self.0.borrow().generate_mipmap
    }

    pub fn internal_format(&self) -> TextureInternalFormat {
        self.0.borrow().internal_format
    }

    pub fn tex_image(&mut self, source: TextureSource, level: usize) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_size_tex_image(level, width, height)?;

        inner.queue.push((source, level, 0, 0));
        Ok(())
    }

    pub fn tex_sub_image(
        &mut self,
        source: TextureSource,
        level: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        let mut inner = self.0.borrow_mut();
        let width = source.width();
        let height = source.height();
        inner.verify_size_tex_sub_image(level, width, height, x_offset, y_offset)?;

        inner.queue.push((source, level, x_offset, y_offset));
        Ok(())
    }
}

pub struct TextureStore {
    gl: WebGl2RenderingContext,
    id: Uuid,
    counter: usize,
    available_memory: usize,
    used_memory: *mut usize,
    descriptors_2d: *mut HashMap<usize, Weak<RefCell<TextureDescriptor2DInner>>>,
    lru: *mut Lru<usize>,

    max_texture_size: *mut Option<u32>,
    max_texture_image_units: *mut Option<u32>,
    compressed_s3tc_supported: *mut Option<bool>,
    compressed_s3tc_srgb_supported: *mut Option<bool>,
    compressed_etc_supported: *mut Option<bool>,
    compressed_pvrtc_supported: *mut Option<bool>,
    compressed_etc1_supported: *mut Option<bool>,
    compressed_astc_supported: *mut Option<bool>,
    compressed_bptc_supported: *mut Option<bool>,
    compressed_rgtc_supported: *mut Option<bool>,
}

impl Drop for TextureStore {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.used_memory));
            drop(Box::from_raw(self.descriptors_2d));
            drop(Box::from_raw(self.lru));
            drop(Box::from_raw(self.max_texture_size));
            drop(Box::from_raw(self.max_texture_image_units));
            drop(Box::from_raw(self.compressed_s3tc_supported));
            drop(Box::from_raw(self.compressed_s3tc_srgb_supported));
            drop(Box::from_raw(self.compressed_etc_supported));
            drop(Box::from_raw(self.compressed_pvrtc_supported));
            drop(Box::from_raw(self.compressed_etc1_supported));
            drop(Box::from_raw(self.compressed_astc_supported));
            drop(Box::from_raw(self.compressed_bptc_supported));
            drop(Box::from_raw(self.compressed_rgtc_supported));
        }
    }
}

impl TextureStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            id: Uuid::new_v4(),
            counter: 0,
            available_memory: i32::MAX as usize,
            used_memory: Box::leak(Box::new(0)),
            descriptors_2d: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),

            max_texture_size: Box::leak(Box::new(None)),
            max_texture_image_units: Box::leak(Box::new(None)),
            compressed_s3tc_supported: Box::leak(Box::new(None)),
            compressed_s3tc_srgb_supported: Box::leak(Box::new(None)),
            compressed_etc_supported: Box::leak(Box::new(None)),
            compressed_pvrtc_supported: Box::leak(Box::new(None)),
            compressed_etc1_supported: Box::leak(Box::new(None)),
            compressed_astc_supported: Box::leak(Box::new(None)),
            compressed_bptc_supported: Box::leak(Box::new(None)),
            compressed_rgtc_supported: Box::leak(Box::new(None)),
        }
    }

    unsafe fn next(&mut self) -> usize {
        if (*self.descriptors_2d).len() == usize::MAX {
            panic!("too many descriptors, only {} are accepted", usize::MAX);
        }

        self.counter = self.counter.wrapping_add(1);
        while (*self.descriptors_2d).contains_key(&self.counter) {
            self.counter = self.counter.wrapping_add(1);
        }
        self.counter
    }

    fn bound_parameters(&self) -> (u32, Option<WebGlTexture>) {
        (
            self.gl
                .get_parameter(WebGl2RenderingContext::ACTIVE_TEXTURE)
                .ok()
                .and_then(|v| v.as_f64())
                .map(|v| v as u32)
                .unwrap(),
            self.gl
                .get_parameter(WebGl2RenderingContext::TEXTURE_BINDING_2D)
                .and_then(|v| v.dyn_into::<WebGlTexture>())
                .ok(),
        )
    }

    fn verify_texture_size(&self, width: usize, height: usize) -> Result<(), Error> {
        let max = self.max_texture_size() as usize;
        if width > max || height > max {
            return Err(Error::TextureSizeOverflowed {
                max: (max, max),
                value: (width, height),
            });
        }

        Ok(())
    }

    fn verify_texture_unit(&self, unit: TextureUnit) -> Result<(), Error> {
        let unit = (unit.unit_index() + 1) as u32;
        let max = self.max_texture_image_units();
        if unit > max {
            return Err(Error::TextureUnitOverflowed { max, value: unit });
        }

        Ok(())
    }

    pub fn use_texture_2d(
        &mut self,
        descriptor: &TextureDescriptor2D,
        unit: TextureUnit,
    ) -> Result<WebGlTexture, Error> {
        self.verify_texture_unit(unit)?;

        unsafe {
            let mut inner = descriptor.0.borrow_mut();

            let texture = match inner.runtime.as_mut() {
                Some(runtime) => {
                    if runtime.store_id != self.id {
                        panic!("share texture descriptor between texture store is not allowed");
                    }

                    runtime.using = true;
                    (*self.lru).cache(runtime.lru_node);

                    runtime.texture.clone()
                }
                None => {
                    debug!(
                        target: "TextureBuffer",
                        "create new texture for {}",
                        inner.name.as_deref().unwrap_or("unnamed"),
                    );

                    let texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl.active_texture(unit.gl_enum());
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                    self.gl.tex_storage_2d(
                        WebGl2RenderingContext::TEXTURE_2D,
                        (1 + inner.max_mipmap_level()) as i32,
                        inner.internal_format.gl_enum(),
                        inner.max_width as i32,
                        inner.max_height as i32,
                    );

                    let id = self.next();
                    let lru_node = LruNode::new(id);
                    (*self.descriptors_2d).insert(id, Rc::downgrade(&descriptor.0));
                    (*self.lru).cache(lru_node);
                    inner.runtime = Some(Box::new(Runtime2D {
                        id,
                        gl: self.gl.clone(),
                        store_id: self.id,
                        texture: texture.clone(),
                        lru_node,
                        using: true,

                        used_memory: self.used_memory,
                        descriptors: self.descriptors_2d,
                        lru: self.lru,
                    }));
                    texture
                }
            };

            if inner.queue.len() != 0 {
                self.gl.active_texture(unit.gl_enum());
                self.gl
                    .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                for (source, level, x_offset, y_offset) in inner.queue.drain(..) {
                    self.verify_texture_size(source.width(), source.height())?;
                    source.tex_sub_image(
                        &self.gl,
                        TextureTarget::TEXTURE_2D,
                        level,
                        x_offset,
                        y_offset,
                    )?;
                }
                if inner.generate_mipmap {
                    self.gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
                }
            }

            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

            Ok(texture)
        }
    }

    pub fn max_texture_size(&self) -> u32 {
        unsafe {
            if let Some(size) = *self.max_texture_size {
                return size;
            }

            let size = self
                .gl
                .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE)
                .ok()
                .and_then(|v| v.as_f64())
                .map(|v| v as u32)
                .unwrap();
            *self.max_texture_size = Some(size);
            size
        }
    }

    pub fn max_texture_image_units(&self) -> u32 {
        unsafe {
            if let Some(size) = *self.max_texture_image_units {
                return size;
            }

            let size = self
                .gl
                .get_parameter(WebGl2RenderingContext::MAX_TEXTURE_IMAGE_UNITS)
                .ok()
                .and_then(|v| v.as_f64())
                .map(|v| v as u32)
                .unwrap();
            *self.max_texture_image_units = Some(size);
            size
        }
    }
}

macro_rules! compressed_supported {
    ($(($func:ident, $field:ident, $($extensions:tt),+))+) => {
        impl TextureStore {
            $(
                pub fn $func(&self) -> bool {
                    unsafe {
                        if let Some(supported) = *self.$field {
                            return supported;
                        }

                        let supported = $(
                            self.gl.get_extension($extensions)
                            .map(|extension| extension.is_some())
                            .unwrap_or(false)
                        ) || +;
                        *self.$field = Some(supported);
                        supported
                    }
                }
            )+
        }
    };
}

compressed_supported! {
    (compressed_s3tc_supported, compressed_s3tc_supported, "WEBGL_compressed_texture_s3tc", "MOZ_WEBGL_compressed_texture_s3tc", "WEBKIT_WEBGL_compressed_texture_s3tc")
    (compressed_s3tc_srgb_supported, compressed_s3tc_srgb_supported, "WEBGL_compressed_texture_s3tc_srgb")
    (compressed_etc_supported, compressed_etc_supported, "WEBGL_compressed_texture_etc")
    (compressed_pvrtc_supported, compressed_pvrtc_supported, "WEBGL_compressed_texture_pvrtc")
    (compressed_etc1_supported, compressed_etc1_supported, "WEBGL_compressed_texture_etc1")
    (compressed_astc_supported, compressed_astc_supported, "WEBGL_compressed_texture_astc")
    (compressed_bptc_supported, compressed_bptc_supported, "EXT_texture_compression_bptc")
    (compressed_rgtc_supported, compressed_rgtc_supported, "EXT_texture_compression_rgtc")
}
