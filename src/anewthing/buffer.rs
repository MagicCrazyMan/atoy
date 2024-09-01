use std::{ops::Range, vec::Drain};

use uuid::Uuid;

use super::channel::Channel;

pub trait BufferData {
    /// Returns the byte length of the buffer data.
    fn byte_length(&self) -> usize;
}

pub(crate) struct BufferItem<T: ?Sized> {
    data: Box<T>,
    dst_byte_offset: usize,
}

impl<T: ?Sized> BufferItem<T> {
    pub(crate) fn data(&self) -> &T {
        &self.data
    }

    pub(crate) fn dst_byte_offset(&self) -> usize {
        self.dst_byte_offset
    }
}

pub struct Buffer<T: ?Sized> {
    id: Uuid,
    queue: Vec<BufferItem<T>>,
    queue_byte_range: Option<Range<usize>>,
    byte_length: usize,
    growable: bool,

    synced: Option<(Channel, Uuid)>,
}

impl<T: ?Sized> Buffer<T> {
    /// Constructs a new growable buffer with initial zero size.
    pub fn new() -> Self {
        Self::with_size(0, true)
    }

    /// Constructs a new buffer with specified initial byte length.
    /// If `growable` is `false`, any data that exceeds the size will be ignored.
    pub fn with_size(byte_length: usize, growable: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Vec::new(),
            queue_byte_range: None,
            byte_length,
            growable,
            synced: None,
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

    /// Returns `true` if the buffer if growable.
    pub fn growable(&self) -> bool {
        self.growable
    }

    /// Returns `true` if the buffer if is already synced by a manager.
    pub fn synced(&self) -> bool {
        self.synced.is_some()
    }

    /// Drains and returns all [`BufferItem`] in queue.
    pub(crate) fn drain_queue(&mut self) -> Drain<'_, BufferItem<T>> {
        self.queue_byte_range = None;
        self.queue.drain(..)
    }

    /// Returns the synced id.
    pub(crate) fn synced_id(&self) -> Option<&Uuid> {
        self.synced.as_ref().map(|synced| &synced.1)
    }

    /// Sets this buffer is already synced by a manager.
    pub(crate) fn set_synced(&mut self, channel: Channel, id: Uuid) {
        self.synced = Some((channel, id));
    }
}

impl<T> Buffer<T>
where
    T: BufferData,
{
    /// Buffers data into the buffer.
    pub fn buffer(&mut self, data: T) {
        self.buffer_with_offset(data, 0)
    }

    /// Buffers data into the buffer with byte offset indicating where to start replacing data.
    pub fn buffer_with_offset(&mut self, data: T, dst_byte_offset: usize) {
        let byte_length = dst_byte_offset + data.byte_length();
        let byte_range = dst_byte_offset..byte_length;
        let item = BufferItem {
            data: Box::new(data),
            dst_byte_offset,
        };

        if byte_length > self.byte_length && self.growable {
            self.byte_length = byte_length;
        }

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
}

impl<T: ?Sized> Drop for Buffer<T> {
    fn drop(&mut self) {
        if let Some((channel, _)) = &self.synced {
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
    pub(crate) fn buffer_id(&self) -> &Uuid {
        &self.id
    }
}
