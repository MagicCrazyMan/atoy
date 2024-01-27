use std::fmt::Display;

use web_sys::js_sys::{ArrayBuffer, DataView, Uint8Array};

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

pub const DDS_MAGIC_NUMBER: u32 = 0x20534444;

/// DirectDraw Surface.
pub struct DirectDrawSurface {
    /// File magic number, always equals `DDS ` aka (0x20534444).
    pub magic_number: u32,
    pub header: Header,
    pub header_dxt10: Option<HeaderDxt10>,
    pub data: Uint8Array,
}

impl DirectDrawSurface {
    /// Parse a DirectDraw Surface file from raw data stored in [`ArrayBuffer`].
    pub fn parse(raw: ArrayBuffer) -> Result<Self, Error> {
        let data_view = DataView::new(&raw, 0, raw.byte_length() as usize);

        // parses magic number
        let magic_number = Self::parse_magic_number(&data_view);
        if magic_number != DDS_MAGIC_NUMBER {
            return Err(Error::InvalidFile);
        }

        // parses header
        let header = Self::parse_header(&data_view);
        log::info!("{:x}", header.flags);

        todo!()
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
}

#[derive(Debug)]
pub enum Error {
    InvalidFile,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
