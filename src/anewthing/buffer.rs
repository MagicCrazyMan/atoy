use std::{ops::Range, vec::Drain};

use uuid::Uuid;

use super::channel::Channel;

pub trait BufferData {
    /// Returns the byte length of the buffer data.
    fn byte_length(&self) -> usize;

    /// Converts the buffer data into a [`WebGlBufferData`](super::web::webgl::buffer::WebGlBufferData).
    #[cfg(feature = "webgl")]
    fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
        None
    }
}

pub(crate) struct BufferItem {
    pub(crate) data: Box<dyn BufferData>,
    pub(crate) dst_byte_offset: usize,
}

pub struct Buffer {
    id: Uuid,
    queue: Vec<BufferItem>,
    queue_byte_range: Option<Range<usize>>,
    byte_length: usize,
    managed: Option<(Channel, Uuid)>,
    #[cfg(feature = "webgl")]
    webgl_options: super::web::webgl::buffer::WebGlBufferOptions,
}

impl Buffer {
    /// Constructs a new buffer with default options.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Vec::new(),
            queue_byte_range: None,
            byte_length: 0,
            managed: None,
            #[cfg(feature = "webgl")]
            webgl_options: super::web::webgl::buffer::WebGlBufferOptions::default(),
        }
    }

    /// Returns the id of the buffer.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns byte length of the buffer.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Drains and returns all [`BufferItem`] in queue.
    pub(crate) fn drain(&mut self) -> Drain<'_, BufferItem> {
        self.queue_byte_range = None;
        self.queue.drain(..)
    }

    /// Returns `true` if the buffer if is managed.
    pub fn is_managed(&self) -> bool {
        self.managed.is_some()
    }

    /// Returns the manager id.
    pub(crate) fn manager_id(&self) -> Option<&Uuid> {
        self.managed.as_ref().map(|synced| &synced.1)
    }

    /// Sets this buffer is already managed by a manager.
    pub(crate) fn set_managed(&mut self, channel: Channel, id: Uuid) {
        match self.managed.as_ref() {
            Some((c, d)) => {
                if c.id() != channel.id() || d != &id {
                    panic!("manage a buffer by multiple managers is prohibited");
                }
            }
            None => self.managed = Some((channel, id)),
        };
    }

    /// Pushes buffer data into the buffer.
    pub fn push<T>(&mut self, data: T)
    where
        T: BufferData + 'static,
    {
        self.push_with_offset(data, 0)
    }

    /// Pushes buffer data into the buffer with byte offset indicating where to start replacing data.
    pub fn push_with_offset<T>(&mut self, data: T, dst_byte_offset: usize)
    where
        T: BufferData + 'static,
    {
        let byte_length = dst_byte_offset + data.byte_length();
        let byte_range = dst_byte_offset..byte_length;
        self.byte_length = self.byte_length.max(byte_length);

        let item = BufferItem {
            data: Box::new(data),
            dst_byte_offset,
        };

        match &self.queue_byte_range {
            Some(queue_byte_range) => {
                // overrides queue if new byte range fully covers the range of current queue
                if byte_range.start <= queue_byte_range.start
                    && byte_range.end >= queue_byte_range.end
                {
                    self.queue_byte_range = Some(byte_range);
                    self.queue.clear();
                    self.queue.push(item);
                } else {
                    self.queue_byte_range = Some(
                        byte_range.start.min(queue_byte_range.start)
                            ..byte_range.end.max(queue_byte_range.end),
                    );
                    self.queue.push(item);
                }
            }
            None => {
                self.queue_byte_range = Some(byte_range);
                self.queue.push(item);
            }
        }
    }

    /// Returns webgl buffer options.
    #[cfg(feature = "webgl")]
    pub fn webgl_options(&self) -> &super::web::webgl::buffer::WebGlBufferOptions {
        &self.webgl_options
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if let Some((channel, _)) = &self.managed {
            channel.send(BufferDropped { id: self.id });
        }
    }
}

/// Events raised when a buffer is dropped.
pub(crate) struct BufferDropped {
    id: Uuid,
}

impl BufferDropped {
    /// Returns the id of the buffer.
    pub(crate) fn id(&self) -> &Uuid {
        &self.id
    }
}

pub struct BufferBuilder {
    byte_length: usize,
    #[cfg(feature = "webgl")]
    webgl_options: Option<super::web::webgl::buffer::WebGlBufferOptions>,
}

impl BufferBuilder {
    /// Constructs a new buffer builder with default options.
    pub fn new() -> Self {
        Self {
            byte_length: 0,
            #[cfg(feature = "webgl")]
            webgl_options: None,
        }
    }

    /// Sets initial byte length of the buffer
    pub fn set_byte_length(mut self, byte_length: usize) -> Self {
        self.byte_length = byte_length;
        self
    }

    /// Sets webgl buffer options.
    #[cfg(feature = "webgl")]
    pub fn set_webgl_options(
        mut self,
        options: super::web::webgl::buffer::WebGlBufferOptions,
    ) -> Self {
        self.webgl_options = Some(options);
        self
    }

    /// Builds buffer.
    pub fn build(self) -> Buffer {
        Buffer {
            id: Uuid::new_v4(),
            queue: Vec::new(),
            queue_byte_range: None,
            byte_length: self.byte_length,
            managed: None,
            #[cfg(feature = "webgl")]
            webgl_options: self.webgl_options.unwrap_or_default(),
        }
    }
}

#[cfg(feature = "web")]
impl BufferData for js_sys::ArrayBuffer {
    fn byte_length(&self) -> usize {
        self.byte_length() as usize
    }

    #[cfg(feature = "webgl")]
    fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
        Some(super::web::webgl::buffer::WebGlBufferData::ArrayBuffer { data: self })
    }
}

macro_rules! web_typed_arrays {
    ($(($buffer: ident, $length: ident, $size: expr))+) => {
        $(
            #[cfg(feature = "web")]
            impl BufferData for js_sys::$buffer {
                fn byte_length(&self) -> usize {
                    self.byte_length() as usize
                }

                #[cfg(feature = "webgl")]
                fn as_webgl_buffer_data(&self) -> Option<super::web::webgl::buffer::WebGlBufferData> {
                    Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: self, element_range: None })
                }
            }


            #[cfg(feature = "webgl")]
            impl BufferData for (js_sys::$buffer, super::web::webgl::buffer::WebGlBufferDataRange) {
                fn byte_length(&self) -> usize {
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
                    Some(super::web::webgl::buffer::WebGlBufferData::$buffer { data: &self.0, element_range: Some(self.1.clone()) })
                }
            }
        )+
    };
}
web_typed_arrays! {
    (DataView, byte_length, 1)
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
