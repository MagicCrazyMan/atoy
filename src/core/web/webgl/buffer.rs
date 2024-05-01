use std::{
    borrow::Cow,
    cell::RefCell,
    collections::VecDeque,
    fmt::Debug,
    future::Future,
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
use web_sys::{Request, RequestInit, WebGl2RenderingContext, WebGlBuffer};

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

// impl BufferUsage {
//     fn from_gl_enum(value: u32) -> Self {
//         match value {
//             WebGl2RenderingContext::STATIC_DRAW => BufferUsage::StaticDraw,
//             WebGl2RenderingContext::DYNAMIC_DRAW => BufferUsage::DynamicDraw,
//             WebGl2RenderingContext::STREAM_DRAW => BufferUsage::StreamDraw,
//             WebGl2RenderingContext::STATIC_READ => BufferUsage::StaticRead,
//             WebGl2RenderingContext::DYNAMIC_READ => BufferUsage::DynamicRead,
//             WebGl2RenderingContext::STREAM_READ => BufferUsage::StreamRead,
//             WebGl2RenderingContext::STATIC_COPY => BufferUsage::StaticCopy,
//             WebGl2RenderingContext::DYNAMIC_COPY => BufferUsage::DynamicCopy,
//             WebGl2RenderingContext::STREAM_COPY => BufferUsage::StreamCopy,
//             _ => panic!("{} is not a valid BufferUsage enum value", value),
//         }
//     }
// }

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

impl BufferSourceLocal for BufferData {
    fn load(&self) -> BufferData {
        self.clone()
    }
}

impl From<JsValue> for BufferData {
    fn from(value: JsValue) -> Self {
        todo!()
    }
}

impl From<BufferData> for JsValue {
    fn from(value: BufferData) -> Self {
        todo!()
    }
}

/// A local buffer source that can be uploaded to WebGL context immediately.
pub trait BufferSourceLocal {
    /// Returns a [`BufferData`].
    fn load(&self) -> BufferData;
}

/// A remote buffer source that have to retrieve data from somewhere else asynchronously
/// and then upload it to WebGL context after that.
#[async_trait]
pub trait BufferSourceRemote {
    /// Returns a [`BufferData`] or an error message.
    async fn load(&self) -> Result<BufferData, String>;
}

enum BufferSource {
    Local(Box<dyn BufferSourceLocal>),
    Remote(Box<dyn BufferSourceRemote>),
}

impl Debug for BufferSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(_) => f.debug_tuple("Local").finish(),
            Self::Remote(_) => f.debug_tuple("Remote").finish(),
        }
    }
}

#[derive(Debug)]
pub(super) struct QueueItem {
    source: BufferSource,
    dst_byte_offset: usize,
}

impl QueueItem {
    fn new(source: BufferSource, dst_byte_offset: usize) -> Self {
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

    pub fn update_to_date(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    pub fn ready(&self) -> bool {
        self.update_to_date() && self.registered.borrow().is_some()
    }

    pub fn write<S>(&self, source: S)
    where
        S: BufferSourceLocal + 'static,
    {
        self.write_with_offset(source, 0)
    }

    pub fn write_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSourceLocal + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSource::Local(Box::new(source)),
            dst_byte_offset,
        ));
    }

    pub fn write_remote<S>(&self, source: S)
    where
        S: BufferSourceRemote + 'static,
    {
        self.write_remote_with_offset(source, 0)
    }

    pub fn write_remote_with_offset<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSourceRemote + 'static,
    {
        self.queue.borrow_mut().push_back(QueueItem::new(
            BufferSource::Remote(Box::new(source)),
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
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::BufferUnregistered)?;
        registered.read_to_array_buffer(src_byte_offset)
    }

    pub async fn read_to_array_buffer_async(&self) -> Result<ArrayBuffer, Error> {
        self.read_to_array_buffer_with_params_async(0, None).await
    }

    pub async fn read_to_array_buffer_with_params_async(
        &self,
        src_byte_offset: usize,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::BufferUnregistered)?;
        registered
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

    pub fn flush(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::BufferUnregistered)?
            .flush();

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

    pub fn copy_to_buffer(
        &self,
        to: &Buffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
    ) -> Result<(), Error> {
        let mut from = self.registered.borrow_mut();
        let to = to.registered.borrow();
        let (from, to) = (
            from.as_mut().ok_or(Error::BufferUnregistered)?,
            to.as_ref().ok_or(Error::BufferUnregistered)?,
        );

        from.copy_to_buffer(
            &to.gl_buffer,
            read_offset,
            write_offset,
            size.or(Some(from.buffer_size.min(to.buffer_size))),
        )
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
                    QueueItem::new(BufferSource::Local(Box::new(buffer_data)), 0),
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

    fn flush(&self) {
        // if there is an ongoing async upload, skips this flush
        if self.buffer_async_upload.borrow().is_some() {
            return;
        }

        let Some(buffer_queue) = self.buffer_queue.upgrade() else {
            return;
        };

        let mut queue = buffer_queue.borrow_mut();
        if queue.is_empty() {
            return;
        }

        self.gl
            .bind_buffer(BUFFER_TARGET.gl_enum(), Some(&self.gl_buffer));

        while let Some(QueueItem {
            source,
            dst_byte_offset,
        }) = queue.pop_front()
        {
            match source {
                BufferSource::Local(source) => {
                    source
                        .load()
                        .upload(&self.gl, BUFFER_TARGET, dst_byte_offset);
                }
                BufferSource::Remote(source) => {
                    let promise = future_to_promise(async move {
                        let data = source.load().await;
                        match data {
                            Ok(data) => Ok(JsValue::from(data)),
                            Err(msg) => {
                                // just return error message as UTF-8 bytes buffer, to prevent converting between UTF-16 and UTF-8
                                let str_array_buffer = ArrayBuffer::new(msg.len() as u32);
                                Uint8Array::new(&str_array_buffer).copy_from(msg.as_bytes());
                                Err(JsValue::from(str_array_buffer))
                            }
                        }
                    });

                    let me = self.clone();
                    let resolve = Closure::once(move |value: JsValue| {
                        let buffer_data = BufferData::from(value);
                        let Some(queue) = me.buffer_queue.upgrade() else {
                            return;
                        };

                        // adds buffer data as the first value to the queue and then continues uploading
                        queue.borrow_mut().push_front(QueueItem::new(
                            BufferSource::Local(Box::new(buffer_data)),
                            dst_byte_offset,
                        ));
                        me.buffer_async_upload.borrow_mut().as_mut().take();
                        me.flush();
                    });
                    let me = self.clone();
                    let reject = Closure::once(move |value: JsValue| {
                        // if reject, prints error message, sends error message to channel and skips this source
                        let str_array_buffer = ArrayBuffer::from(value);
                        let mut str_bytes = vec![0u8; str_array_buffer.byte_length() as usize];
                        Uint8Array::new(&str_array_buffer).copy_to(str_bytes.as_mut_slice());
                        let msg = String::from_utf8(str_bytes).unwrap();

                        log::error!(
                            "failed to load buffer data from remote source remote: {}",
                            msg
                        );

                        // continues uploading
                        me.buffer_async_upload.borrow_mut().as_mut().take();
                        me.flush();
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

        self.gl.bind_buffer(
            BUFFER_TARGET.gl_enum(),
            self.reg_bounds.borrow().get(&BUFFER_TARGET),
        );
    }

    fn read_to_array_buffer(&mut self, src_byte_offset: usize) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.gl
            .bind_buffer(BufferTarget::CopyWriteBuffer.gl_enum(), Some(&tmp));
        self.gl.buffer_data_with_i32(
            BufferTarget::CopyWriteBuffer.gl_enum(),
            self.buffer_size as i32,
            BufferUsage::StreamRead.gl_enum(),
        );
        self.copy_to_buffer(&tmp, None, None, None)?;

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
        &mut self,
        src_byte_offset: usize,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let tmp = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
        self.gl
            .bind_buffer(BufferTarget::CopyWriteBuffer.gl_enum(), Some(&tmp));
        self.gl.buffer_data_with_i32(
            BufferTarget::CopyWriteBuffer.gl_enum(),
            self.buffer_size as i32,
            BufferUsage::StreamRead.gl_enum(),
        );
        self.copy_to_buffer(&tmp, None, None, None)?;

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

    fn copy_to_buffer(
        &mut self,
        to: &WebGlBuffer,
        read_offset: Option<usize>,
        write_offset: Option<usize>,
        size: Option<usize>,
    ) -> Result<(), Error> {
        let read_offset = read_offset.unwrap_or(0);
        let write_offset = write_offset.unwrap_or(0);
        let size = size.unwrap_or(self.buffer_size);

        self.gl.bind_buffer(
            BufferTarget::CopyReadBuffer.gl_enum(),
            Some(&self.gl_buffer),
        );
        self.gl
            .bind_buffer(BufferTarget::CopyWriteBuffer.gl_enum(), Some(to));
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

        Ok(())
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
