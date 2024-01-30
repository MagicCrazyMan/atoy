use web_sys::js_sys::{ArrayBuffer, DataView, Uint8Array};

use crate::render::webgl::texture::{
    texture2d::{Builder, Texture2D},
    TextureCompressedFormat, TextureDescriptor, TextureParameter, TextureSourceCompressed,
};

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
    /// Parse a DirectDraw Surface file from raw data stored in [`ArrayBuffer`].
    pub fn parse(raw: ArrayBuffer) -> Option<Self> {
        // a dds file has at least 128 bytes
        if raw.byte_length() < 128 {
            return None;
        }

        let data_view = DataView::new(&raw, 0, raw.byte_length() as usize);

        // parses magic number
        let magic_number = Self::parse_magic_number(&data_view);
        if magic_number != DDS_MAGIC_NUMBER {
            return None;
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
            return None;
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

        Some(Self {
            magic_number,
            header,
            header_dxt10,
            data,
            raw,
        })
    }

    /// Tries to create a [`TextureDescriptor`] from this DirectDraw Surface.
    /// Returns `None` if unable to create a valid descriptor.
    pub fn texture_descriptor<I>(
        &self,
        dxt1_use_alpha: bool,
        use_srgb: bool,
        read_mipmaps: bool,
        tex_params: I,
    ) -> Option<TextureDescriptor<Texture2D<TextureCompressedFormat>>>
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        let internal_format = match (self.header.pixel_format.four_cc, dxt1_use_alpha, use_srgb) {
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

        match internal_format {
            Some(internal_format) => {
                let base_width = self.header.width as usize;
                let base_height = self.header.height as usize;
                let mut builder = Builder::new(base_width, base_height, internal_format);
                builder = builder.set_texture_parameters(tex_params);

                if read_mipmaps && self.header.ddsd_mipmap_count() {
                    // reads mipmaps
                    let levels = self.header.mipmap_count as usize;
                    let mut offset = 128usize;
                    for level in 0..levels {
                        let width = (base_width >> level).max(1);
                        let height = (base_height >> level).max(1);
                        let bytes_length =
                            internal_format.bytes_length(width as usize, height as usize);
                        let data = Uint8Array::new_with_byte_offset_and_length(
                            &self.raw,
                            offset as u32,
                            bytes_length as u32,
                        );
                        builder = builder.tex_image(
                            TextureSourceCompressed::Uint8Array {
                                width,
                                height,
                                compressed_format: internal_format,
                                data,
                                src_offset: 0,
                                src_length_override: None,
                            },
                            level,
                        );
                        offset += bytes_length;
                    }
                } else {
                    let data = Uint8Array::new_with_byte_offset_and_length(
                        &self.raw,
                        128,
                        internal_format
                            .bytes_length(self.header.width as usize, self.header.height as usize)
                            as u32,
                    );
                    builder = builder.set_base_source(TextureSourceCompressed::Uint8Array {
                        width: self.header.width as usize,
                        height: self.header.height as usize,
                        compressed_format: internal_format,
                        data,
                        src_offset: 0,
                        src_length_override: None,
                    });
                };

                Some(TextureDescriptor::new(builder.build()))
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
