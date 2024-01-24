use std::{
    cell::RefCell,
    hash::Hash,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap};
use log::debug;
use uuid::Uuid;
use web_sys::{
    js_sys::{
        ArrayBuffer, BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array,
        Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::{
    lru::{Lru, LruNode},
    utils::format_bytes_length,
};

use super::{
    conversion::{GLint, ToGlEnum},
    error::Error,
};

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

/// Available component size of a value get from buffer.
/// According to WebGL definition, it should only be `1`, `2`, `3` or `4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BufferComponentSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

/// Available buffer data types mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferDataType {
    Float,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    HalfFloat,
    Int_2_10_10_10_Rev,
    UnsignedInt_2_10_10_10_Rev,
}

impl BufferDataType {
    /// Gets bytes length of a data type.
    pub fn bytes_length(&self) -> GLint {
        match self {
            BufferDataType::Float => 4,
            BufferDataType::Byte => 1,
            BufferDataType::Short => 2,
            BufferDataType::Int => 4,
            BufferDataType::UnsignedByte => 1,
            BufferDataType::UnsignedShort => 2,
            BufferDataType::UnsignedInt => 4,
            BufferDataType::HalfFloat => 2,
            BufferDataType::Int_2_10_10_10_Rev => 4,
            BufferDataType::UnsignedInt_2_10_10_10_Rev => 4,
        }
    }
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

/// Available buffer sources.
///
/// # Note
///
/// Since the linear memory of WASM runtime is impossible to shrink for now,
/// high memory usage could happen if developer create a large WASM native buffer, for example, `Vec<u8>`.
/// It is always a good idea to avoid creating native buffer, use `TypedArrayBuffer` from JavaScript instead.
pub enum BufferSource {
    Preallocate {
        bytes_length: usize,
    },
    Function {
        callback: Box<dyn Fn() -> BufferSource>,
        data_length: usize,
        src_offset: usize,
        src_length: usize,
    },
    Binary {
        data: Box<dyn AsRef<[u8]>>,
        src_offset: usize,
        src_length: usize,
    },
    ArrayBuffer {
        data: ArrayBuffer,
    },
    Int8Array {
        data: Int8Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint8Array {
        data: Uint8Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        src_offset: usize,
        src_length: usize,
    },
    Int16Array {
        data: Int16Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint16Array {
        data: Uint16Array,
        src_offset: usize,
        src_length: usize,
    },
    Int32Array {
        data: Int32Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint32Array {
        data: Uint32Array,
        src_offset: usize,
        src_length: usize,
    },
    Float32Array {
        data: Float32Array,
        src_offset: usize,
        src_length: usize,
    },
    Float64Array {
        data: Float64Array,
        src_offset: usize,
        src_length: usize,
    },
    BigInt64Array {
        data: BigInt64Array,
        src_offset: usize,
        src_length: usize,
    },
    BigUint64Array {
        data: BigUint64Array,
        src_offset: usize,
        src_length: usize,
    },
}

impl BufferSource {
    fn collect_typed_array_buffer(&self) -> (&Object, usize, usize) {
        match self {
            BufferSource::Preallocate { .. }
            | BufferSource::Function { .. }
            | BufferSource::Binary { .. }
            | BufferSource::ArrayBuffer { .. } => {
                unreachable!()
            }
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint8Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
        }
    }

    /// Buffers data to WebGL runtime.
    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        match self {
            BufferSource::Preallocate { bytes_length } => {
                gl.buffer_data_with_i32(target.gl_enum(), *bytes_length as i32, usage.gl_enum())
            }
            BufferSource::Function { callback: data, .. } => {
                let source = data();
                if let BufferSource::Function { .. } = source {
                    panic!("recursive BufferSource::Function is not allowed");
                }
                source.buffer_data(gl, target, usage);
            }
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
            } => gl.buffer_data_with_u8_array_and_src_offset_and_length(
                target.gl_enum(),
                data.as_ref().as_ref(),
                usage.gl_enum(),
                *src_offset as u32,
                *src_length as u32,
            ),
            BufferSource::ArrayBuffer { data } => {
                gl.buffer_data_with_opt_array_buffer(target.gl_enum(), Some(data), usage.gl_enum())
            }
            _ => {
                let (data, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_data_with_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    data,
                    usage.gl_enum(),
                    src_offset as u32,
                    src_length as u32,
                );
            }
        }
    }

    /// Buffers sub data to WebGL runtime.
    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        match self {
            BufferSource::Preallocate { bytes_length } => {
                let src_data = Uint8Array::new_with_length(*bytes_length as u32);
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    &src_data,
                    0,
                    *bytes_length as u32,
                )
            }
            BufferSource::Function { callback, .. } => {
                let source = callback();
                if let BufferSource::Function { .. } = source {
                    panic!("recursively BufferSource::Function is not allowed");
                }
                source.buffer_sub_data(gl, target, dst_byte_offset);
            }
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
            } => gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                target.gl_enum(),
                dst_byte_offset as i32,
                data.as_ref().as_ref(),
                *src_offset as u32,
                *src_length as u32,
            ),
            BufferSource::ArrayBuffer { data } => gl.buffer_sub_data_with_i32_and_array_buffer(
                target.gl_enum(),
                dst_byte_offset as i32,
                data,
            ),
            _ => {
                let (data, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    data,
                    src_offset as u32,
                    src_length as u32,
                )
            }
        }
    }

    /// Returns the length of data in bytes.
    pub fn bytes_length(&self) -> usize {
        let (raw_length, src_offset, src_length) = match self {
            BufferSource::Preallocate { bytes_length } => (*bytes_length, 0, 0),
            BufferSource::Function {
                data_length,
                src_offset,
                src_length,
                ..
            } => (*data_length, *src_offset, *src_length),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
                ..
            } => (data.as_ref().as_ref().len(), *src_offset, *src_length),
            BufferSource::ArrayBuffer { data } => (data.byte_length() as usize, 0, 0),
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
        };

        if src_length == 0 {
            raw_length.saturating_sub(src_offset)
        } else {
            src_length
        }
    }
}

impl BufferSource {
    /// Constructs a new preallocation only buffer source.
    pub fn preallocate(bytes_length: usize) -> Self {
        Self::Preallocate { bytes_length }
    }

    /// Constructs a new buffer source from a callback function.
    /// Preflies information is required and it should be same as the callback value.
    pub fn from_function<F>(
        callback: F,
        data_length: usize,
        src_offset: usize,
        src_length: usize,
    ) -> Self
    where
        F: Fn() -> BufferSource + 'static,
    {
        Self::Function {
            callback: Box::new(callback),
            data_length,
            src_offset,
            src_length,
        }
    }

    /// Constructs a new buffer source from WASM native buffer.
    pub fn from_binary<D>(data: D, src_offset: usize, src_length: usize) -> Self
    where
        D: AsRef<[u8]> + 'static,
    {
        Self::Binary {
            data: Box::new(data),
            src_offset,
            src_length,
        }
    }

    /// Constructs a new buffer source from [`ArrayBuffer`].
    pub fn from_array_buffer(data: ArrayBuffer) -> Self {
        Self::ArrayBuffer { data }
    }
}

macro_rules! impl_typed_array {
    ($(($from: ident, $source: tt, $kind: ident, $name: expr)),+) => {
        impl BufferSource {
            $(
                #[doc = "Constructs a new buffer source from "]
                #[doc = $name]
                #[doc = "."]
                pub fn $from(
                    data: $source,
                    src_offset: usize,
                    src_length: usize,
                ) -> Self {
                    Self::$kind {
                        data,
                        src_offset,
                        src_length
                    }
                }
            )+
        }
    };
}

impl_typed_array! {
    (from_int8_array, Int8Array, Int8Array, "[`Int8Array`]"),
    (from_uint8_array, Uint8Array, Uint8Array, "[`Uint8Array`]"),
    (from_uint8_clamped_array, Uint8ClampedArray, Uint8ClampedArray, "[`Uint8ClampedArray`]"),
    (from_int16_array, Int16Array, Int16Array, "[`Int16Array`]"),
    (from_uint16_array, Uint16Array, Uint16Array, "[`Uint16Array`]"),
    (from_int32_array, Int32Array, Int32Array, "[`Int32Array`]"),
    (from_uint32_array, Uint32Array, Uint32Array, "[`Uint32Array`]"),
    (from_float32_array, Float32Array, Float32Array, "[`Float32Array`]"),
    (from_float64_array, Float64Array, Float64Array, "[`Float64Array`]"),
    (from_big_int64_array, BigInt64Array, BigInt64Array, "[`BigInt64Array`]"),
    (from_big_uint64_array, BigUint64Array, BigUint64Array, "[`BigUint64Array`]")
}

struct BufferDescriptorInner {
    id: Uuid,
    usage: BufferUsage,
    memory_policy: MemoryPolicy,

    queue_bytes_length: usize,
    queue: Vec<(BufferSource, usize)>,

    store: Option<(
        WebGl2RenderingContext,
        *mut HashMap<Uuid, StorageItem>,
        *mut HashMap<u32, Uuid>,
        *mut usize,
        *mut Lru<Uuid>,
    )>,
}

impl Drop for BufferDescriptorInner {
    fn drop(&mut self) {
        unsafe {
            let Some((gl, store, ubo_bindings, used_memory, lru)) = self.store.as_ref() else {
                return;
            };

            let Some(item) = (**store).remove(&self.id) else {
                return;
            };

            gl.delete_buffer(Some(&item.buffer));
            (**used_memory) -= item.bytes_length;
            (**lru).remove(item.lru_node);

            if let Some(key) = (**ubo_bindings)
                .iter()
                .find(|(_, v)| **v == self.id)
                .map(|(v, _)| *v)
            {
                (**ubo_bindings).remove(&key);
            }

            debug!("buffer descriptor {} dropped", &self.id);
        }
    }
}

/// A key to share and control the [`WebGlBuffer`].
/// Checks [`BufferStore`] for more details.
#[derive(Clone)]
pub struct BufferDescriptor(Rc<RefCell<BufferDescriptorInner>>);

impl BufferDescriptor {
    /// Constructs a new buffer descriptor with specified [`BufferSource`] and [`BufferUsage`].
    pub fn new(source: BufferSource, usage: BufferUsage) -> Self {
        Self::with_memory_policy(source, usage, MemoryPolicy::Default)
    }

    /// Constructs a new buffer descriptor with specified [`BufferSource`], [`BufferUsage`] and [`MemoryPolicy`].
    pub fn with_memory_policy(
        source: BufferSource,
        usage: BufferUsage,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(BufferDescriptorInner {
            id: Uuid::new_v4(),
            usage,
            memory_policy,
            queue_bytes_length: source.bytes_length(),
            queue: Vec::from([(source, 0)]),

            store: None,
        })))
    }

    /// Returns buffer descriptor id.
    pub fn id(&self) -> Uuid {
        self.0.borrow().id
    }

    /// Returns the [`BufferTarget`].
    pub fn usage(&self) -> BufferUsage {
        self.0.borrow().usage
    }

    /// Returns [`MemoryPolicyKind`].
    pub fn memory_policy(&self) -> MemoryPolicyKind {
        self.0.borrow().memory_policy.kind()
    }

    /// Sets the [`MemoryPolicy`] of this buffer descriptor.
    pub fn set_memory_policy(&mut self, policy: MemoryPolicy) {
        self.0.borrow_mut().memory_policy = policy;
    }

    /// Allocates new [`WebGlBuffer`] and buffers data to it.
    /// This operation overrides existing data.
    pub fn buffer_data(&mut self, source: BufferSource) {
        let mut inner = self.0.borrow_mut();
        inner.queue_bytes_length = source.bytes_length();
        inner.queue.clear();
        inner.queue.push((source, 0));
    }

    /// Buffers sub data to [`WebGlBuffer`].
    pub fn buffer_sub_data(&mut self, source: BufferSource, dst_byte_offset: usize) {
        let mut inner = self.0.borrow_mut();

        let bytes_length = dst_byte_offset + source.bytes_length();
        if dst_byte_offset == 0 && bytes_length >= inner.queue_bytes_length {
            // overrides sources in queue if new source covers all
            inner.queue_bytes_length = bytes_length;
            inner.queue.clear();
            inner.queue.push((source, 0));
        } else {
            inner.queue_bytes_length = inner.queue_bytes_length.max(bytes_length);
            inner.queue.push((source, dst_byte_offset));
        }
    }
}

/// Memory freeing policies.
pub enum MemoryPolicy {
    Default,
    Restorable(Rc<RefCell<dyn Fn() -> BufferSource>>),
    Unfree,
}

/// Memory freeing policy kinds.
/// Checks [`MemoryPolicy`] for more details.
pub enum MemoryPolicyKind {
    Default,
    Restorable,
    Unfree,
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self::Default
    }
}

impl MemoryPolicy {
    /// Constructs a default memory policy.
    pub fn default() -> Self {
        Self::Default
    }

    /// Constructs a unfreeable memory policy.
    pub fn unfree() -> Self {
        Self::Unfree
    }

    /// Constructs a restorable memory policy.
    pub fn restorable<F>(f: F) -> Self
    where
        F: Fn() -> BufferSource + 'static,
    {
        Self::Restorable(Rc::new(RefCell::new(f)))
    }

    /// Returns [`MemoryPolicyKind`] associated with this policy.
    pub fn kind(&self) -> MemoryPolicyKind {
        match self {
            MemoryPolicy::Default => MemoryPolicyKind::Default,
            MemoryPolicy::Restorable(_) => MemoryPolicyKind::Restorable,
            MemoryPolicy::Unfree => MemoryPolicyKind::Unfree,
        }
    }
}

struct StorageItem {
    using: bool,
    buffer: WebGlBuffer,
    bytes_length: usize,
    lru_node: *mut LruNode<Uuid>,
    descriptor: Weak<RefCell<BufferDescriptorInner>>,
}

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    available_memory: usize,
    used_memory: *mut usize,
    store: *mut HashMap<Uuid, StorageItem>,
    ubo_bindings: *mut HashMap<u32, Uuid>,
    lru: *mut Lru<Uuid>,
}

impl Drop for BufferStore {
    fn drop(&mut self) {
        unsafe {
            let gl = &self.gl;
            (*self.store).iter().for_each(|(_, item)| {
                let StorageItem {
                    buffer, descriptor, ..
                } = item;
                let Some(descriptor) = descriptor.upgrade() else {
                    // deletes if descriptor dropped
                    self.gl.delete_buffer(Some(&item.buffer));
                    return;
                };
                let data = Uint8Array::new_with_length(item.bytes_length as u32);
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&item.buffer));
                self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &data,
                );
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
                self.gl.delete_buffer(Some(&item.buffer));
                descriptor.borrow_mut().queue_bytes_length = item.bytes_length;
                descriptor.borrow_mut().queue.push((
                    BufferSource::from_uint8_array(data, 0, item.bytes_length),
                    0,
                ));
                gl.delete_buffer(Some(&buffer));
                // store dropped, no need to update LRU anymore
            });
        }
    }
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited memory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_available_memory(gl, i32::MAX as usize)
    }

    /// Constructs a new buffer store with a maximum available memory.
    pub fn with_available_memory(gl: WebGl2RenderingContext, available_memory: usize) -> Self {
        Self {
            gl,
            available_memory: available_memory.min(i32::MAX as usize),
            used_memory: Box::leak(Box::new(0)),
            store: Box::leak(Box::new(HashMap::new())),
            ubo_bindings: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),
        }
    }

    /// Returns the maximum available memory in bytes.
    /// Returns [`i32::MAX`] if not specified.
    pub fn available_memory(&self) -> usize {
        self.available_memory
    }

    /// Returns current used memory in bytes.
    pub fn used_memory(&self) -> usize {
        unsafe { *self.used_memory }
    }

    /// Uses a [`WebGlBuffer`] by a [`BufferDescriptor`] and buffer data to it if necessary.
    /// Remembers to calls [`BufferStore::unuse_buffer`] after using the [`WebGlBuffer`],
    /// or the [`WebGlBuffer`] will never be freed.
    pub fn use_buffer(
        &mut self,
        descriptor: &BufferDescriptor,
        target: BufferTarget,
    ) -> Result<WebGlBuffer, Error> {
        unsafe {
            let BufferDescriptorInner {
                id,
                usage,
                queue,
                queue_bytes_length,
                store,
                ..
            } = &mut *descriptor.0.borrow_mut();

            let mut o;
            let item = match (*self.store).entry(*id) {
                Entry::Occupied(occupied) => {
                    o = occupied;
                    o.get_mut()
                }
                Entry::Vacant(vacant) => {
                    debug!(
                        target: "BufferStore",
                        "create new buffer for {}", id
                    );

                    vacant.insert(StorageItem {
                        using: false,
                        buffer: self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?,
                        bytes_length: 0,
                        lru_node: LruNode::new(*id),
                        descriptor: Rc::downgrade(&descriptor.0),
                    })
                }
            };
            let buffer = item.buffer.clone();

            // update buffer data
            if queue.len() > 0 {
                let new_bytes_length = *queue_bytes_length;
                let old_bytes_length = item.bytes_length;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                if new_bytes_length > old_bytes_length {
                    if new_bytes_length == queue[0].0.bytes_length() {
                        let (source, _) = queue.remove(0);
                        source.buffer_data(&self.gl, target, *usage);
                    } else {
                        self.gl.buffer_data_with_i32(
                            target.gl_enum(),
                            new_bytes_length as i32,
                            usage.gl_enum(),
                        );
                    }
                }
                for (source, dst_byte_offset) in queue.drain(..) {
                    source.buffer_sub_data(&self.gl, target, dst_byte_offset);
                }

                item.bytes_length = new_bytes_length;
                (*self.used_memory) = (*self.used_memory) - old_bytes_length + new_bytes_length;

                debug!(
                    target: "BufferStore",
                    "buffer new data for {}, old length {}, new length {}", id, old_bytes_length, new_bytes_length
                );

                self.gl.bind_buffer(target.gl_enum(), None);
            }

            item.using = true;
            (*self.lru).cache(item.lru_node);
            *store = Some((
                self.gl.clone(),
                self.store,
                self.ubo_bindings,
                self.used_memory,
                self.lru,
            ));
            self.free();

            Ok(buffer)
        }
    }

    /// Unuses a [`WebGlBuffer`] by a [`BufferDescriptor`].
    pub fn unuse_buffer(&mut self, descriptor: &BufferDescriptor) {
        unsafe {
            let Some(item) = (*self.store).get_mut(&descriptor.id()) else {
                return;
            };
            item.using = false;
        }
    }

    /// Frees memory if used memory exceeds the maximum available memory.
    fn free(&mut self) {
        // removes buffer from the least recently used until memory usage lower than limitation
        unsafe {
            if *self.used_memory <= self.available_memory {
                return;
            }

            let mut next_node = (*self.lru).least_recently();
            while *self.used_memory > self.available_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };
                let id = (*current_node).data();
                let Entry::Occupied(occupied) = (*self.store).entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let item = occupied.get();
                let Some(descriptor) = item.descriptor.upgrade() else {
                    // deletes if descriptor dropped
                    self.gl.delete_buffer(Some(&item.buffer));
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };
                // skips if using
                if item.using {
                    next_node = (*current_node).more_recently();
                    continue;
                }
                let mut descriptor = descriptor.borrow_mut();
                // skips if unfreeable
                if let MemoryPolicy::Unfree = &descriptor.memory_policy {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                match &descriptor.memory_policy {
                    MemoryPolicy::Default => {
                        // default, gets buffer data back from WebGlBuffer
                        let data = Uint8Array::new_with_length(item.bytes_length as u32);
                        self.gl
                            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&item.buffer));
                        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                            WebGl2RenderingContext::ARRAY_BUFFER,
                            0,
                            &data,
                        );
                        self.gl
                            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
                        self.gl.delete_buffer(Some(&item.buffer));
                        descriptor.queue_bytes_length = item.bytes_length;
                        descriptor.queue.push((
                            BufferSource::from_uint8_array(data, 0, item.bytes_length),
                            0,
                        ));
                    }
                    MemoryPolicy::Restorable(restore) => {
                        self.gl.delete_buffer(Some(&item.buffer));
                        let restore = Rc::clone(&restore);
                        let source = BufferSource::Function {
                            callback: Box::new(move || restore.borrow_mut()()),
                            data_length: item.bytes_length,
                            src_offset: 0,
                            src_length: item.bytes_length,
                        };
                        descriptor.queue_bytes_length = item.bytes_length;
                        descriptor.queue.push((source, 0));
                    }
                    MemoryPolicy::Unfree => unreachable!(),
                }
                // reduces memory
                (*self.used_memory) -= item.bytes_length;
                // removes LRU
                next_node = (*item.lru_node).more_recently();
                (*self.lru).remove(item.lru_node);
                // logs
                match &descriptor.memory_policy {
                    MemoryPolicy::Default => {
                        debug!(
                            target: "BufferStore",
                            "free buffer (default) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(item.bytes_length),
                            format_bytes_length(*self.used_memory )
                        );
                    }
                    MemoryPolicy::Restorable(_) => {
                        debug!(
                            target: "BufferStore",
                            "free buffer (restorable) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(item.bytes_length ),
                            format_bytes_length(*self.used_memory )
                        );
                    }
                    MemoryPolicy::Unfree => unreachable!(),
                }
                occupied.remove();
            }
        }
    }

    pub fn bind_uniform_buffer_object(
        &mut self,
        descriptor: &BufferDescriptor,
        binding: u32,
        offset_and_size: Option<(usize, usize)>,
    ) -> Result<(), Error> {
        unsafe {
            if let Some(id) = (*self.ubo_bindings).get(&binding) {
                if *id != descriptor.id() {
                    return Err(Error::UniformBufferObjectIndexAlreadyBound(binding));
                }

                self.use_buffer(descriptor, BufferTarget::UniformBuffer)?;
            } else {
                let buffer = self.use_buffer(descriptor, BufferTarget::UniformBuffer)?;
                self.gl
                    .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
                match offset_and_size {
                    Some((offset, size)) => self.gl.bind_buffer_range_with_i32_and_i32(
                        WebGl2RenderingContext::UNIFORM_BUFFER,
                        binding,
                        Some(&buffer),
                        offset as i32,
                        size as i32,
                    ),
                    None => self.gl.bind_buffer_base(
                        WebGl2RenderingContext::UNIFORM_BUFFER,
                        binding,
                        Some(&buffer),
                    ),
                };
                self.gl
                    .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);
                (*self.ubo_bindings).insert(binding, descriptor.id());
            };
        }

        Ok(())
    }

    /// Unbinds a uniform buffer object at mount point.
    pub fn unbind_uniform_buffer_object(&mut self, binding: u32) {
        unsafe {
            let Some(id) = (*self.ubo_bindings).remove(&binding) else {
                return;
            };
            let Some(item) = (*self.store).get_mut(&id) else {
                return;
            };
            self.gl
                .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, binding, None);
            item.using = false;
        }
    }
}
