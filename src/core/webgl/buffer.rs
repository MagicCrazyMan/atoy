use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use hashbrown::HashMap;
use js_sys::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{conversion::ToGlEnum, error::Error};

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Buffer data.
#[derive(Debug, Clone)]
pub enum BufferData {
    ArrayBuffer {
        data: ArrayBuffer,
    },
    DataView {
        data: DataView,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int8Array {
        data: Int8Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint8Array {
        data: Uint8Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int16Array {
        data: Int16Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint16Array {
        data: Uint16Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int32Array {
        data: Int32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint32Array {
        data: Uint32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Float32Array {
        data: Float32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Float64Array {
        data: Float64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    BigInt64Array {
        data: BigInt64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    BigUint64Array {
        data: BigUint64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
}

impl BufferData {
    fn as_array_buffer(&self) -> ArrayBuffer {
        match self {
            BufferData::ArrayBuffer { data } => data.clone(),
            BufferData::DataView { data, .. } => data.buffer(),
            BufferData::Int8Array { data, .. } => data.buffer(),
            BufferData::Uint8Array { data, .. } => data.buffer(),
            BufferData::Uint8ClampedArray { data, .. } => data.buffer(),
            BufferData::Int16Array { data, .. } => data.buffer(),
            BufferData::Uint16Array { data, .. } => data.buffer(),
            BufferData::Int32Array { data, .. } => data.buffer(),
            BufferData::Uint32Array { data, .. } => data.buffer(),
            BufferData::Float32Array { data, .. } => data.buffer(),
            BufferData::Float64Array { data, .. } => data.buffer(),
            BufferData::BigInt64Array { data, .. } => data.buffer(),
            BufferData::BigUint64Array { data, .. } => data.buffer(),
        }
    }

    fn byte_per_element(&self) -> usize {
        match self {
            BufferData::ArrayBuffer { .. } => 1,
            BufferData::DataView { .. } => 1,
            BufferData::Int8Array { .. } => 1,
            BufferData::Uint8Array { .. } => 1,
            BufferData::Uint8ClampedArray { .. } => 1,
            BufferData::Int16Array { .. } => 2,
            BufferData::Uint16Array { .. } => 2,
            BufferData::Int32Array { .. } => 4,
            BufferData::Uint32Array { .. } => 4,
            BufferData::Float32Array { .. } => 4,
            BufferData::Float64Array { .. } => 8,
            BufferData::BigInt64Array { .. } => 8,
            BufferData::BigUint64Array { .. } => 8,
        }
    }

    fn byte_length(&self) -> usize {
        let byte_per_element = self.byte_per_element();
        let (native_byte_length, src_element_offset, src_element_length) = match self {
            BufferData::ArrayBuffer { data } => (data.byte_length() as usize, None, None),
            BufferData::DataView {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Int8Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Uint8Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Uint8ClampedArray {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Int16Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Uint16Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Int32Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Uint32Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Float32Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::Float64Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::BigInt64Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
            BufferData::BigUint64Array {
                data,
                src_element_offset,
                src_element_length,
            } => (
                data.byte_length() as usize,
                *src_element_offset,
                *src_element_length,
            ),
        };

        match (src_element_offset, src_element_length) {
            (None, None) => native_byte_length,
            (None, Some(src_element_length)) => {
                let byte_length = src_element_length * byte_per_element;
                native_byte_length.min(byte_length)
            }
            (Some(src_element_offset), None) => {
                let offset_byte_length = src_element_offset * byte_per_element;
                native_byte_length.saturating_sub(offset_byte_length)
            }
            (Some(src_element_offset), Some(src_element_length)) => {
                let byte_length = src_element_length * byte_per_element;
                let offset_byte_length = src_element_offset * byte_per_element;
                native_byte_length
                    .saturating_sub(offset_byte_length)
                    .min(byte_length)
            }
        }
    }

    fn upload(&self, gl: &WebGl2RenderingContext, target: BufferTarget, dst_byte_offset: usize) {
        match self {
            BufferData::DataView { .. }
            | BufferData::Int8Array { .. }
            | BufferData::Uint8Array { .. }
            | BufferData::Uint8ClampedArray { .. }
            | BufferData::Int16Array { .. }
            | BufferData::Uint16Array { .. }
            | BufferData::Int32Array { .. }
            | BufferData::Uint32Array { .. }
            | BufferData::Float32Array { .. }
            | BufferData::Float64Array { .. }
            | BufferData::BigInt64Array { .. }
            | BufferData::BigUint64Array { .. } => {
                let (data, src_element_offset, src_element_length) = match self {
                    BufferData::DataView {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int8Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint8Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint8ClampedArray {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int16Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint16Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Float32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Float64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::BigInt64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::BigUint64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    _ => unreachable!(),
                };
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    &data,
                    src_element_offset.unwrap_or(0) as u32,
                    src_element_length.unwrap_or(0) as u32,
                );
            }
            BufferData::ArrayBuffer { data } => {
                gl.buffer_sub_data_with_i32_and_array_buffer(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    &data,
                );
            }
        };
    }
}

#[derive(Debug)]
struct QueueItem {
    data: BufferData,
    dst_byte_offset: usize,
}

impl QueueItem {
    fn new(data: BufferData, dst_byte_offset: usize) -> Self {
        Self {
            data,
            dst_byte_offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    id: Uuid,
    capacity: usize,
    usage: BufferUsage,

    queue_size: Rc<RefCell<usize>>,
    queue: Rc<RefCell<Vec<QueueItem>>>, // usize is dst_byte_offset

    registered: Rc<RefCell<Option<BufferRegistered>>>,
}

impl Buffer {
    pub fn new(capacity: usize, usage: BufferUsage) -> Self {
        Self {
            id: Uuid::new_v4(),
            capacity,
            usage,

            queue_size: Rc::new(RefCell::new(0)),
            queue: Rc::new(RefCell::new(Vec::new())),

            registered: Rc::new(RefCell::new(None)),
        }
    }

    pub fn with_buffer_data(buffer_data: BufferData, usage: BufferUsage) -> Self {
        Self {
            id: Uuid::new_v4(),
            capacity: buffer_data.byte_length(),
            usage,

            queue_size: Rc::new(RefCell::new(buffer_data.byte_length())),
            queue: Rc::new(RefCell::new(vec![QueueItem::new(buffer_data, 0)])),

            registered: Rc::new(RefCell::new(None)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn gl_buffer(&self) -> Option<WebGlBuffer> {
        self.registered
            .borrow()
            .as_ref()
            .map(|registered| registered.gl_buffer.clone())
    }

    pub fn write(&self, data: BufferData) {
        self.write_with_offset(data, 0)
    }

    pub fn write_with_offset(&self, data: BufferData, dst_byte_offset: usize) {
        let size = data.byte_length() + dst_byte_offset;
        let mut current_size = self.queue_size.borrow_mut();
        *current_size = current_size.max(size);
        self.queue
            .borrow_mut()
            .push(QueueItem::new(data, dst_byte_offset));
    }

    pub fn flush(&self) -> Result<(), Error> {
        if let Some(registered) = self.registered.borrow().as_ref() {
            registered.upload()
        } else {
            Ok(())
        }
    }

    pub fn read(&self) -> ArrayBuffer {
        let downloaded = self
            .registered
            .borrow()
            .as_ref()
            .map(|registered| registered.download())
            .unwrap_or(ArrayBuffer::new(self.capacity as u32));
        let typed_array = Uint8Array::new(&downloaded);

        for QueueItem {
            data,
            dst_byte_offset,
        } in self.queue.borrow().iter()
        {
            typed_array.set(
                &Uint8Array::new(&data.as_array_buffer()),
                *dst_byte_offset as u32,
            );
        }

        downloaded
    }
}

/// [BufferTarget] to use when upload or download data to or from WebGlBuffer.
const BUFFER_TARGET: BufferTarget = BufferTarget::ArrayBuffer;

#[derive(Debug, Clone)]
struct BufferRegistered {
    gl: WebGl2RenderingContext,
    gl_buffer: WebGlBuffer,

    repo_id: Uuid,
    repo_items: Weak<RefCell<HashMap<Uuid, Weak<RefCell<Option<Self>>>>>>,
    repo_used_memory: Weak<RefCell<usize>>,

    buffer_id: Uuid,
    buffer_capacity: usize,
    buffer_queue: Weak<RefCell<Vec<QueueItem>>>,
    buffer_queue_size: Weak<RefCell<usize>>,

    restore_when_drop: bool,
}

impl Drop for BufferRegistered {
    fn drop(&mut self) {
        if let Some(repo_items) = self.repo_items.upgrade() {
            repo_items.borrow_mut().remove(&self.buffer_id);
        }
        if let Some(repo_used_memory) = self.repo_used_memory.upgrade() {
            *repo_used_memory.borrow_mut() -= self.buffer_capacity;
        }

        if self.restore_when_drop {
            let (Some(buffer_queue), Some(buffer_queue_size)) = (
                self.buffer_queue.upgrade(),
                self.buffer_queue_size.upgrade(),
            ) else {
                return;
            };

            let buffer_data = BufferData::ArrayBuffer {
                data: self.download(),
            };
            buffer_queue
                .borrow_mut()
                .insert(0, QueueItem::new(buffer_data, 0));
            *buffer_queue_size.borrow_mut() = self.buffer_capacity;
        }
    }
}

impl BufferRegistered {
    fn upload(&self) -> Result<(), Error> {
        let (Some(buffer_queue), Some(buffer_queue_size)) = (
            self.buffer_queue.upgrade(),
            self.buffer_queue_size.upgrade(),
        ) else {
            return Err(Error::BufferUnexpectedDropped);
        };

        if *buffer_queue_size.borrow() > self.buffer_capacity {
            return Err(Error::DataOutOfBufferCapacity {
                data_size: *buffer_queue_size.borrow(),
                capacity: self.buffer_capacity,
            });
        }

        self.gl
            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&self.gl_buffer));

        let mut queue = buffer_queue.borrow_mut();
        let queue = queue.drain(..);
        for QueueItem {
            data,
            dst_byte_offset,
        } in queue
        {
            data.upload(&self.gl, BUFFER_TARGET, dst_byte_offset);
        }
        *buffer_queue_size.borrow_mut() = 0;

        self.gl.bind_buffer(BUFFER_TARGET.gl_enum(), None);

        Ok(())
    }

    fn download(&self) -> ArrayBuffer {
        let data = Uint8Array::new_with_length(self.buffer_capacity as u32);
        self.gl
            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&self.gl_buffer));
        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            BUFFER_TARGET.gl_enum(),
            0,
            &data,
        );
        self.gl.bind_buffer(BUFFER_TARGET.gl_enum(), None);

        data.buffer()
    }
}

#[derive(Debug)]
pub struct BufferRepository {
    id: Uuid,
    gl: WebGl2RenderingContext,
    items: Rc<RefCell<HashMap<Uuid, Weak<RefCell<Option<BufferRegistered>>>>>>,
    used_memory: Rc<RefCell<usize>>,
}

impl BufferRepository {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            items: Rc::new(RefCell::new(HashMap::new())),
            used_memory: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn used_memory(&self) -> usize {
        *self.used_memory.borrow()
    }

    pub fn register(&self, buffer: &Buffer) -> Result<(), Error> {
        if let Some(registered) = &*buffer.registered.borrow() {
            if registered.repo_id != self.id {
                return Err(Error::RegisterBufferToMultipleRepositoryUnsupported);
            } else {
                registered.upload()?;
                return Ok(());
            }
        }

        let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.gl
            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&gl_buffer));
        self.gl.buffer_data_with_i32(
            BUFFER_TARGET.gl_enum(),
            buffer.capacity as i32,
            buffer.usage.gl_enum(),
        );
        *self.used_memory.borrow_mut() += buffer.capacity;

        let registered = BufferRegistered {
            gl: self.gl.clone(),
            repo_id: self.id,
            repo_items: Rc::downgrade(&self.items),
            repo_used_memory: Rc::downgrade(&self.used_memory),

            buffer_id: *buffer.id(),
            buffer_queue: Rc::downgrade(&buffer.queue),
            buffer_queue_size: Rc::downgrade(&buffer.queue_size),

            gl_buffer: gl_buffer.clone(),
            buffer_capacity: buffer.capacity,

            restore_when_drop: false,
        };
        registered.upload()?;

        *buffer.registered.borrow_mut() = Some(registered);
        self.items
            .borrow_mut()
            .insert(*buffer.id(), Rc::downgrade(&buffer.registered));

        Ok(())
    }
}
