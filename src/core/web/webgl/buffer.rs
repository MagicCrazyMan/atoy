use std::{
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    rc::{Rc, Weak},
};

use async_trait::async_trait;
use hashbrown::{HashMap, HashSet};
use js_sys::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};
use log::error;
use proc::GlEnum;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{client_wait::ClientWaitAsync, error::Error};

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

/// Buffer data.
#[derive(Debug, Clone)]
pub enum BufferData {
    Preallocation {
        size: usize,
    },
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
    fn byte_per_element(&self) -> usize {
        match self {
            BufferData::Preallocation { .. } => 1,
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
            BufferData::Preallocation { size } => (*size, None, None),
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
                    target.to_gl_enum(),
                    dst_byte_offset as i32,
                    &data,
                    src_element_offset.unwrap_or(0) as u32,
                    src_element_length.unwrap_or(0) as u32,
                );
            }
            BufferData::ArrayBuffer { data } => {
                gl.buffer_sub_data_with_i32_and_array_buffer(
                    target.to_gl_enum(),
                    dst_byte_offset as i32,
                    &data,
                );
            }
            BufferData::Preallocation { size } => {
                gl.buffer_data_with_i32(
                    target.to_gl_enum(),
                    *size as i32,
                    BufferUsage::StreamDraw.to_gl_enum(),
                );
            }
        };
    }
}

impl BufferSource for BufferData {
    fn load(&mut self) -> Result<BufferData, String> {
        Ok(self.clone())
    }
}

/// A buffer source returns an available [`BufferData`] immediately
/// and then uploads to WebGL context immediately.
pub trait BufferSource {
    /// Returns a [`BufferData`].
    fn load(&mut self) -> Result<BufferData, String>;
}

/// A buffer source retrieves data asynchronously
/// and then uploads it to WebGL context after that.
#[async_trait(?Send)]
pub trait BufferSourceAsync {
    /// Returns a [`BufferData`] or an error message.
    async fn load(&mut self) -> Result<BufferData, String>;
}

enum BufferSourceInner {
    Sync(Box<dyn BufferSource>),
    Async(Box<dyn BufferSourceAsync>),
}

impl Debug for BufferSourceInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(_) => f.debug_tuple("Sync").finish(),
            Self::Async(_) => f.debug_tuple("Async").finish(),
        }
    }
}

#[derive(Debug)]
pub(super) struct QueueItem {
    source: BufferSourceInner,
    dst_byte_offset: usize,
}

impl QueueItem {
    fn new(source: BufferSourceInner, dst_byte_offset: usize) -> Self {
        Self {
            source,
            dst_byte_offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub(super) id: Uuid,
    pub(super) usage: BufferUsage,
    pub(super) queue: Rc<RefCell<VecDeque<QueueItem>>>, // usize is dst_byte_offset

    pub(super) registered: Rc<RefCell<Option<BufferRegistered>>>,
}

impl Buffer {
    pub fn new(usage: BufferUsage) -> Self {
        Self {
            id: Uuid::new_v4(),
            usage,
            queue: Rc::new(RefCell::new(VecDeque::new())),

            registered: Rc::new(RefCell::new(None)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn size(&self) -> usize {
        self.registered
            .borrow()
            .as_ref()
            .map_or(0, |registered| registered.0.buffer_size)
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn flushing(&self) -> bool {
        self.registered
            .borrow()
            .as_ref()
            .map_or(false, |registered| {
                registered.0.buffer_async_upload.borrow().is_some()
            })
    }

    pub fn ready(&self) -> bool {
        self.registered.borrow().as_ref().is_some()
            && self.queue.borrow().is_empty()
            && !self.flushing()
    }

    pub fn write_source<S>(&self, source: S)
    where
        S: BufferSource + 'static,
    {
        self.write_source_with_offset(source, 0)
    }

    pub fn write_source_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSource + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSourceInner::Sync(Box::new(source)),
            dst_byte_offset,
        ));
    }

    pub fn write_async_source<S>(&self, source: S)
    where
        S: BufferSourceAsync + 'static,
    {
        self.write_async_source_with_offset(source, 0)
    }

    pub fn write_async_source_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSourceAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSourceInner::Async(Box::new(source)),
            dst_byte_offset,
        ));
    }

    pub fn gl_buffer(&self) -> Result<WebGlBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .map(|registered| registered.0.gl_buffer.clone())
            .ok_or(Error::BufferUnregistered)
    }

    pub fn read_to_array_buffer(&self) -> Result<ArrayBuffer, Error> {
        self.read_to_array_buffer_with_params(None)
    }

    pub fn read_to_array_buffer_with_params(
        &self,
        src_byte_offset: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .0
            .read_to_array_buffer(ReadBackKind::NewArrayBuffer, src_byte_offset)
    }

    pub fn read_to_new_array_buffer(&self, to: ArrayBuffer) -> Result<ArrayBuffer, Error> {
        self.read_to_new_array_buffer_with_params(to, None)
    }

    pub fn read_to_new_array_buffer_with_params(
        &self,
        to: ArrayBuffer,
        src_byte_offset: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .0
            .read_to_array_buffer(ReadBackKind::ToArrayBuffer(to), src_byte_offset)
    }

    pub async fn read_to_array_buffer_async(&self) -> Result<ArrayBuffer, Error> {
        self.read_to_array_buffer_with_params_async(None, None)
            .await
    }

    pub async fn read_to_array_buffer_with_params_async(
        &self,
        src_byte_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .0
            .read_to_array_buffer_async(ReadBackKind::NewArrayBuffer, src_byte_offset, max_retries)
            .await
    }

    pub async fn read_to_new_array_buffer_async(
        &self,
        to: ArrayBuffer,
    ) -> Result<ArrayBuffer, Error> {
        self.read_to_new_array_buffer_with_params_async(to, None, None)
            .await
    }

    pub async fn read_to_new_array_buffer_with_params_async(
        &self,
        to: ArrayBuffer,
        src_byte_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .0
            .read_to_array_buffer_async(
                ReadBackKind::ToArrayBuffer(to),
                src_byte_offset,
                max_retries,
            )
            .await
    }

    pub fn flush(&self, continue_when_failed: bool) -> Result<bool, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .0
            .flush(continue_when_failed)
    }

    pub async fn flush_async(&self, continue_when_failed: bool) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .0
            .flush_async(continue_when_failed)
            .await
    }

    pub fn bind(&self, target: BufferTarget) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .0
            .bind(target)
    }

    pub fn unbind(&self, target: BufferTarget) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .0
            .unbind(target);
        Ok(())
    }

    pub fn unbind_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .0
            .unbind_all();
        Ok(())
    }

    pub fn copy_to(&self, to: &Buffer) -> Result<(), Error> {
        self.copy_to_with_params(to, None, None, None)
    }

    pub fn copy_to_with_params(
        &self,
        to: &Buffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
    ) -> Result<(), Error> {
        let from = self.registered.borrow();
        let to = to.registered.borrow();
        let (from, to) = (
            from.as_ref().ok_or(Error::BufferUnregistered)?,
            to.as_ref().ok_or(Error::BufferUnregistered)?,
        );

        from.0
            .copy_to(&to.0.gl_buffer, read_offset, write_offset, size, None);

        Ok(())
    }
}

/// [BufferTarget] to use when upload or download data to or from WebGlBuffer.
const BUFFER_TARGET: BufferTarget = BufferTarget::ArrayBuffer;

enum ReadBackKind {
    NewArrayBuffer,
    ToArrayBuffer(ArrayBuffer),
}

impl ReadBackKind {
    fn into_array_buffer(self, size: usize) -> ArrayBuffer {
        match self {
            ReadBackKind::NewArrayBuffer => ArrayBuffer::new(size as u32),
            ReadBackKind::ToArrayBuffer(array_buffer) => array_buffer,
        }
    }
}

#[derive(Debug)]
pub(super) struct BufferRegistered(pub(super) BufferRegisteredUndrop);

impl Drop for BufferRegistered {
    fn drop(&mut self) {
        if self.0.restore_when_drop {
            let Some(buffer_queue) = self.0.buffer_queue.upgrade() else {
                return;
            };

            if let Ok(data) = self
                .0
                .read_to_array_buffer(ReadBackKind::NewArrayBuffer, None)
            {
                let buffer_data = BufferData::ArrayBuffer { data };
                buffer_queue.borrow_mut().insert(
                    0,
                    QueueItem::new(BufferSourceInner::Sync(Box::new(buffer_data)), 0),
                );
            } else {
                log::warn!("failed to download data from WebGlBuffer");
            }
        }

        // self.unbind_all();
        self.0.gl.delete_buffer(Some(&self.0.gl_buffer));
        self.0
            .reg_used_size
            .upgrade()
            .map(|used_size| *used_size.borrow_mut() -= self.0.buffer_size);
    }
}

#[derive(Debug, Clone)]
pub(super) struct BufferRegisteredUndrop {
    pub(super) gl: WebGl2RenderingContext,
    pub(super) gl_buffer: WebGlBuffer,
    pub(super) gl_bounds: HashSet<BufferTarget>,

    pub(super) reg_id: Uuid,
    pub(super) reg_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    pub(super) reg_used_size: Weak<RefCell<usize>>,

    pub(super) buffer_size: usize,
    pub(super) buffer_usage: BufferUsage,
    pub(super) buffer_queue: Weak<RefCell<VecDeque<QueueItem>>>,
    pub(super) buffer_async_upload:
        Rc<RefCell<Option<(Closure<dyn FnMut(JsValue)>, Closure<dyn FnMut(JsValue)>)>>>,

    pub(super) restore_when_drop: bool,
}

impl BufferRegisteredUndrop {
    fn bind(&mut self, target: BufferTarget) -> Result<(), Error> {
        if let Some(gl_buffer) = self.reg_bounds.borrow().get(&target) {
            if gl_buffer == &self.gl_buffer {
                return Ok(());
            } else {
                return Err(Error::BufferTargetOccupied(target));
            }
        }

        self.gl
            .bind_buffer(target.to_gl_enum(), Some(&self.gl_buffer));
        self.reg_bounds
            .borrow_mut()
            .insert_unique_unchecked(target, self.gl_buffer.clone());
        self.gl_bounds.insert(target);

        Ok(())
    }

    fn unbind(&mut self, target: BufferTarget) {
        if self.gl_bounds.remove(&target) {
            self.gl.bind_buffer(target.to_gl_enum(), None);
            self.reg_bounds.borrow_mut().remove(&target);
        }
    }

    fn unbind_all(&mut self) {
        for bound in self.gl_bounds.drain() {
            self.gl.bind_buffer(bound.to_gl_enum(), None);
            self.reg_bounds.borrow_mut().remove(&bound);
        }
    }

    fn enlarge(&mut self, new_size: usize) -> Result<(), Error> {
        // copies existing data to a temporary buffer
        let tmp_gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.copy_to(
            &tmp_gl_buffer,
            None,
            None,
            None,
            Some(BufferUsage::StreamCopy),
        );

        // enlarges buffer size
        self.gl
            .bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&self.gl_buffer));
        self.gl.buffer_data_with_i32(
            BUFFER_TARGET.to_gl_enum(),
            new_size as i32,
            self.buffer_usage.to_gl_enum(),
        );

        // copies data back from temporary buffer
        self.copy_from(&tmp_gl_buffer, None, None, Some(self.buffer_size));

        // updates buffer size and used memory
        if let Some(used_size) = self.reg_used_size.upgrade() {
            let mut used_size = used_size.borrow_mut();
            *used_size -= self.buffer_size;
            *used_size += new_size;
        }
        self.buffer_size = new_size;

        Ok(())
    }

    fn flush(&mut self, continue_when_failed: bool) -> Result<bool, Error> {
        // if there is an ongoing async upload, skips this flush
        if self.buffer_async_upload.borrow().is_some() {
            return Ok(false);
        }

        let Some(buffer_queue) = self.buffer_queue.upgrade() else {
            return Ok(true);
        };

        let mut queue = buffer_queue.borrow_mut();
        while let Some(item) = queue.pop_front() {
            let QueueItem {
                source,
                dst_byte_offset,
            } = item;

            match source {
                BufferSourceInner::Sync(mut source) => {
                    let data = match source.load() {
                        Ok(data) => data,
                        Err(msg) => {
                            if continue_when_failed {
                                error!("failed to load buffer source: {msg}");
                                continue;
                            } else {
                                return Err(Error::LoadBufferSourceFailure(Some(msg)));
                            }
                        }
                    };
                    let data_size = data.byte_length();

                    // if data size larger than the buffer size, expands the buffer size
                    if data_size > self.buffer_size {
                        self.enlarge(data_size)?; // enlarge will bind buffer to BUFFER_TARGET
                    } else {
                        self.gl
                            .bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&self.gl_buffer));
                    }

                    data.upload(&self.gl, BUFFER_TARGET, dst_byte_offset);

                    self.gl.bind_buffer(
                        BUFFER_TARGET.to_gl_enum(),
                        self.reg_bounds.borrow().get(&BUFFER_TARGET),
                    );
                }
                BufferSourceInner::Async(mut source) => {
                    let promise = future_to_promise(async move {
                        let data = source.load().await;
                        match data {
                            Ok(data) => {
                                let data_ptr = Box::leak(Box::new(data)) as *const _ as usize;
                                Ok(JsValue::from(data_ptr))
                            }
                            Err(msg) => {
                                let msg_ptr = Box::leak(Box::new(msg)) as *const _ as usize;
                                Err(JsValue::from(msg_ptr))
                            }
                        }
                    });

                    let mut me = self.clone();
                    let resolve = Closure::once(move |value: JsValue| unsafe {
                        let buffer_data =
                            Box::from_raw(value.as_f64().unwrap() as usize as *mut BufferData);
                        let Some(queue) = me.buffer_queue.upgrade() else {
                            return;
                        };

                        // adds buffer data as the first value to the queue and then continues uploading
                        queue.borrow_mut().push_front(QueueItem::new(
                            BufferSourceInner::Sync(buffer_data),
                            dst_byte_offset,
                        ));
                        me.buffer_async_upload.borrow_mut().as_mut().take();
                        let _ = me.flush(continue_when_failed);
                    });

                    let mut me = self.clone();
                    let reject = Closure::once(move |value: JsValue| unsafe {
                        // if reject, prints error message, sends error message to channel and skips this source
                        let msg = Box::from_raw(value.as_f64().unwrap() as usize as *mut String);
                        error!("failed to load async buffer source: {}", msg);

                        me.buffer_async_upload.borrow_mut().as_mut().take();
                        if continue_when_failed {
                            // continues uploading
                            let _ = me.flush(continue_when_failed);
                        }
                    });

                    *self.buffer_async_upload.borrow_mut() = Some((resolve, reject));
                    let _ = promise
                        .then(
                            self.buffer_async_upload
                                .borrow()
                                .as_ref()
                                .map(|(resolve, _)| resolve)
                                .unwrap(),
                        )
                        .catch(
                            self.buffer_async_upload
                                .borrow()
                                .as_ref()
                                .map(|(_, reject)| reject)
                                .unwrap(),
                        );

                    break;
                }
            }
        }

        Ok(self.buffer_async_upload.borrow().is_none())
    }

    async fn flush_async(&mut self, continue_when_failed: bool) -> Result<(), Error> {
        let Some(buffer_queue) = self.buffer_queue.upgrade() else {
            return Ok(());
        };

        let mut queue = buffer_queue.borrow_mut();
        if queue.is_empty() {
            return Ok(());
        }

        while let Some(item) = queue.pop_front() {
            let QueueItem {
                source,
                dst_byte_offset,
            } = item;

            let data = match source {
                BufferSourceInner::Sync(mut source) => source.load(),
                BufferSourceInner::Async(mut source) => source.load().await,
            };
            let data = match data {
                Ok(data) => data,
                Err(msg) => {
                    if continue_when_failed {
                        error!("failed to load buffer source: {msg}");
                        continue;
                    } else {
                        return Err(Error::LoadBufferSourceFailure(Some(msg)));
                    }
                }
            };
            let data_size = data.byte_length();

            // if data size larger than the buffer size, expands the buffer size
            if data_size > self.buffer_size {
                self.enlarge(data_size)?; // enlarge will bind buffer to BUFFER_TARGET
            } else {
                self.gl
                    .bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&self.gl_buffer));
            }

            data.upload(&self.gl, BUFFER_TARGET, dst_byte_offset);

            self.gl.bind_buffer(
                BUFFER_TARGET.to_gl_enum(),
                self.reg_bounds.borrow().get(&BUFFER_TARGET),
            );
        }

        Ok(())
    }

    fn read_to_array_buffer(
        &self,
        read_back_kind: ReadBackKind,
        src_byte_offset: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.copy_to(&tmp, None, None, None, Some(BufferUsage::StreamRead));

        let src_byte_offset = src_byte_offset.unwrap_or(0);
        let array_buffer = read_back_kind.into_array_buffer(self.buffer_size);
        self.gl.bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&tmp));
        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            BUFFER_TARGET.to_gl_enum(),
            src_byte_offset as i32,
            &Uint8Array::new(&array_buffer),
        );
        self.gl.bind_buffer(
            BUFFER_TARGET.to_gl_enum(),
            self.reg_bounds.borrow().get(&BUFFER_TARGET),
        );
        self.gl.delete_buffer(Some(&tmp));

        Ok(array_buffer)
    }

    async fn read_to_array_buffer_async(
        &self,
        read_back_kind: ReadBackKind,
        src_byte_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.copy_to(&tmp, None, None, None, Some(BufferUsage::StreamRead));

        ClientWaitAsync::new(self.gl.clone(), 0, 5, max_retries)
            .wait()
            .await?;

        let src_byte_offset = src_byte_offset.unwrap_or(0);
        let array_buffer = read_back_kind.into_array_buffer(self.buffer_size);
        self.gl.bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&tmp));
        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            BUFFER_TARGET.to_gl_enum(),
            src_byte_offset as i32,
            &Uint8Array::new(&array_buffer),
        );
        self.gl.bind_buffer(
            BUFFER_TARGET.to_gl_enum(),
            self.reg_bounds.borrow().get(&BUFFER_TARGET),
        );
        self.gl.delete_buffer(Some(&tmp));

        Ok(array_buffer)
    }

    fn copy_to(
        &self,
        to: &WebGlBuffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
        reallocate: Option<BufferUsage>,
    ) {
        let read_offset = read_offset.unwrap_or(0);
        let write_offset = write_offset.unwrap_or(0);
        let size = size.unwrap_or(self.buffer_size);

        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.to_gl_enum(),
            Some(&self.gl_buffer),
        );
        self.gl
            .bind_buffer(BufferTarget::CopyWriteBuffer.to_gl_enum(), Some(to));
        if let Some(usage) = reallocate {
            self.gl.buffer_data_with_i32(
                BufferTarget::CopyWriteBuffer.to_gl_enum(),
                size as i32,
                usage.to_gl_enum(),
            );
        }
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            BufferTarget::CopyReadBuffer.to_gl_enum(),
            BufferTarget::CopyWriteBuffer.to_gl_enum(),
            read_offset as i32,
            write_offset as i32,
            size as i32,
        );
        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.to_gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyReadBuffer),
        );
        self.gl.bind_buffer(
            BufferTarget::CopyWriteBuffer.to_gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyWriteBuffer),
        );
    }

    fn copy_from(
        &self,
        from: &WebGlBuffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
    ) {
        let read_offset = read_offset.unwrap_or(0);
        let write_offset = write_offset.unwrap_or(0);
        let size = size.unwrap_or(self.buffer_size);

        self.gl
            .bind_buffer(BufferTarget::CopyReadBuffer.to_gl_enum(), Some(from));
        self.gl.bind_buffer(
            BufferTarget::CopyWriteBuffer.to_gl_enum(),
            Some(&self.gl_buffer),
        );
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            BufferTarget::CopyReadBuffer.to_gl_enum(),
            BufferTarget::CopyWriteBuffer.to_gl_enum(),
            read_offset as i32,
            write_offset as i32,
            size as i32,
        );
        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.to_gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyReadBuffer),
        );
        self.gl.bind_buffer(
            BufferTarget::CopyWriteBuffer.to_gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyWriteBuffer),
        );
    }
}

#[derive(Debug, Clone)]
pub struct BufferRegistry {
    pub(super) id: Uuid,
    pub(super) gl: WebGl2RenderingContext,
    pub(super) bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    pub(super) used_size: Rc<RefCell<usize>>,
}

impl BufferRegistry {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            bounds: Rc::new(RefCell::new(HashMap::new())),
            used_size: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn used_size(&self) -> usize {
        *self.used_size.borrow()
    }

    pub fn register(&self, buffer: &Buffer) -> Result<(), Error> {
        if let Some(registered) = &*buffer.registered.borrow() {
            if &registered.0.reg_id != &self.id {
                return Err(Error::RegisterBufferToMultipleRepositoryUnsupported);
            } else {
                return Ok(());
            }
        }

        let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        let registered = BufferRegistered(BufferRegisteredUndrop {
            gl: self.gl.clone(),
            gl_buffer: gl_buffer.clone(),
            gl_bounds: HashSet::new(),

            reg_id: self.id,
            reg_bounds: Rc::clone(&self.bounds),
            reg_used_size: Rc::downgrade(&self.used_size),

            buffer_size: 0,
            buffer_usage: buffer.usage,
            buffer_queue: Rc::downgrade(&buffer.queue),
            buffer_async_upload: Rc::new(RefCell::new(None)),

            restore_when_drop: false,
        });

        *buffer.registered.borrow_mut() = Some(registered);

        Ok(())
    }

    pub fn capture(
        &self,
        gl_buffer: WebGlBuffer,
        buffer_size: Option<usize>,
        buffer_usage: Option<BufferUsage>,
    ) -> Result<Buffer, Error> {
        let (buffer_size, buffer_usage) = match (buffer_size, buffer_usage) {
            (None, None) | (None, Some(_)) | (Some(_), None) => {
                self.gl
                    .bind_buffer(BUFFER_TARGET.to_gl_enum(), Some(&gl_buffer));

                let buffer_size = match buffer_size {
                    Some(buffer_size) => buffer_size,
                    None => self
                        .gl
                        .get_buffer_parameter(
                            BUFFER_TARGET.to_gl_enum(),
                            WebGl2RenderingContext::BUFFER_SIZE,
                        )
                        .as_f64()
                        .unwrap() as usize,
                };
                let buffer_usage = match buffer_usage {
                    Some(buffer_usage) => buffer_usage,
                    None => {
                        let gl_enum = self
                            .gl
                            .get_buffer_parameter(
                                BUFFER_TARGET.to_gl_enum(),
                                WebGl2RenderingContext::BUFFER_USAGE,
                            )
                            .as_f64()
                            .unwrap() as u32;

                        BufferUsage::from_gl_enum(gl_enum).unwrap()
                    }
                };

                self.gl.bind_buffer(
                    BUFFER_TARGET.to_gl_enum(),
                    self.bounds.borrow().get(&BUFFER_TARGET),
                );

                (buffer_size, buffer_usage)
            }
            (Some(buffer_size), Some(buffer_usage)) => (buffer_size, buffer_usage),
        };

        let queue = Rc::new(RefCell::new(VecDeque::new()));
        let registered = BufferRegistered(BufferRegisteredUndrop {
            gl: self.gl.clone(),
            gl_buffer,
            gl_bounds: HashSet::new(),

            reg_id: self.id,
            reg_bounds: Rc::clone(&self.bounds),
            reg_used_size: Rc::downgrade(&self.used_size),

            buffer_size,
            buffer_usage,
            buffer_queue: Rc::downgrade(&queue),
            buffer_async_upload: Rc::new(RefCell::new(None)),

            restore_when_drop: false,
        });

        *self.used_size.borrow_mut() += buffer_size;

        Ok(Buffer {
            id: Uuid::new_v4(),
            usage: buffer_usage,
            queue,
            registered: Rc::new(RefCell::new(Some(registered))),
        })
    }
}
