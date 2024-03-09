use std::{cell::RefCell, rc::Rc};

use js_sys::Promise;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{ArrayBuffer, DataView, Uint8Array},
    Request, RequestInit, Response,
};

use crate::{
    error::Error, notify::Notifier, renderer::webgl::texture::{
        texture2d::{Builder, Texture2D, Texture2DBase},
        SamplerParameter, TextureCompressedFormat, TextureParameter, TextureSourceCompressed,
    }, share::Share, window
};

use super::{Loader, LoaderStatus};

pub const DDS_MAGIC_NUMBER: u32 = 0x20534444;
pub const DDS_DXT1: u32 = 0x31545844;
pub const DDS_DXT3: u32 = 0x33545844;
pub const DDS_DXT5: u32 = 0x35545844;
pub const DDS_DXT10: u32 = 0x30315844;
pub const DDS_HEADER_SIZE: u32 = 124;
pub const DDS_PIXELFORMAT_SIZE: u32 = 32;
pub const DDS_HEADER_FLAG_DDSD_CAPS: u32 = 0x1;
pub const DDS_HEADER_FLAG_DDSD_HEIGHT: u32 = 0x2;
pub const DDS_HEADER_FLAG_DDSD_WIDTH: u32 = 0x4;
pub const DDS_HEADER_FLAG_DDSD_PITCH: u32 = 0x8;
pub const DDS_HEADER_FLAG_DDSD_PIXELFORMAT: u32 = 0x1000;
pub const DDS_HEADER_FLAG_DDSD_MIPMAPCOUNT: u32 = 0x20000;
pub const DDS_HEADER_FLAG_DDSD_LINEARSIZE: u32 = 0x80000;
pub const DDS_HEADER_FLAG_DDSD_DEPTH: u32 = 0x800000;
pub const DDS_PIXELFORMAT_FLAG_ALPHA_PIXELS: u32 = 0x1;
pub const DDS_PIXELFORMAT_FLAG_ALPHA: u32 = 0x2;
pub const DDS_PIXELFORMAT_FLAG_FOUR_CC: u32 = 0x4;
pub const DDS_PIXELFORMAT_FLAG_RGB: u32 = 0x40;
pub const DDS_PIXELFORMAT_FLAG_YUV: u32 = 0x200;
pub const DDS_PIXELFORMAT_FLAG_LUMINANCE: u32 = 0x20000;

pub struct Header {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mipmap_count: u32,
    pub reserved1: [u32; 11],
    pub pixel_format: PixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
}

impl Header {
    /// Returns `true` if `DDSD_CAPS` flag is available.
    pub fn ddsd_caps(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_CAPS != 0
    }

    /// Returns `true` if `DDSD_HEIGHT` flag is available.
    pub fn ddsd_height(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_HEIGHT != 0
    }

    /// Returns `true` if `DDSD_WIDTH` flag is available.
    pub fn ddsd_width(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_WIDTH != 0
    }

    /// Returns `true` if `DDSD_PITCH` flag is available.
    pub fn ddsd_pitch(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_PITCH != 0
    }

    /// Returns `true` if `DDSD_PIXELFORMAT` flag is available.
    pub fn ddsd_pixel_format(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_PIXELFORMAT != 0
    }

    /// Returns `true` if `DDSD_MIPMAPCOUNT` flag is available.
    pub fn ddsd_mipmap_count(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_MIPMAPCOUNT != 0
    }

    /// Returns `true` if `DDSD_LINEARSIZE` flag is available.
    pub fn ddsd_linear_size(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_LINEARSIZE != 0
    }

    /// Returns `true` if `DDSD_DEPTH` flag is available.
    pub fn ddsd_depth(&self) -> bool {
        self.flags & DDS_HEADER_FLAG_DDSD_DEPTH != 0
    }
}

pub struct HeaderDxt10 {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

pub struct PixelFormat {
    pub size: u32,
    pub flags: u32,
    pub four_cc: u32,
    pub rgb_bit_count: u32,
    pub r_bit_mask: u32,
    pub g_bit_mask: u32,
    pub b_bit_mask: u32,
    pub a_bit_mask: u32,
}

impl PixelFormat {
    /// Returns `true` if `DDPF_ALPHAPIXELS` flag is available.
    pub fn ddpf_alpha_pixels(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_ALPHA_PIXELS != 0
    }

    /// Returns `true` if `DDPF_ALPHA` flag is available.
    pub fn ddpf_alpha(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_ALPHA != 0
    }

    /// Returns `true` if `DDPF_FOURCC` flag is available.
    pub fn ddpf_four_cc(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_FOUR_CC != 0
    }

    /// Returns `true` if `DDPF_RGB` flag is available.
    pub fn ddpf_rgb(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_RGB != 0
    }

    /// Returns `true` if `DDPF_YUV` flag is available.
    pub fn ddpf_yuv(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_YUV != 0
    }

    /// Returns `true` if `DDPF_LUMINANCE` flag is available.
    pub fn ddpf_luminance(&self) -> bool {
        self.flags & DDS_PIXELFORMAT_FLAG_LUMINANCE != 0
    }
}

/// DirectDraw Surface.
pub struct DirectDrawSurface {
    pub magic_number: u32,
    pub header: Header,
    pub header_dxt10: Option<HeaderDxt10>,
    pub data: Uint8Array,
    pub raw: ArrayBuffer,
}

impl DirectDrawSurface {
    /// Parses a DirectDraw Surface file from raw data stored in [`ArrayBuffer`].
    pub fn from_array_buffer(raw: ArrayBuffer) -> Result<Self, Error> {
        // a dds file has at least 128 bytes
        if raw.byte_length() < 128 {
            return Err(Error::InvalidDirectDrawSurface);
        }

        let data_view = DataView::new(&raw, 0, raw.byte_length() as usize);

        // parses magic number
        let magic_number = Self::parse_magic_number(&data_view);
        if magic_number != DDS_MAGIC_NUMBER {
            return Err(Error::InvalidDirectDrawSurface);
        }

        // parses header
        let header = Self::parse_header(&data_view);
        if header.size != DDS_HEADER_SIZE
            || header.pixel_format.size != DDS_PIXELFORMAT_SIZE
            // those flags are required
            || !header.ddsd_caps()
            || !header.ddsd_height()
            || !header.ddsd_width()
            || !header.ddsd_pixel_format()
            || !header.pixel_format.ddpf_four_cc()
        {
            return Err(Error::InvalidDirectDrawSurface);
        }

        // parses header dxt10
        let (header_dxt10, data) = if header.pixel_format.four_cc == DDS_DXT10 {
            (
                Some(Self::parse_header_dxt10(&data_view)),
                Uint8Array::new_with_byte_offset(&raw, 148),
            )
        } else {
            (None, Uint8Array::new_with_byte_offset(&raw, 128))
        };

        Ok(Self {
            magic_number,
            header,
            header_dxt10,
            data,
            raw,
        })
    }

    /// Tries to create a [`Texture2D`] from this DirectDraw Surface.
    /// Returns `None` if unable to create a valid descriptor.
    pub fn texture_2d<SI, TI>(
        &self,
        dxt1_use_alpha: bool,
        use_srgb: bool,
        read_mipmaps: bool,
        sampler_params: SI,
        texture_params: TI,
    ) -> Option<Texture2DBase<TextureCompressedFormat>>
    where
        SI: IntoIterator<Item = SamplerParameter>,
        TI: IntoIterator<Item = TextureParameter>,
    {
        let compressed_format = match (self.header.pixel_format.four_cc, dxt1_use_alpha, use_srgb) {
            (DDS_DXT1, false, false) => Some(TextureCompressedFormat::RGB_S3TC_DXT1),
            (DDS_DXT1, true, false) => Some(TextureCompressedFormat::RGBA_S3TC_DXT1),
            (DDS_DXT1, false, true) => Some(TextureCompressedFormat::SRGB_S3TC_DXT1),
            (DDS_DXT1, true, true) => Some(TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT1),
            (DDS_DXT3, _, false) => Some(TextureCompressedFormat::RGBA_S3TC_DXT3),
            (DDS_DXT3, _, true) => Some(TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT3),
            (DDS_DXT5, _, false) => Some(TextureCompressedFormat::RGBA_S3TC_DXT5),
            (DDS_DXT5, _, true) => Some(TextureCompressedFormat::SRGB_ALPHA_S3TC_DXT5),
            (_, _, _) => None,
        };

        match compressed_format {
            Some(compressed_format) => {
                let base_width = self.header.width as usize;
                let base_height = self.header.height as usize;
                let levels = self.header.mipmap_count as usize;

                let mut builder = Builder::new(base_width, base_height, compressed_format)
                    .set_max_mipmap_level(levels - 1)
                    .set_texture_parameters(texture_params)
                    .set_sampler_parameters(sampler_params);

                if read_mipmaps && self.header.ddsd_mipmap_count() {
                    // reads mipmaps
                    let mut offset = 128usize;
                    for level in 0..levels {
                        let width = (base_width >> level).max(1);
                        let height = (base_height >> level).max(1);
                        let byte_length =
                            compressed_format.byte_length(width as usize, height as usize);
                        let data = Uint8Array::new_with_byte_offset_and_length(
                            &self.raw,
                            offset as u32,
                            byte_length as u32,
                        );
                        builder = builder.tex_image(
                            TextureSourceCompressed::Uint8Array {
                                width,
                                height,
                                compressed_format,
                                data,
                                src_offset: 0,
                                src_length_override: None,
                            },
                            level,
                        );
                        offset += byte_length;
                    }
                } else {
                    let data = Uint8Array::new_with_byte_offset_and_length(
                        &self.raw,
                        128,
                        compressed_format
                            .byte_length(self.header.width as usize, self.header.height as usize)
                            as u32,
                    );
                    builder = builder.set_base_source(TextureSourceCompressed::Uint8Array {
                        width: self.header.width as usize,
                        height: self.header.height as usize,
                        compressed_format,
                        data,
                        src_offset: 0,
                        src_length_override: None,
                    });
                };

                Some(builder.build())
            }
            None => None,
        }
    }

    fn parse_magic_number(data_view: &DataView) -> u32 {
        data_view.get_uint32_endian(0, true)
    }

    fn parse_header(data_view: &DataView) -> Header {
        let size = data_view.get_uint32_endian(4, true);
        let flags = data_view.get_uint32_endian(8, true);
        let height = data_view.get_uint32_endian(12, true);
        let width = data_view.get_uint32_endian(16, true);
        let pitch_or_linear_size = data_view.get_uint32_endian(20, true);
        let depth = data_view.get_uint32_endian(24, true);
        let mipmap_count = data_view.get_uint32_endian(28, true);
        let reserved1 = [
            data_view.get_uint32_endian(32, true),
            data_view.get_uint32_endian(36, true),
            data_view.get_uint32_endian(40, true),
            data_view.get_uint32_endian(44, true),
            data_view.get_uint32_endian(48, true),
            data_view.get_uint32_endian(52, true),
            data_view.get_uint32_endian(56, true),
            data_view.get_uint32_endian(60, true),
            data_view.get_uint32_endian(64, true),
            data_view.get_uint32_endian(68, true),
            data_view.get_uint32_endian(72, true),
        ];
        let pixel_format = Self::parse_pixel_format(data_view);
        let caps = data_view.get_uint32_endian(108, true);
        let caps2 = data_view.get_uint32_endian(112, true);
        let caps3 = data_view.get_uint32_endian(116, true);
        let caps4 = data_view.get_uint32_endian(120, true);
        let reserved2 = data_view.get_uint32_endian(124, true);

        Header {
            size,
            flags,
            height,
            width,
            pitch_or_linear_size,
            depth,
            mipmap_count,
            reserved1,
            pixel_format,
            caps,
            caps2,
            caps3,
            caps4,
            reserved2,
        }
    }

    fn parse_pixel_format(data_view: &DataView) -> PixelFormat {
        let size = data_view.get_uint32_endian(76, true);
        let flags = data_view.get_uint32_endian(80, true);
        let four_cc = data_view.get_uint32_endian(84, true);
        let rgb_bit_count = data_view.get_uint32_endian(88, true);
        let r_bit_mask = data_view.get_uint32_endian(92, true);
        let g_bit_mask = data_view.get_uint32_endian(96, true);
        let b_bit_mask = data_view.get_uint32_endian(100, true);
        let a_bit_mask = data_view.get_uint32_endian(104, true);

        PixelFormat {
            size,
            flags,
            four_cc,
            rgb_bit_count,
            r_bit_mask,
            g_bit_mask,
            b_bit_mask,
            a_bit_mask,
        }
    }

    fn parse_header_dxt10(data_view: &DataView) -> HeaderDxt10 {
        let dxgi_format = data_view.get_uint32_endian(128, true);
        let resource_dimension = data_view.get_uint32_endian(132, true);
        let misc_flag = data_view.get_uint32_endian(136, true);
        let array_size = data_view.get_uint32_endian(140, true);
        let misc_flags2 = data_view.get_uint32_endian(144, true);

        HeaderDxt10 {
            dxgi_format,
            resource_dimension,
            misc_flag,
            array_size,
            misc_flags2,
        }
    }
}

/// An texture loader loads texture from DirectDraw Surface file.
pub struct DirectDrawSurfaceLoader {
    url: String,
    status: *mut LoaderStatus,
    notifier: Share<Notifier<LoaderStatus>>,
    dds: *mut Option<DirectDrawSurface>,
    error: *mut Option<Error>,

    dxt1_use_alpha: bool,
    use_srgb: bool,
    read_mipmaps: bool,
    sampler_params: Vec<SamplerParameter>,
    texture_params: Vec<TextureParameter>,

    promise: *mut Option<Promise>,
    promise_resolve: *mut Option<Closure<dyn FnMut(JsValue)>>,
    promise_reject: *mut Option<Closure<dyn FnMut(JsValue)>>,
}

impl Drop for DirectDrawSurfaceLoader {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.status));
            drop(Box::from_raw(self.dds));
            drop(Box::from_raw(self.error));
            drop(Box::from_raw(self.promise));
            drop(Box::from_raw(self.promise_resolve));
            drop(Box::from_raw(self.promise_reject));
        }
    }
}

impl DirectDrawSurfaceLoader {
    /// Constructs a new dds loader.
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            status: Box::leak(Box::new(LoaderStatus::Unload)),
            notifier: Rc::new(RefCell::new(Notifier::new())),
            dds: Box::leak(Box::new(None)),
            error: Box::leak(Box::new(None)),

            dxt1_use_alpha: true,
            use_srgb: true,
            read_mipmaps: true,
            sampler_params: Vec::new(),
            texture_params: Vec::new(),

            promise: Box::leak(Box::new(None)),
            promise_resolve: Box::leak(Box::new(None)),
            promise_reject: Box::leak(Box::new(None)),
        }
    }

    /// Constructs a new dds loader with parameters.
    pub fn with_params<S, SI, TI>(
        url: S,
        dxt1_use_alpha: bool,
        use_srgb: bool,
        read_mipmaps: bool,
        sampler_params: SI,
        texture_params: TI,
    ) -> Self
    where
        S: Into<String>,
        SI: IntoIterator<Item = SamplerParameter>,
        TI: IntoIterator<Item = TextureParameter>,
    {
        Self {
            url: url.into(),
            status: Box::leak(Box::new(LoaderStatus::Unload)),
            notifier: Rc::new(RefCell::new(Notifier::new())),
            dds: Box::leak(Box::new(None)),
            error: Box::leak(Box::new(None)),

            dxt1_use_alpha,
            use_srgb,
            read_mipmaps,
            sampler_params: sampler_params.into_iter().collect(),
            texture_params: texture_params.into_iter().collect(),

            promise: Box::leak(Box::new(None)),
            promise_resolve: Box::leak(Box::new(None)),
            promise_reject: Box::leak(Box::new(None)),
        }
    }

    async fn fetch_buffer(url: String) -> Result<JsValue, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");

        let request = Request::new_with_str_and_init(&url, &opts)?;
        let response = JsFuture::from(window().fetch_with_request(&request))
            .await?
            .dyn_into::<Response>()
            .unwrap();

        let array_buffer = JsFuture::from(response.array_buffer()?).await?;

        Ok(array_buffer)
    }

    /// Starts loading image.
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub fn load(&self) {
        unsafe {
            if LoaderStatus::Unload != *self.status {
                return;
            }

            let status = self.status;
            let dds = self.dds;
            let error = self.error;

            let notifier = Rc::downgrade(&self.notifier);
            *self.promise_resolve = Some(Closure::new(move |array_buffer: JsValue| {
                match DirectDrawSurface::from_array_buffer(
                    array_buffer.dyn_into::<ArrayBuffer>().unwrap(),
                ) {
                    Ok(parsed) => {
                        (*status) = LoaderStatus::Loaded;
                        (*dds) = Some(parsed);

                        if let Some(notifier) = notifier.upgrade() {
                            notifier.borrow_mut().notify(&*status);
                        }
                    }
                    Err(err) => {
                        (*status) = LoaderStatus::Errored;
                        (*error) = Some(err);

                        if let Some(notifier) = notifier.upgrade() {
                            notifier.borrow_mut().notify(&*status);
                        }
                    }
                }
            }));

            let notifier = Rc::downgrade(&self.notifier);
            *self.promise_reject = Some(Closure::new(move |err: JsValue| {
                (*status) = LoaderStatus::Errored;
                (*error) = Some(Error::JsError(err.dyn_into::<js_sys::Error>().unwrap()));

                if let Some(notifier) = notifier.upgrade() {
                    notifier.borrow_mut().notify(&*status);
                }
            }));

            (*self.status) = LoaderStatus::Loading;

            let promise =
                wasm_bindgen_futures::future_to_promise(Self::fetch_buffer(self.url.to_string()));
            let promise = promise
                .then(&(*self.promise_resolve).as_ref().unwrap())
                .catch(&(*self.promise_reject).as_ref().unwrap());
            (*self.promise) = Some(promise);
        }
    }

    /// Starts loading image and puts it into a [`Promise`].
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub fn load_promise(&self) -> Promise {
        unsafe {
            self.load();
            (*self.promise).clone().unwrap()
        }
    }

    /// Starts loading image and asynchronous awaiting.
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub async fn load_async(&self) -> Result<&DirectDrawSurface, Error> {
        unsafe {
            self.load();
            match wasm_bindgen_futures::JsFuture::from((*self.promise).clone().unwrap()).await {
                Ok(_) => Ok((*self.dds).as_ref().unwrap()),
                Err(_) => Err((*self.error).clone().unwrap()),
            }
        }
    }

    /// Returns [`DirectDrawSurface`] regardless whether successfully loaded or not.
    pub fn dds(&self) -> Option<&DirectDrawSurface> {
        unsafe { (*self.dds).as_ref() }
    }

    /// Returns [`DirectDrawSurface`] if successfully loaded.
    pub fn loaded_dds(&self) -> Option<&DirectDrawSurface> {
        unsafe {
            match &*self.status {
                LoaderStatus::Unload | LoaderStatus::Loading | LoaderStatus::Errored => None,
                LoaderStatus::Loaded => (*self.dds).as_ref(),
            }
        }
    }

    /// Returns image source url.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Loader<Texture2D> for DirectDrawSurfaceLoader {
    type Failure = Error;

    fn status(&self) -> LoaderStatus {
        unsafe { *self.status }
    }

    fn load(&mut self) {
        Self::load(&self);
    }

    fn loaded(&self) -> Result<Texture2D, Error> {
        unsafe {
            if let Some(err) = &*self.error {
                return Err(err.clone());
            }

            let dds = (*self.dds).as_ref().unwrap();
            let texture = dds
                .texture_2d(
                    self.dxt1_use_alpha,
                    self.use_srgb,
                    self.read_mipmaps,
                    self.sampler_params.clone(),
                    self.texture_params.clone(),
                )
                .unwrap();

            Ok(Texture2D::Compressed(texture))
        }
    }

    fn notifier(&self) -> &Share<Notifier<LoaderStatus>> {
        &self.notifier
    }
}
