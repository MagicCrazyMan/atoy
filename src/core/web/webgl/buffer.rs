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
    Int32Array, Int8Array, Object, Promise, Uint16Array, Uint32Array, Uint8Array,
    Uint8ClampedArray,
};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{client_wait::ClientWaitAsync, conversion::ToGlEnum, error::Error};

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
            BufferData::Preallocation { size } => {
                gl.buffer_data_with_i32(
                    target.gl_enum(),
                    *size as i32,
                    BufferUsage::StreamDraw.gl_enum(),
                );
            }
        };
    }
}

impl BufferSource for BufferData {
    fn load(&mut self) -> BufferData {
        self.clone()
    }
}

/// A buffer source returns an available [`BufferData`] immediately
/// and then uploads to WebGL context immediately.
pub trait BufferSource {
    /// Returns a [`BufferData`].
    fn load(&mut self) -> BufferData;
}

/// A buffer source retrieves data asynchronously
/// and then uploads it to WebGL context after that.
#[async_trait]
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
            .map_or(0, |registered| registered.buffer_size)
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    pub fn flushing(&self) -> bool {
        self.registered
            .borrow()
            .as_ref()
            .map_or(false, |registered| {
                registered.buffer_async_upload.borrow().is_some()
            })
    }

    pub fn ready(&self) -> bool {
        self.registered.borrow().as_ref().is_some()
            && self.queue.borrow().is_empty()
            && !self.flushing()
    }

    pub fn write<S>(&self, source: S)
    where
        S: BufferSource + 'static,
    {
        self.write_with_offset(source, 0)
    }

    pub fn write_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSource + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSourceInner::Sync(Box::new(source)),
            dst_byte_offset,
        ));
    }

    pub fn write_remote<S>(&self, source: S)
    where
        S: BufferSourceAsync + 'static,
    {
        self.write_remote_with_offset(source, 0)
    }

    pub fn write_remote_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSourceAsync + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSourceInner::Async(Box::new(source)),
            dst_byte_offset,
        ));
    }

    pub fn read_to_array_buffer(&self) -> Result<ArrayBuffer, Error> {
        self.read_to_array_buffer_with_params(0)
    }

    pub fn read_to_array_buffer_with_params(
        &self,
        src_byte_offset: usize,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .read_to_array_buffer(src_byte_offset)
    }

    pub async fn read_to_array_buffer_async(&self) -> Result<ArrayBuffer, Error> {
        self.read_to_array_buffer_with_params_async(0, None).await
    }

    pub async fn read_to_array_buffer_with_params_async(
        &self,
        src_byte_offset: usize,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .ok_or(Error::BufferUnregistered)?
            .read_to_array_buffer_async(src_byte_offset, max_retries)
            .await
    }

    pub fn gl_buffer(&self) -> Result<WebGlBuffer, Error> {
        self.registered
            .borrow()
            .as_ref()
            .map(|registered| registered.gl_buffer.clone())
            .ok_or(Error::BufferUnregistered)
    }

    pub fn flush(&self) -> Result<bool, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .flush()
    }

    pub async fn flush_async(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .flush_async()
            .await?;

        Ok(())
    }

    pub fn bind(&self, target: BufferTarget) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .bind(target)
    }

    pub fn unbind(&self, target: BufferTarget) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .unbind(target);
        Ok(())
    }

    pub fn unbind_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .unbind_all();
        Ok(())
    }

    pub fn copy_to(
        &self,
        to: &Buffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
        reallocate: Option<bool>,
    ) -> Result<(), Error> {
        let from = self.registered.borrow();
        let to = to.registered.borrow();
        let (from, to) = (
            from.as_ref().ok_or(Error::BufferUnregistered)?,
            to.as_ref().ok_or(Error::BufferUnregistered)?,
        );

        from.copy_to(&to.gl_buffer, read_offset, write_offset, size, reallocate);

        Ok(())
    }
}

/// [BufferTarget] to use when upload or download data to or from WebGlBuffer.
const BUFFER_TARGET: BufferTarget = BufferTarget::ArrayBuffer;

#[derive(Debug, Clone)]
pub(super) struct BufferRegistered {
    pub(super) gl: WebGl2RenderingContext,
    pub(super) gl_buffer: WebGlBuffer,
    pub(super) gl_bounds: HashSet<BufferTarget>,

    pub(super) reg_id: Uuid,
    pub(super) reg_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    pub(super) reg_used_memory: Weak<RefCell<usize>>,

    pub(super) buffer_size: usize,
    pub(super) buffer_usage: BufferUsage,
    pub(super) buffer_queue: Weak<RefCell<VecDeque<QueueItem>>>,
    pub(super) buffer_async_upload: Rc<
        RefCell<
            Option<(
                Promise,
                Closure<dyn FnMut(JsValue)>,
                Closure<dyn FnMut(JsValue)>,
            )>,
        >,
    >,

    pub(super) restore_when_drop: bool,
}

impl Drop for BufferRegistered {
    fn drop(&mut self) {
        if self.restore_when_drop {
            let Some(buffer_queue) = self.buffer_queue.upgrade() else {
                return;
            };

            if let Ok(data) = self.read_to_array_buffer(0) {
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
        self.gl.delete_buffer(Some(&self.gl_buffer));
        self.reg_used_memory
            .upgrade()
            .map(|used_memory| *used_memory.borrow_mut() -= self.buffer_size);
    }
}

impl BufferRegistered {
    fn bind(&mut self, target: BufferTarget) -> Result<(), Error> {
        if let Some(gl_buffer) = self.reg_bounds.borrow().get(&target) {
            if gl_buffer == &self.gl_buffer {
                return Ok(());
            } else {
                return Err(Error::BufferTargetOccupied(target));
            }
        }

        self.gl.bind_buffer(target.gl_enum(), Some(&self.gl_buffer));
        self.reg_bounds
            .borrow_mut()
            .insert_unique_unchecked(target, self.gl_buffer.clone());
        self.gl_bounds.insert(target);

        Ok(())
    }

    fn unbind(&mut self, target: BufferTarget) {
        if self.gl_bounds.remove(&target) {
            self.gl.bind_buffer(target.gl_enum(), None);
            self.reg_bounds.borrow_mut().remove(&target);
        }
    }

    fn unbind_all(&mut self) {
        for bound in self.gl_bounds.drain() {
            self.gl.bind_buffer(bound.gl_enum(), None);
            self.reg_bounds.borrow_mut().remove(&bound);
        }
    }

    fn flush(&mut self) -> Result<bool, Error> {
        // if there is an ongoing async upload, skips this flush
        if self.buffer_async_upload.borrow().is_some() {
            return Ok(false);
        }

        let Some(buffer_queue) = self.buffer_queue.upgrade() else {
            return Ok(true);
        };

        let mut queue = buffer_queue.borrow_mut();
        if queue.is_empty() {
            return Ok(true);
        }

        while let Some(item) = queue.pop_front() {
            let QueueItem {
                source,
                dst_byte_offset,
            } = item;

            match source {
                BufferSourceInner::Sync(mut source) => {
                    let data = source.load();
                    let data_size = data.byte_length();

                    // if data size larger than the buffer size, expands the buffer size
                    if data_size > self.buffer_size {
                        let new_gl_buffer =
                            self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                        self.gl
                            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&new_gl_buffer));
                        self.gl.buffer_data_with_i32(
                            BUFFER_TARGET.gl_enum(),
                            data_size as i32,
                            self.buffer_usage.gl_enum(),
                        );
                        self.copy_to(&new_gl_buffer, None, None, None, Some(false));

                        // replaces all existing bound buffers to the new buffer
                        for target in self.gl_bounds.iter() {
                            self.gl.bind_buffer(target.gl_enum(), Some(&new_gl_buffer));
                            self.reg_bounds
                                .borrow_mut()
                                .insert(*target, new_gl_buffer.clone());
                        }
                        self.gl_buffer = new_gl_buffer;

                        // updates used memory
                        if let Some(used_memory) = self.reg_used_memory.upgrade() {
                            let mut used_memory = used_memory.borrow_mut();
                            *used_memory -= self.buffer_size;
                            *used_memory += data_size;
                        }
                        self.buffer_size = data_size;
                    } else {
                        self.gl
                            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&self.gl_buffer));
                    }

                    data.upload(&self.gl, BUFFER_TARGET, dst_byte_offset);

                    self.gl.bind_buffer(
                        BUFFER_TARGET.gl_enum(),
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
                        let _ = me.flush();
                    });

                    let mut me = self.clone();
                    let reject = Closure::once(move |value: JsValue| unsafe {
                        // if reject, prints error message, sends error message to channel and skips this source
                        let msg = Box::from_raw(value.as_f64().unwrap() as usize as *mut String);
                        log::error!(
                            "failed to load buffer data from remote source remote: {}",
                            msg
                        );

                        // continues uploading
                        me.buffer_async_upload.borrow_mut().as_mut().take();
                        let _ = me.flush();
                    });

                    *self.buffer_async_upload.borrow_mut() =
                        Some((promise.clone(), resolve, reject));
                    let promise = promise
                        .then(
                            self.buffer_async_upload
                                .borrow()
                                .as_ref()
                                .map(|(_, resolve, _)| resolve)
                                .unwrap(),
                        )
                        .catch(
                            self.buffer_async_upload
                                .borrow()
                                .as_ref()
                                .map(|(_, _, reject)| reject)
                                .unwrap(),
                        );
                    self.buffer_async_upload.borrow_mut().as_mut().unwrap().0 = promise;

                    break;
                }
            }
        }

        Ok(self.buffer_async_upload.borrow().is_none())
    }

    async fn flush_async(&mut self) -> Result<(), Error> {
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
                BufferSourceInner::Async(mut source) => source
                    .load()
                    .await
                    .map_err(|msg| Error::AsyncLoadBufferSourceFailure(msg))?,
            };
            let data_size = data.byte_length();

            // if data size larger than the buffer size, expands the buffer size
            if data_size > self.buffer_size {
                let new_gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                self.gl
                    .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&new_gl_buffer));
                self.gl.buffer_data_with_i32(
                    BUFFER_TARGET.gl_enum(),
                    data_size as i32,
                    self.buffer_usage.gl_enum(),
                );
                self.copy_to(&new_gl_buffer, None, None, None, Some(false));

                // replaces all existing bound buffers to the new buffer
                for target in self.gl_bounds.iter() {
                    self.gl.bind_buffer(target.gl_enum(), Some(&new_gl_buffer));
                    self.reg_bounds
                        .borrow_mut()
                        .insert(*target, new_gl_buffer.clone());
                }
                self.gl_buffer = new_gl_buffer;

                // updates used memory
                if let Some(used_memory) = self.reg_used_memory.upgrade() {
                    let mut used_memory = used_memory.borrow_mut();
                    *used_memory -= self.buffer_size;
                    *used_memory += data_size;
                }
                self.buffer_size = data_size;
            } else {
                self.gl
                    .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&self.gl_buffer));
            }

            data.upload(&self.gl, BUFFER_TARGET, dst_byte_offset);

            self.gl.bind_buffer(
                BUFFER_TARGET.gl_enum(),
                self.reg_bounds.borrow().get(&BUFFER_TARGET),
            );
        }

        Ok(())
    }

    fn read_to_array_buffer(&self, src_byte_offset: usize) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.copy_to(&tmp, None, None, None, Some(true));

        let data = Uint8Array::new_with_length(self.buffer_size as u32);
        self.gl.bind_buffer(BUFFER_TARGET.gl_enum(), Some(&tmp));
        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            BUFFER_TARGET.gl_enum(),
            src_byte_offset as i32,
            &data,
        );
        self.gl.bind_buffer(
            BUFFER_TARGET.gl_enum(),
            self.reg_bounds.borrow().get(&BUFFER_TARGET),
        );
        self.gl.delete_buffer(Some(&tmp));

        Ok(data.buffer())
    }

    async fn read_to_array_buffer_async(
        &self,
        src_byte_offset: usize,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.copy_to(&tmp, None, None, None, Some(true));

        ClientWaitAsync::new(self.gl.clone(), 0, 5, max_retries)
            .wait()
            .await?;

        let data = Uint8Array::new_with_length(self.buffer_size as u32);
        self.gl.bind_buffer(BUFFER_TARGET.gl_enum(), Some(&tmp));
        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            BUFFER_TARGET.gl_enum(),
            src_byte_offset as i32,
            &data,
        );
        self.gl.bind_buffer(
            BUFFER_TARGET.gl_enum(),
            self.reg_bounds.borrow().get(&BUFFER_TARGET),
        );
        self.gl.delete_buffer(Some(&tmp));

        Ok(data.buffer())
    }

    fn copy_to(
        &self,
        to: &WebGlBuffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
        reallocate: Option<bool>,
    ) {
        let read_offset = read_offset.unwrap_or(0);
        let write_offset = write_offset.unwrap_or(0);
        let size = size.unwrap_or(self.buffer_size);
        let reallocate = reallocate.unwrap_or(false);

        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.gl_enum(),
            Some(&self.gl_buffer),
        );
        self.gl
            .bind_buffer(BufferTarget::CopyWriteBuffer.gl_enum(), Some(to));
        if reallocate {
            self.gl.buffer_data_with_i32(
                BufferTarget::CopyWriteBuffer.gl_enum(),
                size as i32,
                BufferUsage::StreamDraw.gl_enum(),
            );
        }
        self.gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            BufferTarget::CopyReadBuffer.gl_enum(),
            BufferTarget::CopyWriteBuffer.gl_enum(),
            read_offset as i32,
            write_offset as i32,
            size as i32,
        );
        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyReadBuffer),
        );
        self.gl.bind_buffer(
            BufferTarget::CopyWriteBuffer.gl_enum(),
            self.reg_bounds.borrow().get(&BufferTarget::CopyWriteBuffer),
        );
    }
}

#[derive(Debug)]
pub struct BufferRegistry {
    pub(super) id: Uuid,
    pub(super) gl: WebGl2RenderingContext,
    pub(super) bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    pub(super) used_memory: Rc<RefCell<usize>>,
}

impl BufferRegistry {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            bounds: Rc::new(RefCell::new(HashMap::new())),
            used_memory: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn used_memory(&self) -> usize {
        *self.used_memory.borrow()
    }

    pub fn bounds(&self) -> Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>> {
        Rc::clone(&self.bounds)
    }

    pub fn register(&self, buffer: &Buffer) -> Result<(), Error> {
        if let Some(registered) = &*buffer.registered.borrow() {
            if &registered.reg_id != &self.id {
                return Err(Error::RegisterBufferToMultipleRepositoryUnsupported);
            } else {
                return Ok(());
            }
        }

        let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        let registered = BufferRegistered {
            gl: self.gl.clone(),
            gl_buffer: gl_buffer.clone(),
            gl_bounds: HashSet::new(),

            reg_id: self.id,
            reg_bounds: Rc::clone(&self.bounds),
            reg_used_memory: Rc::downgrade(&self.used_memory),

            buffer_size: 0,
            buffer_usage: buffer.usage,
            buffer_queue: Rc::downgrade(&buffer.queue),
            buffer_async_upload: Rc::new(RefCell::new(None)),

            restore_when_drop: false,
        };

        *buffer.registered.borrow_mut() = Some(registered);

        Ok(())
    }

    // /// Registers a native [`WebGlBuffer`] to the registry and wraps it into a [`Buffer`].
    // ///
    // /// Buffer size and usage are queried from WebGL context if `size` or `usage` is not provided.
    // /// Always provides correct size and usage to avoid stalling CPU.
    // pub fn register_gl_buffer(
    //     &self,
    //     gl_buffer: WebGlBuffer,
    //     size: Option<usize>,
    //     usage: Option<BufferUsage>,
    // ) -> Result<Buffer, Error> {
    //     let require_parameter = size.is_none() || usage.is_none();
    //     if require_parameter {
    //         self.gl
    //             .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&gl_buffer));
    //     }
    //     let capacity = match size {
    //         Some(capacity) => capacity,
    //         None => self
    //             .gl
    //             .get_buffer_parameter(BUFFER_TARGET.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
    //             .as_f64()
    //             .map(|size| size as usize)
    //             .ok_or(Error::BufferUnexpectedDropped)?,
    //     };
    //     let usage = match usage {
    //         Some(usage) => usage,
    //         None => self
    //             .gl
    //             .get_buffer_parameter(
    //                 BUFFER_TARGET.gl_enum(),
    //                 WebGl2RenderingContext::BUFFER_USAGE,
    //             )
    //             .as_f64()
    //             .map(|usage| BufferUsage::from_gl_enum(usage as u32))
    //             .ok_or(Error::BufferUnexpectedDropped)?,
    //     };
    //     if require_parameter {
    //         self.gl.bind_buffer(
    //             BUFFER_TARGET.gl_enum(),
    //             self.bounds.borrow().get(&BUFFER_TARGET),
    //         );
    //     }

    //     let queue = Rc::new(RefCell::new(Vec::new()));
    //     let queue_size = Rc::new(RefCell::new(0));

    //     let registered = BufferRegistered {
    //         gl: self.gl.clone(),
    //         gl_buffer,
    //         gl_bounds: HashSet::new(),

    //         reg_id: self.id,
    //         reg_bounds: Rc::clone(&self.bounds),
    //         reg_used_memory: Rc::downgrade(&self.used_memory),

    //         buffer_capacity: capacity,
    //         buffer_queue: Rc::downgrade(&queue),
    //         buffer_queue_size: Rc::downgrade(&queue_size),

    //         restore_when_drop: false,
    //     };

    //     Ok(Buffer {
    //         id: Uuid::new_v4(),
    //         capacity,
    //         usage,
    //         queue_size,
    //         queue,
    //         registered: Rc::new(RefCell::new(Some(registered))),
    //     })
    // }
}
