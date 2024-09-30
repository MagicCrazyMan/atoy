use std::{
    cell::{RefCell, RefMut},
    fmt::Debug,
    ops::Range,
    rc::Rc,
    vec::Drain,
};

use tokio::sync::broadcast::{self, Receiver, Sender};
use uuid::Uuid;

pub trait BufferData {
    /// Returns the byte length of the buffer data.
    fn bytes_length(&self) -> usize;

    /// Converts the buffer data into a [`WebGlBufferData`](super::web::webgl::buffer::WebGlBufferData).
    #[cfg(feature = "webgl")]
    fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
        None
    }
}

pub(crate) struct BufferingItem {
    /// Buffer data.
    pub(crate) data: Box<dyn BufferData>,
    /// Offset in bytes specifying where data start to write to.
    pub(crate) dst_bytes_offset: usize,
}

pub(crate) struct BufferingQueue {
    queue: Vec<BufferingItem>,
    covered_bytes_range: Option<Range<usize>>,
}

impl BufferingQueue {
    fn new() -> Self {
        Self {
            queue: Vec::new(),
            covered_bytes_range: None,
        }
    }

    pub(crate) fn drain(&mut self) -> Drain<'_, BufferingItem> {
        self.covered_bytes_range = None;
        self.queue.drain(..)
    }
}

#[derive(Clone)]
pub struct Buffering {
    id: Uuid,
    queue: Rc<RefCell<BufferingQueue>>,
    bytes_length: Rc<RefCell<usize>>,

    channel: Sender<BufferingMessage>,
}

impl Buffering {
    /// Constructs a new buffering container.
    pub fn new() -> Self {
        Self::with_bytes_length(0)
    }

    /// Constructs a new buffering container with byte length.
    pub fn with_bytes_length(bytes_length: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Rc::new(RefCell::new(BufferingQueue::new())),
            bytes_length: Rc::new(RefCell::new(bytes_length)),

            channel: broadcast::channel(5).0,
        }
    }

    /// Returns id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns total byte length.
    pub fn bytes_length(&self) -> usize {
        *self.bytes_length.borrow()
    }

    /// Returns the inner buffer queue.
    pub(crate) fn queue(&self) -> RefMut<'_, BufferingQueue> {
        self.queue.borrow_mut()
    }

    /// Pushes buffer data into the buffering.
    pub fn push<T>(&self, data: T)
    where
        T: BufferData + 'static,
    {
        self.push_with_bytes_offset(data, 0)
    }

    /// Pushes buffer data into the buffering with byte offset indicating where to start replacing data.
    pub fn push_with_bytes_offset<T>(&self, data: T, dst_bytes_offset: usize)
    where
        T: BufferData + 'static,
    {
        let mut queue = self.queue.borrow_mut();
        let BufferingQueue {
            queue,
            covered_bytes_range,
        } = &mut *queue;
        let bytes_length = dst_bytes_offset + data.bytes_length();
        let bytes_range = dst_bytes_offset..bytes_length;
        self.bytes_length
            .replace_with(|length| (*length).max(bytes_length));

        let item = BufferingItem {
            data: Box::new(data),
            dst_bytes_offset,
        };

        match covered_bytes_range {
            Some(covered_bytes_range) => {
                // overrides queue if new byte range fully covers the range of current queue
                if bytes_range.start <= covered_bytes_range.start
                    && bytes_range.end >= covered_bytes_range.end
                {
                    *covered_bytes_range = bytes_range;
                    queue.clear();
                    queue.push(item);
                } else {
                    *covered_bytes_range = bytes_range.start.min(covered_bytes_range.start)
                        ..bytes_range.end.max(covered_bytes_range.end);
                    queue.push(item);
                }
            }
            None => {
                *covered_bytes_range = Some(bytes_range);
                queue.push(item);
            }
        }
    }

    /// Returns a message receiver associated with this buffering.
    pub fn receiver(&self) -> Receiver<BufferingMessage> {
        self.channel.subscribe()
    }
}

impl Default for Buffering {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Buffering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffering")
            .field("id", self.id())
            .field("bytes_length", &self.bytes_length())
            .finish()
    }
}

impl Drop for Buffering {
    fn drop(&mut self) {
        let _ = self.channel.send(BufferingMessage::Dropped);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BufferingMessage {
    Dropped,
}

#[cfg(feature = "web")]
impl BufferData for js_sys::ArrayBuffer {
    fn bytes_length(&self) -> usize {
        self.byte_length() as usize
    }

    #[cfg(feature = "webgl")]
    fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
        Some(super::web::webgl::buffer::WebGlBufferData::ArrayBuffer { data: self.clone() })
    }
}

macro_rules! web_typed_arrays {
    ($(($buffer: ident, $length: ident, $size: expr))+) => {
        $(
            #[cfg(feature = "web")]
            impl BufferData for js_sys::$buffer {
                fn bytes_length(&self) -> usize {
                    self.byte_length() as usize
                }

                #[cfg(feature = "webgl")]
                fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
                    Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: self.clone(), element_range: None })
                }
            }


            #[cfg(feature = "webgl")]
            impl BufferData for (js_sys::$buffer, super::web::webgl::buffer::WebGlBufferDataRange) {
                fn bytes_length(&self) -> usize {
                    let data_element_length = self.0.$length() as usize;
                    let element_length = match &self.1 {
                        super::web::webgl::buffer::WebGlBufferDataRange::Range(range) => {
                            if range.start > data_element_length {
                                0
                            } else if range.end > data_element_length {
                                data_element_length - range.start
                            } else {
                                range.len()
                            }
                        },
                        super::web::webgl::buffer::WebGlBufferDataRange::RangeFrom(range) => {
                            if range.start > data_element_length {
                                0
                            } else {
                                data_element_length - range.start
                            }
                        },
                    };

                    element_length * $size
                }

                fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
                    Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: self.0.clone(), element_range: Some(self.1.clone()) })
                }
            }
        )+
    };
}
web_typed_arrays! {
    (DataView, bytes_length, 1)
    (Int8Array, length, 1)
    (Uint8Array, length, 1)
    (Uint8ClampedArray, length, 1)
    (Int16Array, length, 2)
    (Uint16Array, length, 2)
    (Int32Array, length, 4)
    (Uint32Array, length, 4)
    (Float32Array, length, 4)
    (Float64Array, length, 8)
    (BigInt64Array, length, 8)
    (BigUint64Array, length, 8)
}
