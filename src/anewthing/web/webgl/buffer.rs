use std::{
    cell::RefCell,
    fmt::Debug,
    ops::{Bound, Deref, DerefMut, Range, RangeBounds, RangeFrom},
    rc::Rc,
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
    buffering::{BufferData, Buffering, BufferingDropped},
    channel::{Channel, Event, Handler},
};

use super::error::Error;

/// WebGl buffer create options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebGlBufferCreateOptions {
    /// Buffer usage.
    pub usage: WebGlBufferUsage,
}

impl Default for WebGlBufferCreateOptions {
    fn default() -> Self {
        Self {
            usage: WebGlBufferUsage::StaticDraw,
        }
    }
}

/// A wrapped [`Buffering`] with [`WebGlBufferCreateOptions`].
#[derive(Debug, Clone, Default)]
pub struct WebGlBuffering {
    pub buffering: Buffering,
    /// Create options of a buffer.
    /// This field only works once, changing this does not influence anything.
    pub create_options: WebGlBufferCreateOptions,
}

impl WebGlBuffering {
    /// Constructs a new WebGl buffering container.
    pub fn new(buffering: Buffering, options: WebGlBufferCreateOptions) -> Self {
        Self {
            buffering,
            create_options: options,
        }
    }

    /// Constructs a new WebGl buffering container with default [`WebGlBufferOptions`].
    pub fn with_default_options(buffering: Buffering) -> Self {
        Self {
            buffering,
            create_options: WebGlBufferCreateOptions::default(),
        }
    }
}

impl Deref for WebGlBuffering {
    type Target = Buffering;

    fn deref(&self) -> &Self::Target {
        &self.buffering
    }
}

impl DerefMut for WebGlBuffering {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffering
    }
}

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlBufferTarget {
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
#[derive(Clone)]
pub enum WebGlBufferData<'a> {
    Binary {
        data: &'a [u8],
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

impl<'a> WebGlBufferData<'a> {
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
        let data_element_length = self.data_element_length();
        match self.element_range() {
            Some(range) => match range {
                WebGlBufferDataRange::Range(range) => {
                    if range.start > data_element_length {
                        0
                    } else if range.end > data_element_length {
                        data_element_length - range.start
                    } else {
                        range.len()
                    }
                }
                WebGlBufferDataRange::RangeFrom(range) => {
                    if range.start > data_element_length {
                        0
                    } else {
                        data_element_length - range.start
                    }
                }
            },
            None => data_element_length,
        }
    }

    /// Returns the length of elements of the whole data ignoring element range.
    /// For [`TypedArray`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray),
    /// element length is the size of elements, not bytes.
    pub fn data_element_length(&self) -> usize {
        match self {
            Self::Binary { data, .. } => data.len(),
            Self::ArrayBuffer { data } => data.bytes_length() as usize,
            Self::DataView { data, .. } => data.bytes_length(),
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
        }
    }

    /// Returns the byte length of the buffer data.
    pub fn bytes_length(&self) -> usize {
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

    fn upload(
        &self,
        gl: &WebGl2RenderingContext,
        target: WebGlBufferTarget,
        dst_bytes_offset: usize,
    ) {
        let target = target.to_gl_enum();
        let dst_bytes_offset = dst_bytes_offset as i32;
        let src_element_offset = self.element_offset() as u32;
        let src_element_length = self.element_length() as u32;

        match self {
            WebGlBufferData::Binary { data, .. } => gl
                .buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                    target,
                    dst_bytes_offset,
                    &data,
                    src_element_offset,
                    src_element_length,
                ),
            WebGlBufferData::ArrayBuffer { data } => {
                gl.buffer_sub_data_with_i32_and_array_buffer(target, dst_bytes_offset, &data)
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
                    dst_bytes_offset,
                    &data,
                    src_element_offset,
                    src_element_length,
                );
            }
        }
    }
}

impl<'a> BufferData for WebGlBufferData<'a> {
    fn bytes_length(&self) -> usize {
        self.bytes_length()
    }

    fn as_webgl_buffer_data(&self) -> Option<WebGlBufferData> {
        Some(self.clone())
    }
}

#[derive(Clone)]
pub struct WebGlBufferItem {
    gl_buffer: WebGlBuffer,
    bytes_length: Rc<RefCell<usize>>,
    usage: WebGlBufferUsage,
}

impl WebGlBufferItem {
    /// Returns native [`WebGlBuffer`].
    pub fn gl_buffer(&self) -> &WebGlBuffer {
        &self.gl_buffer
    }

    /// Returns byte length of the buffer.
    pub fn bytes_length(&self) -> usize {
        *self.bytes_length.borrow()
    }

    /// Returns [`WebGlBufferUsage`].
    pub fn usage(&self) -> WebGlBufferUsage {
        self.usage
    }

    /// Normalizes a [`RangeBounds`] to a [`Range<usize>`].
    /// Returns [`None`] if start and end bounds of [`RangeBounds`] are both unbounded.
    pub fn normalize_bytes_range<R>(&self, range: R) -> Range<usize>
    where
        R: RangeBounds<usize>,
    {
        match (range.start_bound(), range.end_bound()) {
            (Bound::Included(s), Bound::Included(e)) => *s..*e + 1,
            (Bound::Included(s), Bound::Excluded(e)) => *s..*e,
            (Bound::Included(s), Bound::Unbounded) => *s..self.bytes_length(),
            (Bound::Unbounded, Bound::Included(e)) => 0..*e + 1,
            (Bound::Unbounded, Bound::Excluded(e)) => 0..*e,
            (Bound::Unbounded, Bound::Unbounded) => 0..self.bytes_length(),
            (Bound::Excluded(_), _) => unreachable!(),
        }
    }
}

pub struct WebGlBufferManager {
    id: Uuid,
    gl: WebGl2RenderingContext,
    channel: Channel,
    buffers: Rc<RefCell<HashMap<Uuid, WebGlBufferItem>>>,
}

impl WebGlBufferManager {
    /// Constructs a new buffer manager.
    pub fn new(gl: WebGl2RenderingContext, channel: Channel) -> Self {
        let buffers = Rc::new(RefCell::new(HashMap::new()));
        channel.on(BufferingDroppedHandler::new(Rc::clone(&buffers)));

        Self {
            id: Uuid::new_v4(),
            gl,
            channel,
            buffers,
        }
    }

    /// Returns buffer manager id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Wraps a native [`WebGlBuffer`] into an managed [`WebGlBuffering`], returning [`WebGlBuffering`] and [`WebGlBufferItem`].
    /// Developer should stop using the native buffer after wrapping it.
    ///
    /// Developer should provides a correct [`WebGlBufferUsage`] of the buffer,
    /// nor the manager may fetch it from WebGl context.
    pub fn wrap_gl_buffer(
        &mut self,
        gl_buffer: WebGlBuffer,
        usage: Option<WebGlBufferUsage>,
    ) -> Result<(WebGlBuffering, WebGlBufferItem), Error> {
        let buffering = Buffering::new();
        buffering.set_managed(self.id, self.channel.clone());

        let usage = match usage {
            Some(usage) => usage,
            None => {
                self.gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(&gl_buffer),
                );
                let usage = self
                    .gl
                    .get_buffer_parameter(
                        WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                        WebGl2RenderingContext::BUFFER_USAGE,
                    )
                    .as_f64()
                    .unwrap() as u32;
                let usage = WebGlBufferUsage::from_gl_enum(usage).unwrap();
                self.gl
                    .bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);

                usage
            }
        };
        let buffer_item = WebGlBufferItem {
            gl_buffer,
            bytes_length: Rc::new(RefCell::new(0)),
            usage,
        };
        self.buffers
            .borrow_mut()
            .insert_unique_unchecked(*buffering.id(), buffer_item.clone());

        let buffering = WebGlBuffering::new(buffering, WebGlBufferCreateOptions { usage });
        Ok((buffering, buffer_item))
    }

    /// Manages a [`WebGlBuffering`] and syncs its queueing [`BufferData`] into WebGl context.
    pub fn sync_buffering(&mut self, buffering: &WebGlBuffering) -> Result<WebGlBufferItem, Error> {
        self.verify_manager(buffering)?;

        let mut buffers = self.buffers.borrow_mut();
        let buffer_item = match buffers.entry(*buffering.id()) {
            Entry::Occupied(entry) => {
                let buffer_item = entry.into_mut();
                let WebGlBufferItem {
                    bytes_length,
                    gl_buffer,
                    usage,
                } = buffer_item;
                let mut bytes_length = bytes_length.borrow_mut();

                // creates a new buffer with new byte length,
                // then copies data from old buffer to new buffer
                if buffering.bytes_length() > *bytes_length {
                    let new_gl_buffer =
                        self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                    self.gl.bind_buffer(
                        WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
                        Some(&new_gl_buffer),
                    );
                    self.gl.buffer_data_with_i32(
                        WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
                        buffering.bytes_length() as i32,
                        usage.to_gl_enum(),
                    );
                    self.gl.bind_buffer(
                        WebGlBufferTarget::CopyReadBuffer.to_gl_enum(),
                        Some(gl_buffer),
                    );
                    self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                        WebGlBufferTarget::CopyReadBuffer.to_gl_enum(),
                        WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(),
                        0,
                        0,
                        *bytes_length as i32,
                    );
                    self.gl
                        .bind_buffer(WebGlBufferTarget::CopyWriteBuffer.to_gl_enum(), None);
                    self.gl
                        .bind_buffer(WebGlBufferTarget::CopyReadBuffer.to_gl_enum(), None);

                    *gl_buffer = new_gl_buffer;
                    *bytes_length = buffering.bytes_length();
                }

                self.gl
                    .bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), Some(gl_buffer));
                for item in buffering.queue().drain() {
                    let Some(data) = item.data.as_webgl_buffer_data() else {
                        return Err(Error::BufferDataUnsupported);
                    };
                    data.upload(
                        &self.gl,
                        WebGlBufferTarget::ArrayBuffer,
                        item.dst_bytes_offset,
                    );
                }
                drop(bytes_length);

                buffer_item
            }
            Entry::Vacant(entry) => {
                let usage = buffering.create_options.usage;
                let bytes_length = buffering.bytes_length();

                let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                self.gl.bind_buffer(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    Some(&gl_buffer),
                );
                self.gl.buffer_data_with_i32(
                    WebGlBufferTarget::ArrayBuffer.to_gl_enum(),
                    bytes_length as i32,
                    usage.to_gl_enum(),
                );
                for item in buffering.queue().drain() {
                    let Some(data) = item.data.as_webgl_buffer_data() else {
                        return Err(Error::BufferDataUnsupported);
                    };
                    data.upload(
                        &self.gl,
                        WebGlBufferTarget::ArrayBuffer,
                        item.dst_bytes_offset,
                    );
                }

                let buffer_item = WebGlBufferItem {
                    bytes_length: Rc::new(RefCell::new(bytes_length)),
                    gl_buffer: gl_buffer.clone(),
                    usage,
                };
                buffering.set_managed(self.id, self.channel.clone());

                entry.insert(buffer_item)
            }
        };

        self.gl
            .bind_buffer(WebGlBufferTarget::ArrayBuffer.to_gl_enum(), None);

        Ok(buffer_item.clone())
    }

    fn verify_manager(&self, buffering: &WebGlBuffering) -> Result<(), Error> {
        if let Some(manager_id) = buffering.manager_id() {
            if manager_id != self.id {
                return Err(Error::BufferManagedByOtherManager);
            }
        }

        Ok(())
    }
}

impl Drop for WebGlBufferManager {
    fn drop(&mut self) {
        self.channel
            .off::<BufferingDropped, BufferingDroppedHandler>();
    }
}

/// A handler removes [`WebGlBufferItem`] from manager when a [`Buffer`] dropped.
/// This handler only removes items from [`WebGlBufferManager::buffers`], without unbinding them from WebGL context.
struct BufferingDroppedHandler {
    buffers: Rc<RefCell<HashMap<Uuid, WebGlBufferItem>>>,
}

impl BufferingDroppedHandler {
    fn new(buffers: Rc<RefCell<HashMap<Uuid, WebGlBufferItem>>>) -> Self {
        Self { buffers }
    }
}

impl Handler<BufferingDropped> for BufferingDroppedHandler {
    fn handle(&mut self, evt: &mut Event<'_, BufferingDropped>) {
        self.buffers.borrow_mut().remove(evt.id());
    }
}
