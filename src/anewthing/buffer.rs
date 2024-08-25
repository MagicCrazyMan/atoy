use hashbrown::HashSet;
use uuid::Uuid;

use super::channel::Channel;

pub trait BufferData {
    /// Returns the bytes size of the buffer data.
    fn bytes_size(&self) -> usize;
}

pub struct Buffer<T: ?Sized> {
    id: Uuid,
    queue: Vec<Box<T>>,
    size: usize,
    growable: bool,
    channels: HashSet<Channel>,
}

impl<T: ?Sized> Buffer<T> {
    /// Constructs a new growable buffer with initial zero size.
    pub fn new() -> Self {
        Self::with_size(0, true)
    }

    /// Constructs a new buffer with specified size.
    /// If `growable` is `false`, any data that exceeds the size will be ignored.
    pub fn with_size(size: usize, growable: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue: Vec::new(),
            size,
            growable,
            channels: HashSet::new(),
        }
    }

    /// Returns the id of the buffer.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns size of the buffer.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns `true` if the buffer if growable.
    pub fn growable(&self) -> bool {
        self.growable
    }

    /// Returns the queue of the buffer.
    pub(crate) fn queue(&mut self) -> &mut Vec<Box<T>> {
        &mut self.queue
    }

    /// Registers a channel to the buffer.
    pub(crate) fn register_channel(&mut self, channel: Channel) {
        self.channels.insert(channel);
    }
}

impl<T> Buffer<T>
where
    T: BufferData,
{
    /// Buffers data into the buffer.
    pub fn buffer_data(&mut self, data: T) {
        self.size += data.bytes_size();
        self.queue.push(Box::new(data));
    }
}

impl<T: ?Sized> Drop for Buffer<T> {
    fn drop(&mut self) {
        for channel in &self.channels {
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
