use std::{
    borrow::Cow,
    ops::{Range, RangeFrom},
};

use hashbrown::{hash_map::Entry, HashMap};
use js_sys::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};
use proc::GlEnum;
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::anewthing::{
    buffer::{self, Buffer, BufferData},
    channel::Channel,
};

use super::error::Error;

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
#[repr(usize)]
pub enum WebGlBufferTarget {
    ArrayBuffer = 0,
    ElementArrayBuffer = 1,
    CopyReadBuffer = 2,
    CopyWriteBuffer = 3,
    TransformFeedbackBuffer = 4,
    UniformBuffer = 5,
    PixelPackBuffer = 6,
    PixelUnpackBuffer = 7,
}

/// Available buffer usages mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlBufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
    StaticRead,
    DynamicRead,
    StreamRead,
    StaticCopy,
    DynamicCopy,
    StreamCopy,
}

/// Buffer data range.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebGlBufferDataRange {
    Range(Range<usize>),
    RangeFrom(RangeFrom<usize>),
}

/// Buffer data.
#[derive(Debug, Clone)]
pub enum WebGlBufferData {
    Binary {
        data: Cow<'static, [u8]>,
        element_range: Option<WebGlBufferDataRange>,
    },
    ArrayBuffer {
        data: ArrayBuffer,
    },
    DataView {
        data: DataView,
        element_range: Option<WebGlBufferDataRange>,
    },
    Int8Array {
        data: Int8Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Uint8Array {
        data: Uint8Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        element_range: Option<WebGlBufferDataRange>,
    },
    Int16Array {
        data: Int16Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Uint16Array {
        data: Uint16Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Int32Array {
        data: Int32Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Uint32Array {
        data: Uint32Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Float32Array {
        data: Float32Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    Float64Array {
        data: Float64Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    BigInt64Array {
        data: BigInt64Array,
        element_range: Option<WebGlBufferDataRange>,
    },
    BigUint64Array {
        data: BigUint64Array,
        element_range: Option<WebGlBufferDataRange>,
    },
}

impl WebGlBufferData {
    /// Returns the range of elements.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element range is the offset of elements, not bytes.
    pub fn element_range(&self) -> Option<WebGlBufferDataRange> {
        match self {
            Self::Binary { element_range, .. }
            | Self::DataView { element_range, .. }
            | Self::Int8Array { element_range, .. }
            | Self::Uint8Array { element_range, .. }
            | Self::Uint8ClampedArray { element_range, .. }
            | Self::Int16Array { element_range, .. }
            | Self::Uint16Array { element_range, .. }
            | Self::Int32Array { element_range, .. }
            | Self::Uint32Array { element_range, .. }
            | Self::Float32Array { element_range, .. }
            | Self::Float64Array { element_range, .. }
            | Self::BigInt64Array { element_range, .. }
            | Self::BigUint64Array { element_range, .. } => element_range.clone(),
            Self::ArrayBuffer { .. } => None,
        }
    }

    /// Returns the offset of elements.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element offset is the offset of elements, not bytes.
    pub fn element_offset(&self) -> usize {
        match self.element_range() {
            Some(range) => match range {
                WebGlBufferDataRange::Range(range) => range.start,
                WebGlBufferDataRange::RangeFrom(range) => range.start,
            },
            None => 0,
        }
    }

    /// Returns the length of elements.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element length is the size of elements, not bytes.
    pub fn element_length(&self) -> usize {
        match self.element_range() {
            Some(range) => match range {
                WebGlBufferDataRange::Range(range) => range.len(),
                WebGlBufferDataRange::RangeFrom(_) => todo!(),
            },
            None => match self {
                Self::Binary { data, .. } => data.len(),
                Self::ArrayBuffer { data } => data.byte_length() as usize,
                Self::DataView { data, .. } => data.byte_length(),
                Self::Int8Array { data, .. } => data.length() as usize,
                Self::Uint8Array { data, .. } => data.length() as usize,
                Self::Uint8ClampedArray { data, .. } => data.length() as usize,
                Self::Int16Array { data, .. } => data.length() as usize,
                Self::Uint16Array { data, .. } => data.length() as usize,
                Self::Int32Array { data, .. } => data.length() as usize,
                Self::Uint32Array { data, .. } => data.length() as usize,
                Self::Float32Array { data, .. } => data.length() as usize,
                Self::Float64Array { data, .. } => data.length() as usize,
                Self::BigInt64Array { data, .. } => data.length() as usize,
                Self::BigUint64Array { data, .. } => data.length() as usize,
            },
        }
    }

    /// Returns the byte length of the buffer data.
    pub fn byte_length(&self) -> usize {
        let element_size = self.element_length();
        match self {
            Self::Binary { .. }
            | Self::ArrayBuffer { .. }
            | Self::DataView { .. }
            | Self::Int8Array { .. }
            | Self::Uint8Array { .. }
            | Self::Uint8ClampedArray { .. } => element_size,
            Self::Int16Array { .. } | Self::Uint16Array { .. } => element_size * 2,
            Self::Int32Array { .. } | Self::Uint32Array { .. } | Self::Float32Array { .. } => {
                element_size * 4
            }
            Self::Float64Array { .. }
            | Self::BigInt64Array { .. }
            | Self::BigUint64Array { .. } => element_size * 8,
        }
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: WebGlBufferTarget,
        dst_byte_offset: usize,
    ) {
        let target = target.to_gl_enum();
        let dst_byte_offset = dst_byte_offset as i32;
        let src_element_offset = self.element_offset() as u32;
        let src_element_length = self.element_length() as u32;

        match self {
            WebGlBufferData::Binary { data, .. } => gl
                .buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                    target,
                    dst_byte_offset,
                    &data,
                    src_element_offset,
                    src_element_length,
                ),
            WebGlBufferData::ArrayBuffer { data } => {
                gl.buffer_sub_data_with_i32_and_array_buffer(target, dst_byte_offset, &data)
            }
            WebGlBufferData::DataView { .. }
            | WebGlBufferData::Int8Array { .. }
            | WebGlBufferData::Uint8Array { .. }
            | WebGlBufferData::Uint8ClampedArray { .. }
            | WebGlBufferData::Int16Array { .. }
            | WebGlBufferData::Uint16Array { .. }
            | WebGlBufferData::Int32Array { .. }
            | WebGlBufferData::Uint32Array { .. }
            | WebGlBufferData::Float32Array { .. }
            | WebGlBufferData::Float64Array { .. }
            | WebGlBufferData::BigInt64Array { .. }
            | WebGlBufferData::BigUint64Array { .. } => {
                let data: &Object = match self {
                    WebGlBufferData::DataView { data, .. } => data.as_ref(),
                    WebGlBufferData::Int8Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Uint8Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Uint8ClampedArray { data, .. } => data.as_ref(),
                    WebGlBufferData::Int16Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Uint16Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Int32Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Uint32Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Float32Array { data, .. } => data.as_ref(),
                    WebGlBufferData::Float64Array { data, .. } => data.as_ref(),
                    WebGlBufferData::BigInt64Array { data, .. } => data.as_ref(),
                    WebGlBufferData::BigUint64Array { data, .. } => data.as_ref(),
                    _ => unreachable!(),
                };
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target,
                    dst_byte_offset,
                    &data,
                    src_element_offset,
                    src_element_length,
                );
            }
        }
    }
}

impl BufferData for WebGlBufferData {
    fn byte_length(&self) -> usize {
        self.byte_length()
    }
}

enum WebGlBufferOwner {
    Me(Uuid),
    Outside,
}

struct WebGlBufferItem {
    byte_length: usize,
    gl_buffer: WebGlBuffer,
    usage: WebGlBufferUsage,
}

pub struct WebGlBufferManager {
    gl: WebGl2RenderingContext,
    channel: Channel,
    buffers: HashMap<Uuid, WebGlBufferItem>,

    /// of all buffer targets
    bound_targets: [Option<Uuid>; 8],
}

impl WebGlBufferManager {
    /// Constructs a new buffer manager.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            gl,
            channel,
            buffers: HashMap::new(),

            bound_targets: [None; 8],
        }
    }

    pub fn bind_buffer(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        target: WebGlBufferTarget,
    ) -> Result<(), Error> {
        self.sync_buffer(buffer, target)?;

        if self.bound_targets[target as usize] == Some(*buffer.id()) {
            return Ok(());
        }

        todo!()
    }

    fn sync_buffer(
        &mut self,
        buffer: &mut Buffer<WebGlBufferData>,
        target: WebGlBufferTarget,
    ) -> Result<(), Error> {
        match self.buffers.entry(*buffer.id()) {
            Entry::Occupied(entry) => {
                todo!()
            }
            Entry::Vacant(entry) => {
                let usage = if buffer.growable() {
                    WebGlBufferUsage::DynamicDraw
                } else {
                    WebGlBufferUsage::StaticDraw
                };
                let byte_length = buffer.byte_length();

                let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                self.gl.bind_buffer(target.to_gl_enum(), Some(&gl_buffer));
                self.gl.buffer_data_with_i32(
                    target.to_gl_enum(),
                    byte_length as i32,
                    usage.to_gl_enum(),
                );
                for item in buffer.drain_queue() {
                    item.data()
                        .buffer_sub_data(&self.gl, target, item.dst_byte_offset());
                }

                self.bound_targets[target as usize] = Some(*buffer.id());
                let buffer_item = WebGlBufferItem {
                    byte_length,
                    gl_buffer,
                    usage,
                };
                entry.insert(buffer_item);

                Ok(())
            }
        }
    }
}
