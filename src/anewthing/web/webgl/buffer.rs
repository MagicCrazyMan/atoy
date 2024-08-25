use std::{
    borrow::Cow,
    ops::{Range, RangeFrom},
    rc::{Rc, Weak},
};

use hashbrown::{HashMap, HashSet};
use js_sys::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};
use proc::GlEnum;
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use crate::anewthing::channel::Channel;

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BufferTarget {
    ArrayBuffer,
    ElementArrayBuffer,
    CopyReadBuffer,
    CopyWriteBuffer,
    TransformFeedbackBuffer,
    UniformBuffer,
    PixelPackBuffer,
    PixelUnpackBuffer,
}

/// Available buffer usages mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum BufferUsage {
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
pub enum BufferDataRange {
    Range(Range<usize>),
    RangeFrom(RangeFrom<usize>),
}

/// Buffer data.
#[derive(Debug, Clone)]
pub enum BufferData {
    Binary {
        data: Cow<'static, [u8]>,
        element_range: Option<BufferDataRange>,
    },
    ArrayBuffer {
        data: ArrayBuffer,
    },
    DataView {
        data: DataView,
        element_range: Option<BufferDataRange>,
    },
    Int8Array {
        data: Int8Array,
        element_range: Option<BufferDataRange>,
    },
    Uint8Array {
        data: Uint8Array,
        element_range: Option<BufferDataRange>,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        element_range: Option<BufferDataRange>,
    },
    Int16Array {
        data: Int16Array,
        element_range: Option<BufferDataRange>,
    },
    Uint16Array {
        data: Uint16Array,
        element_range: Option<BufferDataRange>,
    },
    Int32Array {
        data: Int32Array,
        element_range: Option<BufferDataRange>,
    },
    Uint32Array {
        data: Uint32Array,
        element_range: Option<BufferDataRange>,
    },
    Float32Array {
        data: Float32Array,
        element_range: Option<BufferDataRange>,
    },
    Float64Array {
        data: Float64Array,
        element_range: Option<BufferDataRange>,
    },
    BigInt64Array {
        data: BigInt64Array,
        element_range: Option<BufferDataRange>,
    },
    BigUint64Array {
        data: BigUint64Array,
        element_range: Option<BufferDataRange>,
    },
}

impl BufferData {
    /// Returns the range of elements.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element offset is the offset of elements, not bytes.
    pub fn element_range(&self) -> Option<BufferDataRange> {
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

    /// Returns the element size.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element length is the size of elements, not bytes.
    pub fn element_size(&self) -> usize {
        match self.element_range() {
            Some(_) => todo!(),
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

    /// Returns the bytes size of the buffer data.
    pub fn bytes_size(&self) -> usize {
        let element_size = self.element_size();
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
}

impl crate::anewthing::buffer::BufferData for BufferData {
    fn bytes_size(&self) -> usize {
        self.bytes_size()
    }
}

struct Buffer {
    size: usize,
    native: WebGlBuffer,
}

pub struct WebGlBufferManager {
    gl: WebGl2RenderingContext,
    channel: Channel,
    buffers: HashMap<Uuid, ()>,
}

impl WebGlBufferManager {
    /// Constructs a new buffer manager.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        Self {
            gl,
            channel,
            buffers: HashMap::new(),
        }
    }
}
