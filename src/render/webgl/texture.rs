use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use uuid::Uuid;
use web_sys::{HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlTexture};

use super::{conversion::ToGlEnum, error::Error};

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
    /// Color renderable when extension EXT_color_buffer_float is enabled
    RGBA32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled
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
    /// Color renderable when extension EXT_color_buffer_float is enabled
    R11F_G11F_B10F,
    RGB9_E5,
    /// Color renderable when extension EXT_color_buffer_float is enabled
    RG32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled
    RG16F,
    RG8_SNORM,
    /// Color renderable when extension EXT_color_buffer_float is enabled
    R32F,
    /// Color renderable when extension EXT_color_buffer_float is enabled
    R16F,
    R8_SNORM,
    DEPTH_COMPONENT32F,
    DEPTH_COMPONENT24,
    DEPTH_COMPONENT16,
    DEPTH32F_STENCIL8,
    DEPTH24_STENCIL8,
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
    Custom(u32),
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
            TextureUnit::Custom(index) => *index as i32,
        }
    }

    pub fn max_combined_texture_image_units(gl: &WebGl2RenderingContext) -> u32 {
        let value = gl
            .get_parameter(WebGl2RenderingContext::MAX_COMBINED_TEXTURE_IMAGE_UNITS)
            .unwrap();
        value.as_f64().unwrap() as u32
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

pub enum TextureSource {
    Preallocate {
        internal_format: TextureInternalFormat,
        width: i32,
        height: i32,
        format: TextureFormat,
        data_type: TextureDataType,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromBinary {
        internal_format: TextureInternalFormat,
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
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        canvas: Box<dyn AsRef<HtmlCanvasElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlCanvasElementWithSize {
        internal_format: TextureInternalFormat,
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
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        image: Box<dyn AsRef<HtmlImageElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
    FromHtmlImageElementWithSize {
        format: TextureFormat,
        width: i32,
        height: i32,
        internal_format: TextureInternalFormat,
        data_type: TextureDataType,
        image: Box<dyn AsRef<HtmlImageElement>>,
        pixel_storages: Vec<TexturePixelStorage>,
        x_offset: i32,
        y_offset: i32,
    },
}

impl TextureSource {
    fn pixel_storages(&self) -> &[TexturePixelStorage] {
        match self {
            TextureSource::Preallocate { pixel_storages, .. } => &pixel_storages,
            TextureSource::FromBinary { pixel_storages, .. } => &pixel_storages,
            TextureSource::FromHtmlCanvasElement { pixel_storages, .. } => &pixel_storages,
            TextureSource::FromHtmlCanvasElementWithSize { pixel_storages, .. } => &pixel_storages,
            TextureSource::FromHtmlImageElement { pixel_storages, .. } => &pixel_storages,
            TextureSource::FromHtmlImageElementWithSize { pixel_storages, .. } => &pixel_storages,
        }
    }

    fn tex_image(
        &self,
        gl: &WebGl2RenderingContext,
        tex_target: u32,
        level: i32,
    ) -> Result<(), Error> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image data
        let result = match self {
            TextureSource::Preallocate {
                internal_format,
                width,
                height,
                format,
                data_type,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                tex_target,
                level,
                internal_format.gl_enum() as i32,
                *width,
                *height,
                0,
                format.gl_enum(),
                data_type.gl_enum(),
                None
            ),
            TextureSource::FromBinary {
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
                level,
                internal_format.gl_enum() as i32,
                *width,
                *height,
                0,
                format.gl_enum(),
                data_type.gl_enum(),
                data.as_ref().as_ref(),
                *src_offset
            ),
            TextureSource::FromHtmlCanvasElement {
                internal_format,
                format,
                data_type,
                canvas,
                ..
            } => gl
            .tex_image_2d_with_u32_and_u32_and_html_canvas_element(
                tex_target,
                level,
                internal_format.gl_enum() as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureSource::FromHtmlCanvasElementWithSize {
                internal_format,
                width,
                height,
                format,
                data_type,
                canvas,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_canvas_element(
                tex_target,
                level,
                internal_format.gl_enum() as i32,
                *width,
                *height,
                0,
                format.gl_enum(),
                data_type.gl_enum(),
                canvas.as_ref().as_ref()
            ),
            TextureSource::FromHtmlImageElement {
                internal_format,
                format,
                data_type,
                image,
                ..
            } => gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                tex_target,
                level,
                internal_format.gl_enum() as i32,
                format.gl_enum(),
                data_type.gl_enum(),
                image.as_ref().as_ref(),
            ),
            TextureSource::FromHtmlImageElementWithSize {
                format,
                width,
                height,
                internal_format,
                data_type,
                image,
                ..
            } => gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_html_image_element(
                tex_target,
                level,
                internal_format.gl_enum() as i32,
                *width,
                *height,
                0,
                format.gl_enum(),
                data_type.gl_enum(),
                image.as_ref().as_ref()
            ),
        };

        result.map_err(|err| Error::TexImageFailure(err.as_string()))
    }

    fn tex_sub_image(
        &self,
        gl: &WebGl2RenderingContext,
        tex_target: u32,
        level: i32,
    ) -> Result<(), Error> {
        // setups pixel storage parameters
        self.pixel_storages()
            .iter()
            .for_each(|param| gl.pixel_storei(param.key(), param.value()));

        // buffers image data
        let result = match self {
            TextureSource::Preallocate {
                width,
                height,
                format,
                data_type,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
                tex_target,
                level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.gl_enum(),
                data_type.gl_enum(),
                None,
            ),
            TextureSource::FromBinary {
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
                level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.gl_enum(),
                data_type.gl_enum(),
                data.as_ref().as_ref(),
                *src_offset,
            ),
            TextureSource::FromHtmlCanvasElement {
                format,
                data_type,
                canvas,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_u32_and_u32_and_html_canvas_element(
                tex_target,
                level,
                *x_offset,
                *y_offset,
                format.gl_enum(),
                data_type.gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureSource::FromHtmlCanvasElementWithSize {
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
                level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.gl_enum(),
                data_type.gl_enum(),
                canvas.as_ref().as_ref(),
            ),
            TextureSource::FromHtmlImageElement {
                format,
                data_type,
                image,
                x_offset,
                y_offset,
                ..
            } => gl.tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
                tex_target,
                level,
                *x_offset,
                *y_offset,
                format.gl_enum(),
                data_type.gl_enum(),
                image.as_ref().as_ref(),
            ),
            TextureSource::FromHtmlImageElementWithSize {
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
                level,
                *x_offset,
                *y_offset,
                *width,
                *height,
                format.gl_enum(),
                data_type.gl_enum(),
                image.as_ref().as_ref(),
            ),
        };

        result.map_err(|err| Error::TexImageFailure(err.as_string()))
    }
}

enum TextureData {
    Texture2D(HashMap<i32, TextureSource>),
    TextureCubeMap {
        positive_x: HashMap<i32, TextureSource>,
        negative_x: HashMap<i32, TextureSource>,
        positive_y: HashMap<i32, TextureSource>,
        negative_y: HashMap<i32, TextureSource>,
        positive_z: HashMap<i32, TextureSource>,
        negative_z: HashMap<i32, TextureSource>,
    },
}

impl TextureData {
    fn texture_target(&self) -> u32 {
        match self {
            TextureData::Texture2D(_) => WebGl2RenderingContext::TEXTURE_2D,
            TextureData::TextureCubeMap { .. } => WebGl2RenderingContext::TEXTURE_CUBE_MAP,
        }
    }

    fn tex_image(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        match self {
            TextureData::Texture2D(data) => {
                for (level, data) in data.iter() {
                    data.tex_image(gl, WebGl2RenderingContext::TEXTURE_2D, *level)?
                }
            }
            TextureData::TextureCubeMap {
                positive_x,
                negative_x,
                positive_y,
                negative_y,
                positive_z,
                negative_z,
            } => {
                for (level, data) in positive_x.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
                        *level,
                    )?
                }
                for (level, data) in negative_x.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
                        *level,
                    )?
                }
                for (level, data) in positive_y.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
                        *level,
                    )?
                }
                for (level, data) in negative_y.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                        *level,
                    )?
                }
                for (level, data) in positive_z.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
                        *level,
                    )?
                }
                for (level, data) in negative_z.iter() {
                    data.tex_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                        *level,
                    )?
                }
            }
        };

        Ok(())
    }

    fn tex_sub_image(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        match self {
            TextureData::Texture2D(data) => {
                for (level, data) in data.iter() {
                    data.tex_sub_image(gl, WebGl2RenderingContext::TEXTURE_2D, *level)?
                }
            }
            TextureData::TextureCubeMap {
                positive_x,
                negative_x,
                positive_y,
                negative_y,
                positive_z,
                negative_z,
            } => {
                for (level, data) in positive_x.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
                        *level,
                    )?
                }
                for (level, data) in negative_x.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
                        *level,
                    )?
                }
                for (level, data) in positive_y.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
                        *level,
                    )?
                }
                for (level, data) in negative_y.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
                        *level,
                    )?
                }
                for (level, data) in positive_z.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
                        *level,
                    )?
                }
                for (level, data) in negative_z.iter() {
                    data.tex_sub_image(
                        gl,
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
                        *level,
                    )?
                }
            }
        };

        Ok(())
    }
}

enum TextureStatus {
    Unchanged { id: Uuid, target: u32 },
    UpdateTexture { id: Option<Uuid>, data: TextureData },
    UpdateSubTexture { id: Uuid, data: TextureData },
}

#[derive(Clone)]
pub struct TextureDescriptor {
    status: Rc<RefCell<TextureStatus>>,
    generate_mipmap: bool,
}

impl TextureDescriptor {
    pub fn texture_2d_with_html_image_element<I: AsRef<HtmlImageElement> + 'static>(
        image: I,
        data_type: TextureDataType,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        level: i32,
        pixel_storages: Vec<TexturePixelStorage>,
        generate_mipmap: bool,
    ) -> Self {
        Self {
            status: Rc::new(RefCell::new(TextureStatus::UpdateTexture {
                id: None,
                data: TextureData::Texture2D(HashMap::from([(
                    level,
                    TextureSource::FromHtmlImageElement {
                        image: Box::new(image),
                        format,
                        internal_format,
                        data_type,
                        pixel_storages,
                        x_offset: 0,
                        y_offset: 0,
                    },
                )])),
            })),
            generate_mipmap,
        }
    }

    pub fn texture_cube_map_with_html_image_element(
        px: TextureSource,
        nx: TextureSource,
        py: TextureSource,
        ny: TextureSource,
        pz: TextureSource,
        nz: TextureSource,
        generate_mipmap: bool,
    ) -> Self {
        Self {
            status: Rc::new(RefCell::new(TextureStatus::UpdateTexture {
                id: None,
                data: TextureData::TextureCubeMap {
                    positive_x: HashMap::from([(0, px)]),
                    negative_x: HashMap::from([(0, nx)]),
                    positive_y: HashMap::from([(0, py)]),
                    negative_y: HashMap::from([(0, ny)]),
                    positive_z: HashMap::from([(0, pz)]),
                    negative_z: HashMap::from([(0, nz)]),
                },
            })),
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

    pub fn use_texture(
        &mut self,
        TextureDescriptor {
            status,
            generate_mipmap,
        }: &TextureDescriptor,
    ) -> Result<(u32, &WebGlTexture), Error> {
        let mut status = status.borrow_mut();
        match &*status {
            TextureStatus::Unchanged { id, target } => match self.store.get(id) {
                Some(texture) => Ok((*target, texture)),
                None => Err(Error::TextureStorageNotFount(id.clone())),
            },
            TextureStatus::UpdateTexture { id, data } => {
                // delete old texture
                if let Some(texture) = id.as_ref().and_then(|id| self.store.remove(id)) {
                    self.gl.delete_texture(Some(&texture));
                }

                let texture_target = data.texture_target();
                // create texture
                let Some(texture) = self.gl.create_texture() else {
                    return Err(Error::CreateTextureFailure);
                };

                // binds texture
                self.gl.bind_texture(texture_target, Some(&texture));
                // buffer images
                data.tex_image(&self.gl)?;
                // generates mipmaps
                if *generate_mipmap {
                    self.gl.generate_mipmap(texture_target);
                }

                // unbinds for good practice
                self.gl.bind_texture(texture_target, None);

                // stores it
                let id = Uuid::new_v4();
                let texture = self.store.entry(id.clone()).or_insert(texture);

                // updates status
                *status = TextureStatus::Unchanged {
                    id,
                    target: texture_target,
                };

                Ok((texture_target, texture))
            }
            TextureStatus::UpdateSubTexture { id, data } => {
                let Some(texture) = self.store.get(id) else {
                    return Err(Error::TextureStorageNotFount(id.clone()));
                };

                let texture_target = data.texture_target();
                // binds texture
                self.gl.bind_texture(texture_target, Some(texture));
                // buffers images
                data.tex_sub_image(&self.gl)?;
                // generates mipmaps
                if *generate_mipmap {
                    self.gl.generate_mipmap(texture_target);
                }
                // unbinds for good practice
                self.gl.bind_texture(texture_target, None);

                // updates status
                *status = TextureStatus::Unchanged {
                    id: id.clone(),
                    target: texture_target,
                };

                Ok((texture_target, texture))
            }
        }
    }
}
