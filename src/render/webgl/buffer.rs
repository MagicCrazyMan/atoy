use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
    hash::Hash,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::debug;
use uuid::Uuid;
use web_sys::{
    js_sys::{
        ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array,
        Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array,
        Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::{
    lru::{Lru, LruNode},
    utils::format_bytes_length,
};

use super::{conversion::ToGlEnum, error::Error, utils::array_buffer_binding};

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    ARRAY_BUFFER,
    ELEMENT_ARRAY_BUFFER,
    COPY_READ_BUFFER,
    COPY_WRITE_BUFFER,
    TRANSFORM_FEEDBACK_BUFFER,
    UNIFORM_BUFFER,
    PIXEL_PACK_BUFFER,
    PIXEL_UNPACK_BUFFER,
}

/// Available component size of a value get from buffer.
/// According to WebGL definition, it should only be `1`, `2`, `3` or `4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
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
    FLOAT,
    BYTE,
    SHORT,
    INT,
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
    HALF_FLOAT,
    INT_2_10_10_10_REV,
    UNSIGNED_INT_2_10_10_10_REV,
}

impl BufferDataType {
    /// Gets bytes length of a data type.
    pub fn bytes_length(&self) -> usize {
        match self {
            BufferDataType::FLOAT => 4,
            BufferDataType::BYTE => 1,
            BufferDataType::SHORT => 2,
            BufferDataType::INT => 4,
            BufferDataType::UNSIGNED_BYTE => 1,
            BufferDataType::UNSIGNED_SHORT => 2,
            BufferDataType::UNSIGNED_INT => 4,
            BufferDataType::HALF_FLOAT => 2,
            BufferDataType::INT_2_10_10_10_REV => 4,
            BufferDataType::UNSIGNED_INT_2_10_10_10_REV => 4,
        }
    }
}

/// Available buffer usages mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    STATIC_DRAW,
    DYNAMIC_DRAW,
    STREAM_DRAW,
    STATIC_READ,
    DYNAMIC_READ,
    STREAM_READ,
    STATIC_COPY,
    DYNAMIC_COPY,
    STREAM_COPY,
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
    DataView {
        data: DataView,
        src_offset: usize,
        src_length: usize,
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
            BufferSource::DataView {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
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
            BufferSource::Function { callback, .. } => {
                let source = callback();
                if let BufferSource::Function { .. } = source {
                    panic!("recursive BufferSource::Function is not allowed");
                }
                if self.bytes_length() != source.bytes_length() {
                    panic!(
                        "source returned from BufferSource::Function should have same bytes length"
                    );
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
                if self.bytes_length() != source.bytes_length() {
                    panic!(
                        "source returned from BufferSource::Function should have same bytes length"
                    );
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
            BufferSource::ArrayBuffer { data } => {
                log::info!("{} {}", data.byte_length(), dst_byte_offset);
                gl.buffer_sub_data_with_i32_and_array_buffer(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    data,
                )
            }
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
            BufferSource::DataView {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
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
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
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

struct Runtime {
    gl: WebGl2RenderingContext,
    id: usize,
    buffer: WebGlBuffer,
    bytes_length: usize,
    binding: bool,
    binding_ubos: HashSet<u32>,
    lru_node: *mut LruNode<usize>,

    store_id: Uuid,
    used_memory: *mut usize,
    items: *mut HashMap<usize, Weak<RefCell<BufferDescriptorInner>>>,
    ubos: *mut HashMap<u32, usize>,
    lru: *mut Lru<usize>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            (*self.used_memory) -= self.bytes_length;
            (*self.lru).remove(self.lru_node);
            (*self.items).remove(&self.id);
            for ubo in self.binding_ubos.iter() {
                (*self.ubos).remove(ubo);
            }
            self.gl.delete_buffer(Some(&self.buffer));
        }
    }
}

struct BufferDescriptorInner {
    name: Option<Cow<'static, str>>,
    usage: BufferUsage,
    memory_policy: MemoryPolicy,

    queue_bytes_length: usize,
    queue: Vec<(BufferSource, usize)>,

    runtime: Option<Box<Runtime>>,
}

/// A key to share and control the [`WebGlBuffer`].
/// Checks [`BufferStore`] for more details.
#[derive(Clone)]
pub struct BufferDescriptor(Rc<RefCell<BufferDescriptorInner>>);

impl BufferDescriptor {
    /// Constructs a new buffer descriptor with specified [`BufferSource`] and [`BufferUsage`].
    pub fn new(source: BufferSource, usage: BufferUsage) -> Self {
        Self::with_memory_policy(source, usage, MemoryPolicy::ReadBack)
    }

    /// Constructs a new buffer descriptor with specified [`BufferSource`], [`BufferUsage`] and [`MemoryPolicy`].
    pub fn with_memory_policy(
        source: BufferSource,
        usage: BufferUsage,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self(Rc::new(RefCell::new(BufferDescriptorInner {
            name: None,
            usage,
            memory_policy,

            queue_bytes_length: source.bytes_length(),
            queue: Vec::from([(source, 0)]),

            runtime: None,
        })))
    }

    /// Returns buffer descriptor name.
    pub fn name(&self) -> Ref<Option<Cow<'static, str>>> {
        Ref::map(self.0.borrow(), |inner| &inner.name)
    }

    /// Sets buffer descriptor name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.0.borrow_mut().name.replace(Cow::Owned(name.into()));
    }

    /// Sets buffer descriptor name.
    pub fn set_name_str(&mut self, name: &'static str) {
        self.0.borrow_mut().name.replace(Cow::Borrowed(name));
    }

    /// Returns [`BufferUsage`].
    pub fn usage(&self) -> BufferUsage {
        self.0.borrow().usage
    }

    pub fn memory_policy(&self) -> Ref<MemoryPolicy> {
        Ref::map(self.0.borrow(), |inner| &inner.memory_policy)
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
        if dst_byte_offset == 0 {
            if bytes_length >= inner.queue_bytes_length {
                // overrides sources in queue if new source covers all
                inner.queue_bytes_length = bytes_length;
                inner.queue.clear();
                inner.queue.push((source, 0));
            } else {
                inner.queue.push((source, dst_byte_offset));
            }
        } else {
            if bytes_length <= inner.queue_bytes_length {
                inner.queue.push((source, dst_byte_offset));
            } else {
                if let Some(runtime) = inner.runtime.as_deref_mut() {
                    // heavy job!
                    let current_binding = array_buffer_binding(&runtime.gl);

                    let bytes_length = runtime.bytes_length;
                    let data = Uint8Array::new_with_length(bytes_length as u32);
                    runtime
                        .gl
                        .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&runtime.buffer));
                    runtime
                        .gl
                        .get_buffer_sub_data_with_i32_and_array_buffer_view(
                            WebGl2RenderingContext::ARRAY_BUFFER,
                            0,
                            &data,
                        );
                    runtime.gl.bind_buffer(
                        WebGl2RenderingContext::ARRAY_BUFFER,
                        current_binding.as_ref(),
                    );

                    inner
                        .queue
                        .insert(0, (BufferSource::preallocate(bytes_length), 0));
                    inner.queue.insert(
                        1,
                        (BufferSource::from_uint8_array(data, 0, bytes_length), 0),
                    );
                    inner.queue.push((source, dst_byte_offset));
                } else {
                    inner
                        .queue
                        .insert(0, (BufferSource::preallocate(bytes_length), 0));
                    inner.queue.push((source, dst_byte_offset));
                }
            }
        }
    }
}

/// Memory freeing policies.
pub enum MemoryPolicy {
    Unfree,
    ReadBack,
    Restorable(Rc<RefCell<dyn Fn() -> BufferSource>>),
}

impl Default for MemoryPolicy {
    fn default() -> Self {
        Self::ReadBack
    }
}

impl MemoryPolicy {
    /// Constructs a unfree-able memory policy.
    pub fn unfree() -> Self {
        Self::Unfree
    }

    /// Constructs a read back memory policy.
    pub fn read_back() -> Self {
        Self::ReadBack
    }

    /// Constructs a restorable memory policy.
    pub fn restorable<F>(f: F) -> Self
    where
        F: Fn() -> BufferSource + 'static,
    {
        Self::Restorable(Rc::new(RefCell::new(f)))
    }
}

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    id: Uuid,
    counter: usize,
    available_memory: usize,
    used_memory: *mut usize,
    descriptors: *mut HashMap<usize, Weak<RefCell<BufferDescriptorInner>>>,
    ubos: *mut HashMap<u32, usize>,
    lru: *mut Lru<usize>,
}

impl Drop for BufferStore {
    fn drop(&mut self) {
        unsafe {
            for (_, descriptor) in (*self.descriptors).iter() {
                let Some(descriptor) = descriptor.upgrade() else {
                    return;
                };
                let mut descriptor = descriptor.borrow_mut();
                let Some(runtime) = descriptor.runtime.take() else {
                    return;
                };

                let current_binding = array_buffer_binding(&self.gl);

                let data = Uint8Array::new_with_length(runtime.bytes_length as u32);
                self.gl
                    .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&runtime.buffer));
                self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &data,
                );
                self.gl.bind_buffer(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    current_binding.as_ref(),
                );
                self.gl.delete_buffer(Some(&runtime.buffer));

                descriptor.queue.insert(
                    0,
                    (
                        BufferSource::from_uint8_array(data, 0, runtime.bytes_length),
                        0,
                    ),
                );
                descriptor.queue_bytes_length =
                    descriptor.queue_bytes_length.max(runtime.bytes_length);
                // store dropped, no need to update LRU anymore
            }

            drop(Box::from_raw(self.used_memory));
            drop(Box::from_raw(self.ubos));
            drop(Box::from_raw(self.lru));
            drop(Box::from_raw(self.descriptors));
        }
    }
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited memory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_available_memory(gl, i32::MAX as usize)
    }

    /// Constructs a new buffer store with a maximum available memory.
    /// Maximum available memory is clamped to [`i32::MAX`] if larger than [`i32::MAX`];
    pub fn with_available_memory(gl: WebGl2RenderingContext, available_memory: usize) -> Self {
        Self {
            gl,
            id: Uuid::new_v4(),
            counter: 0,
            available_memory: available_memory.min(i32::MAX as usize),
            used_memory: Box::leak(Box::new(0)),
            descriptors: Box::leak(Box::new(HashMap::new())),
            ubos: Box::leak(Box::new(HashMap::new())),
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

    unsafe fn next(&mut self) -> usize {
        if (*self.descriptors).len() == usize::MAX {
            panic!("too many descriptors, only {} are accepted", usize::MAX);
        }

        self.counter = self.counter.wrapping_add(1);
        while (*self.descriptors).contains_key(&self.counter) {
            self.counter = self.counter.wrapping_add(1);
        }
        self.counter
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
            let mut inner = descriptor.0.borrow_mut();
            let BufferDescriptorInner {
                name,
                usage,
                queue_bytes_length,
                queue,
                runtime,
                ..
            } = &mut *inner;

            let runtime = match runtime {
                Some(runtime) => {
                    if runtime.store_id != self.id {
                        panic!("share buffer descriptor between buffer store is not allowed");
                    }
                    runtime
                }
                None => {
                    debug!(
                        target: "BufferStore",
                        "create new buffer for {}",
                        name.as_deref().unwrap_or("unnamed"),
                    );

                    let buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                    let id = self.next();
                    (*self.descriptors).insert(id, Rc::downgrade(&descriptor.0));
                    runtime.insert(Box::new(Runtime {
                        id,
                        gl: self.gl.clone(),
                        binding: false,
                        buffer,
                        bytes_length: 0,
                        binding_ubos: HashSet::new(),
                        lru_node: LruNode::new(id),

                        store_id: self.id,
                        used_memory: self.used_memory,
                        items: self.descriptors,
                        ubos: self.ubos,
                        lru: self.lru,
                    }))
                }
            };

            // update buffer data
            if queue.len() > 0 {
                let new_bytes_length = *queue_bytes_length;
                let old_bytes_length = runtime.bytes_length;

                self.gl.bind_buffer(target.gl_enum(), Some(&runtime.buffer));
                if queue[0].1 == 0 && new_bytes_length == queue[0].0.bytes_length() {
                    let (source, _) = queue.remove(0);
                    source.buffer_data(&self.gl, target, *usage);
                } else if new_bytes_length > old_bytes_length {
                    self.gl.buffer_data_with_i32(
                        target.gl_enum(),
                        new_bytes_length as i32,
                        usage.gl_enum(),
                    );
                }
                for (source, dst_byte_offset) in queue.drain(..) {
                    source.buffer_sub_data(&self.gl, target, dst_byte_offset);
                }

                let new_bytes_length = self
                    .gl
                    .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                    .as_f64()
                    .map(|size| size as usize)
                    .unwrap();
                (*self.used_memory) = (*self.used_memory) - old_bytes_length + new_bytes_length;
                runtime.bytes_length = new_bytes_length;

                debug!(
                    target: "BufferStore",
                    "buffer new data for {}, old length {}, new length {}",
                    name.as_deref().unwrap_or("unnamed"),
                    old_bytes_length,
                    new_bytes_length
                );

                self.gl.bind_buffer(target.gl_enum(), None);
            }

            runtime.binding = true;
            (*self.lru).cache(runtime.lru_node);
            let buffer = runtime.buffer.clone();

            drop(inner);
            self.free(target);

            Ok(buffer)
        }
    }

    /// Unuses a [`WebGlBuffer`] by a [`BufferDescriptor`].
    pub fn unuse_buffer(&mut self, descriptor: &BufferDescriptor) {
        let mut descriptor = descriptor.0.borrow_mut();
        let Some(runtime) = descriptor.runtime.as_mut() else {
            return;
        };
        runtime.binding = false;
    }

    /// Frees memory if used memory exceeds the maximum available memory.
    fn free(&mut self, target: BufferTarget) {
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
                let Entry::Occupied(occupied) = (*self.descriptors).entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let descriptor = occupied.get();
                let Some(descriptor) = descriptor.upgrade() else {
                    // deletes if descriptor dropped
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };

                let descriptor = descriptor.borrow();
                let runtime = descriptor.runtime.as_ref().unwrap();
                // skips if using
                if runtime.binding || runtime.binding_ubos.len() != 0 {
                    next_node = (*current_node).more_recently();
                    continue;
                }
                // skips if unfree
                if let MemoryPolicy::Unfree = &descriptor.memory_policy {
                    next_node = (*current_node).more_recently();
                    continue;
                }

                // free
                let descriptor = occupied.remove().upgrade().unwrap();
                let mut descriptor = descriptor.borrow_mut();
                let runtime = descriptor.runtime.take().unwrap();
                match &descriptor.memory_policy {
                    MemoryPolicy::Unfree => unreachable!(),
                    MemoryPolicy::ReadBack => {
                        // default, gets buffer data back from WebGlBuffer
                        let data = Uint8Array::new_with_length(runtime.bytes_length as u32);
                        self.gl.bind_buffer(target.gl_enum(), Some(&runtime.buffer));
                        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                            target.gl_enum(),
                            0,
                            &data,
                        );
                        self.gl.bind_buffer(target.gl_enum(), None);
                        self.gl.delete_buffer(Some(&runtime.buffer));

                        descriptor.queue.insert(
                            0,
                            (
                                BufferSource::from_uint8_array(data, 0, runtime.bytes_length),
                                0,
                            ),
                        );
                        descriptor.queue_bytes_length =
                            descriptor.queue_bytes_length.max(runtime.bytes_length);
                        debug!(
                            target: "BufferStore",
                            "free buffer (default) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(runtime.bytes_length),
                            format_bytes_length(*self.used_memory)
                        );
                    }
                    MemoryPolicy::Restorable(restore) => {
                        self.gl.delete_buffer(Some(&runtime.buffer));
                        let restore = Rc::clone(&restore);
                        let source = BufferSource::Function {
                            callback: Box::new(move || restore.borrow_mut()()),
                            data_length: runtime.bytes_length,
                            src_offset: 0,
                            src_length: runtime.bytes_length,
                        };

                        descriptor.queue.insert(0, (source, 0));
                        descriptor.queue_bytes_length =
                            descriptor.queue_bytes_length.max(runtime.bytes_length);
                        debug!(
                            target: "BufferStore",
                            "free buffer (restorable) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(runtime.bytes_length),
                            format_bytes_length(*self.used_memory)
                        );
                    }
                }
                // reduces used memory
                (*self.used_memory) -= runtime.bytes_length;
                // removes LRU
                (*self.lru).remove(runtime.lru_node);

                next_node = (*current_node).more_recently();
            }
        }
    }

    pub fn bind_uniform_buffer_object(
        &mut self,
        descriptor: &BufferDescriptor,
        ubo_binding: u32,
        offset_and_size: Option<(usize, usize)>,
    ) -> Result<(), Error> {
        unsafe {
            let bounded = (*self.ubos).contains_key(&ubo_binding);
            let runtime = Ref::map(descriptor.0.borrow(), |inner| &inner.runtime);
            let (buffer, mut runtime) = match (bounded, runtime.as_deref()) {
                (true, None) => {
                    return Err(Error::UniformBufferObjectIndexAlreadyBound(ubo_binding))?;
                }
                (true, Some(Runtime { store_id, .. })) => {
                    if store_id != &self.id {
                        return Err(Error::UniformBufferObjectIndexAlreadyBound(ubo_binding));
                    } else {
                        drop(runtime);
                        self.use_buffer(descriptor, BufferTarget::UNIFORM_BUFFER)?;
                        return Ok(());
                    }
                }
                (false, _) => {
                    drop(runtime);
                    let buffer = self.use_buffer(descriptor, BufferTarget::UNIFORM_BUFFER)?;
                    let runtime =
                        RefMut::map(descriptor.0.borrow_mut(), |inner| &mut inner.runtime);
                    (buffer, runtime)
                }
            };

            let runtime = runtime.as_deref_mut().unwrap();
            self.gl
                .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
            match offset_and_size {
                Some((offset, size)) => self.gl.bind_buffer_range_with_i32_and_i32(
                    WebGl2RenderingContext::UNIFORM_BUFFER,
                    ubo_binding,
                    Some(&buffer),
                    offset as i32,
                    size as i32,
                ),
                None => self.gl.bind_buffer_base(
                    WebGl2RenderingContext::UNIFORM_BUFFER,
                    ubo_binding,
                    Some(&buffer),
                ),
            };
            self.gl
                .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);

            (*self.ubos).insert(ubo_binding, runtime.id);
            runtime.binding_ubos.insert(ubo_binding);

            Ok(())
        }
    }

    /// Unbinds a uniform buffer object at mount point.
    pub fn unbind_uniform_buffer_object(&mut self, binding: u32) {
        unsafe {
            let Some(id) = (*self.ubos).remove(&binding) else {
                return;
            };
            let Some(descriptor) = (*self.descriptors).get(&id) else {
                return;
            };
            let Some(descriptor) = descriptor.upgrade() else {
                return;
            };
            let mut descriptor = descriptor.borrow_mut();
            let runtime = descriptor.runtime.as_mut().unwrap();
            if runtime.binding_ubos.remove(&binding) {
                self.gl
                    .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, binding, None);
            }
            runtime.binding = runtime.binding_ubos.len() != 0;
        }
    }
}
