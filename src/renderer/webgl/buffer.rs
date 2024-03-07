use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
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
    renderer::webgl::params::GetWebGlParameters,
    utils::format_bytes_length,
};

use super::{
    conversion::ToGlEnum,
    error::Error,
    params::{self},
};

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

struct RegisteredStore {
    lru_node: *mut LruNode<Uuid>,
    used_memory: *mut usize,
    items: *mut HashMap<
        Uuid,
        (
            Weak<RefCell<Queue>>,
            Weak<RefCell<MemoryPolicy>>,
            Weak<RefCell<Runtime>>,
        ),
    >,
    bindings: *mut HashMap<BufferTarget, Uuid>,
    binding_ubos: *mut HashMap<u32, Uuid>,
    lru: *mut Lru<Uuid>,
}

impl RegisteredStore {
    fn 
}

struct Runtime {
    gl: WebGl2RenderingContext,
    buffer: Option<WebGlBuffer>,
    bytes_length: usize,
    bindings: HashSet<BufferTarget>,
    binding_ubos: HashSet<u32>,
}

impl Runtime {
    fn read_back(&self) -> ArrayBuffer {
        let Some(buffer) = self.buffer.as_ref() else {
            return ArrayBuffer::new(0);
        };
        if self.bytes_length == 0 {
            return ArrayBuffer::new(0);
        }

        let gl = &self.gl;
        let data = ArrayBuffer::new(self.bytes_length as u32);
        let binding = gl.array_buffer_binding();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            &data,
        );
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        data
    }

    fn get_or_create_buffer(&mut self) -> Result<&WebGlBuffer, Error> {
        match self.buffer.as_ref() {
            Some(buffer) => Ok(buffer),
            None => {
                let buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                Ok(self.buffer.insert(buffer))
            }
        }
    }
}

struct QueueItem {
    source: BufferSource,
    bytes_offset: usize,
}

impl QueueItem {
    fn new(source: BufferSource, bytes_offset: usize) -> Self {
        Self {
            source,
            bytes_offset,
        }
    }
}

struct Queue {
    max_bytes_length: usize,
    items: Vec<QueueItem>,
}

impl Queue {
    fn new() -> Self {
        Self {
            max_bytes_length: 0,
            items: Vec::new(),
        }
    }
}

pub struct Buffer {
    id: Uuid,
    name: Option<Cow<'static, str>>,
    usage: BufferUsage,
    memory_policy: MemoryPolicy,
    queue: Queue,

    store: Option<RegisteredStore>,
    runtime: Option<Rc<RefCell<Runtime>>>,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.clear(false);
    }
}

impl Buffer {
    /// Constructs a new buffer descriptor with specified name, [`BufferSource`], [`BufferUsage`] and [`MemoryPolicy`].
    pub fn new(
        name: Option<Cow<'static, str>>,
        usage: BufferUsage,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            usage,
            memory_policy,
            queue: Queue::new(),

            runtime: None,
            store: None,
        }
    }

    /// Returns id of this buffer.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns buffer descriptor name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets buffer descriptor name.
    pub fn set_name(&mut self, name: Option<Cow<'static, str>>) {
        self.name = name;
    }

    /// Returns [`BufferUsage`].
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    /// Returns current [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy {
        &self.memory_policy
    }

    /// Sets the [`MemoryPolicy`] of this buffer descriptor.
    pub fn set_memory_policy(&mut self, memory_policy: MemoryPolicy) {
        self.memory_policy = memory_policy;
    }

    /// Initializes this buffer by a [`WebGl2RenderingContext`].
    pub fn init(&mut self, gl: WebGl2RenderingContext) -> Result<(), Error> {
        match &self.runtime {
            Some(runtime) => {
                if &runtime.borrow().gl == &gl {
                    Ok(())
                } else {
                    Err(Error::BufferAlreadyInitialized)
                }
            }
            None => {
                let runtime = Runtime {
                    gl,
                    buffer: None,
                    bytes_length: 0,
                    bindings: HashSet::new(),
                    binding_ubos: HashSet::new(),
                };
                self.runtime = Some(Rc::new(RefCell::new(runtime)));
                Ok(())
            }
        }
    }

    fn runtime(&self) -> Result<&Rc<RefCell<Runtime>>, Error> {
        self.runtime
            .as_ref()
            .map(|runtime| runtime)
            .ok_or(Error::BufferUninitialized)
    }

    fn store(&self) -> Option<&RegisteredStore> {
        self.store.as_ref()
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind(&self, target: BufferTarget) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let mut runtime = runtime.borrow_mut();

        let buffer = runtime.get_or_create_buffer()?;
        runtime.gl.bind_buffer(target.gl_enum(), Some(buffer));
        runtime.bindings.insert(target);

        self.upload_unchecked(target, &mut *runtime);

        Ok(())
    }

    /// Unbinds buffer at specified [`BufferTarget`].
    pub fn unbind(&self, target: BufferTarget) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let mut runtime = runtime.borrow_mut();

        if runtime.bindings.remove(&target) {
            runtime.gl.bind_buffer(target.gl_enum(), None);
        }

        Ok(())
    }

    /// Unbinds buffer from specified uniform buffer object index.
    pub fn unbind_ubo(&self, index: u32) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let mut runtime = runtime.borrow_mut();

        if runtime.binding_ubos.remove(&index) {
            runtime
                .gl
                .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);
        }

        Ok(())
    }

    /// Unbinds buffer from all bindings, including uniform buffer objects.
    pub fn unbind_all(&self) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let mut runtime = runtime.borrow_mut();

        let gl = &runtime.gl;
        for index in runtime.binding_ubos.drain() {
            gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);
        }
        for target in runtime.bindings.drain() {
            gl.bind_buffer(target.gl_enum(), None);
        }

        Ok(())
    }

    fn upload_unchecked(&mut self, target: BufferTarget, runtime: &mut Runtime) {
        if self.queue.items.len() > 0 {
            let gl = &runtime.gl;
            let new_bytes_length = self.queue.max_bytes_length;
            let old_bytes_length = runtime.bytes_length;

            if new_bytes_length >= old_bytes_length {
                gl.buffer_data_with_i32(
                    target.gl_enum(),
                    new_bytes_length as i32,
                    self.usage.gl_enum(),
                );
            }

            for item in self.queue.items.drain(..) {
                item.source.buffer_sub_data(gl, target, item.bytes_offset);
            }

            runtime.bytes_length = new_bytes_length;

            debug!(
                target: "BufferStore",
                "buffer new data for {}, old length {}, new length {}",
                self.name().unwrap_or("unnamed"),
                old_bytes_length,
                new_bytes_length
            );
        }
    }

    /// Uploads data to WebGL runtime.
    pub fn upload(&mut self) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let mut runtime = runtime.borrow_mut();

        let binding = runtime.gl.array_buffer_binding();
        runtime.gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(runtime.get_or_create_buffer()?),
        );
        self.upload_unchecked(BufferTarget::ARRAY_BUFFER, &mut *runtime);
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        Ok(())
    }

    /// Clears and unbinds buffer from WebGL runtime.
    /// Data will be read back from WebGL runtime and
    /// insert to the first place of the queue if `read_back` is `true`.
    pub fn clear(&mut self, read_back: bool) -> Result<(), Error> {
        let runtime = self.runtime()?;
        let runtime = runtime.borrow_mut();

        self.queue.items.clear();
        self.queue.max_bytes_length = 0;

        if read_back {
            let data = runtime.read_back();
            if data.byte_length() != 0 {
                self.queue
                    .items
                    .push(QueueItem::new(BufferSource::from_array_buffer(data), 0));
            }
        }

        let gl = &runtime.gl;
        if let Some(buffer) = runtime.buffer.take() {
            for index in runtime.binding_ubos.drain() {
                gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);
            }
            for target in runtime.bindings.drain() {
                gl.bind_buffer(target.gl_enum(), None);
            }
            gl.delete_buffer(Some(&buffer))
        }
        runtime.bytes_length = 0;

        Ok(())
    }

    /// Reads buffer data back from WebGL runtime and stores it to an [`ArrayBuffer`].
    pub fn read_back(&self) -> Result<ArrayBuffer, Error> {
        Ok(self.runtime()?.borrow().read_back())
    }

    /// Overrides existing data and then buffers new data.
    pub fn buffer_data(&mut self, source: BufferSource) {
        self.queue.max_bytes_length = source.bytes_length();
        self.queue.items.clear();
        self.queue.items.push(QueueItem::new(source, 0));
    }

    /// Buffers sub data.
    pub fn buffer_sub_data(&mut self, source: BufferSource, dst_byte_offset: usize) {
        let bytes_length = dst_byte_offset + source.bytes_length();
        if dst_byte_offset == 0 {
            if bytes_length >= self.queue.max_bytes_length {
                // overrides sources in queue if new source covers all
                self.queue.max_bytes_length = bytes_length;
                self.queue.items.clear();
                self.queue.items.push(QueueItem::new(source, 0));
            } else {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            }
        } else {
            if bytes_length <= self.queue.max_bytes_length {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            } else {
                if let Some(runtime) = self.runtime.as_ref().map(|runtime| runtime.borrow()) {
                    // heavy job!
                    let data = runtime.read_back();

                    self.queue.items.insert(
                        0,
                        QueueItem::new(BufferSource::preallocate(bytes_length), 0),
                    );
                    self.queue
                        .items
                        .insert(1, QueueItem::new(BufferSource::from_array_buffer(data), 0));
                    self.queue
                        .items
                        .push(QueueItem::new(source, dst_byte_offset));
                } else {
                    self.queue.items.insert(
                        0,
                        QueueItem::new(BufferSource::preallocate(bytes_length), 0),
                    );
                    self.queue
                        .items
                        .push(QueueItem::new(source, dst_byte_offset));
                }
            }
        }
    }
}

pub trait Restorer {
    fn restore(&self) -> BufferSource;
}

/// Memory freeing policies.
pub enum MemoryPolicy {
    Unfree,
    ReadBack,
    Restorable(Rc<RefCell<dyn Restorer>>),
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
    pub fn restorable<R>(restorer: R) -> Self
    where
        R: Restorer + 'static,
    {
        Self::Restorable(Rc::new(RefCell::new(restorer)))
    }
}

pub struct BufferStore {
    id: Uuid,
    gl: WebGl2RenderingContext,
    available_memory: usize,
    used_memory: *mut usize,
    items: *mut HashMap<
        Uuid,
        (
            Weak<RefCell<Queue>>,
            Weak<RefCell<MemoryPolicy>>,
            Weak<RefCell<Runtime>>,
        ),
    >,
    bindings: *mut HashMap<BufferTarget, Uuid>,
    binding_ubos: *mut HashMap<u32, Uuid>,
    lru: *mut Lru<Uuid>,
}

// impl Drop for BufferStore {
//     fn drop(&mut self) {
//         unsafe {
//             for (queue, _, runtime) in (*self.descriptors).values() {
//                 let (Some(queue), Some(runtime)) = (queue.upgrade(), runtime.upgrade()) else {
//                     return;
//                 };
//                 let mut queue = queue.borrow_mut();
//                 let runtime = runtime.borrow();

//                 let current_binding = array_buffer_binding(&self.gl);

//                 let data = Uint8Array::new_with_length(runtime.bytes_length as u32);
//                 self.gl
//                     .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&runtime.buffer));
//                 self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
//                     WebGl2RenderingContext::ARRAY_BUFFER,
//                     0,
//                     &data,
//                 );
//                 self.gl.bind_buffer(
//                     WebGl2RenderingContext::ARRAY_BUFFER,
//                     current_binding.as_ref(),
//                 );
//                 self.gl.delete_buffer(Some(&runtime.buffer));

//                 queue.items.insert(
//                     0,
//                     QueueItem::new(
//                         BufferSource::from_uint8_array(data, 0, runtime.bytes_length),
//                         0,
//                     ),
//                 );
//                 queue.max_bytes_length = queue.max_bytes_length.max(runtime.bytes_length);
//                 // store dropped, no need to update LRU anymore
//             }

//             drop(Box::from_raw(self.used_memory));
//             drop(Box::from_raw(self.ubos));
//             drop(Box::from_raw(self.lru));
//             drop(Box::from_raw(self.descriptors));
//         }
//     }
// }

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
            available_memory: available_memory.min(i32::MAX as usize),
            used_memory: Box::leak(Box::new(0)),
            items: Box::leak(Box::new(HashMap::new())),
            bindings: Box::leak(Box::new(HashMap::new())),
            binding_ubos: Box::leak(Box::new(HashMap::new())),
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

    /// Registers a buffer to buffer store.
    pub fn register(&mut self, buffer: &mut Buffer) {
        unsafe {
            (*self.used_memory) += buffer
                .runtime
                .as_ref()
                .map(|runtime| runtime.borrow().bytes_length)
                .unwrap_or(0);
            buffer.store = Some(RegisteredStore {
                lru_node: LruNode::new(buffer.id),
                used_memory: self.used_memory,
                items: self.items,
                bindings: self.bindings,
                binding_ubos: self.binding_ubos,
                lru: self.lru,
            });
        }
    }

    pub fn unregister(&mut self, buffer: &mut Buffer) {
        unsafe {
            (*self.used_memory) -= buffer
                .runtime
                .as_ref()
                .map(|runtime| runtime.borrow().bytes_length)
                .unwrap_or(0);
            buffer.store = None;
        }
    }

    // /// Uses a [`WebGlBuffer`] by a [`BufferDescriptor`] and buffer data to it if necessary.
    // /// Remembers to calls [`BufferStore::unuse_buffer`] after using the [`WebGlBuffer`],
    // /// or the [`WebGlBuffer`] will never be freed.
    // pub fn use_buffer(
    //     &mut self,
    //     descriptor: &Buffer,
    //     target: BufferTarget,
    // ) -> Result<WebGlBuffer, Error> {
    //     unsafe {
    //         let Buffer {
    //             name,
    //             usage,
    //             memory_policy,
    //             queue,
    //             runtime,
    //             ..
    //         } = descriptor;

    //         let mut runtime = runtime.borrow_mut();
    //         let runtime = match runtime.as_ref() {
    //             Some(runtime) => {
    //                 if runtime.borrow().store_id != self.id {
    //                     panic!("share buffer descriptor between buffer store is not allowed");
    //                 }
    //                 runtime
    //             }
    //             None => {
    //                 debug!(
    //                     target: "BufferStore",
    //                     "create new buffer for {}",
    //                     name.as_deref().unwrap_or("unnamed"),
    //                 );

    //                 let buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
    //                 let id = self.next();
    //                 let r = Rc::new(RefCell::new(Runtime {
    //                     id,
    //                     gl: self.gl.clone(),
    //                     bindings: false,
    //                     buffer,
    //                     bytes_length: 0,
    //                     binding_ubos: HashSet::new(),
    //                     lru_node: LruNode::new(id),

    //                     store_id: self.id,
    //                     used_memory: self.used_memory,
    //                     items: self.descriptors,
    //                     ubos: self.ubos,
    //                     lru: self.lru,
    //                 }));
    //                 (*self.descriptors).insert(
    //                     id,
    //                     (
    //                         Rc::downgrade(queue),
    //                         Rc::downgrade(memory_policy),
    //                         Rc::downgrade(&r),
    //                     ),
    //                 );
    //                 runtime.insert(r)
    //             }
    //         };

    //         // update buffer data
    //         let mut runtime = runtime.borrow_mut();
    //         let mut queue = queue.borrow_mut();
    //         if queue.items.len() > 0 {
    //             let new_bytes_length = queue.max_bytes_length;
    //             let old_bytes_length = runtime.bytes_length;

    //             self.gl.bind_buffer(target.gl_enum(), Some(&runtime.buffer));
    //             if queue.items[0].bytes_offset == 0
    //                 && new_bytes_length == queue.items[0].source.bytes_length()
    //             {
    //                 let item = queue.items.remove(0);
    //                 item.source.buffer_data(&self.gl, target, *usage);
    //             } else if new_bytes_length > old_bytes_length {
    //                 self.gl.buffer_data_with_i32(
    //                     target.gl_enum(),
    //                     new_bytes_length as i32,
    //                     usage.gl_enum(),
    //                 );
    //             }
    //             for item in queue.items.drain(..) {
    //                 item.source
    //                     .buffer_sub_data(&self.gl, target, item.bytes_offset);
    //             }

    //             let new_bytes_length = self
    //                 .gl
    //                 .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
    //                 .as_f64()
    //                 .map(|size| size as usize)
    //                 .unwrap();
    //             (*self.used_memory) = (*self.used_memory) - old_bytes_length + new_bytes_length;
    //             runtime.bytes_length = new_bytes_length;

    //             debug!(
    //                 target: "BufferStore",
    //                 "buffer new data for {}, old length {}, new length {}",
    //                 name.as_deref().unwrap_or("unnamed"),
    //                 old_bytes_length,
    //                 new_bytes_length
    //             );

    //             self.gl.bind_buffer(target.gl_enum(), None);
    //         }

    //         runtime.binding = true;
    //         (*self.lru).cache(runtime.lru_node);
    //         let buffer = runtime.buffer.clone();

    //         self.free(target);

    //         Ok(buffer)
    //     }
    // }

    // /// Unuses a [`WebGlBuffer`] by a [`BufferDescriptor`].
    // pub fn unuse_buffer(&mut self, descriptor: &Buffer) {
    //     let runtime = descriptor.runtime.borrow_mut();
    //     let Some(runtime) = runtime.as_ref() else {
    //         return;
    //     };
    //     runtime.borrow_mut().binding = false;
    // }

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
                let Entry::Occupied(occupied) = (*self.items).entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let (queue, memory_policy, runtime) = occupied.get();
                let (Some(_), Some(memory_policy), Some(runtime)) =
                    (queue.upgrade(), memory_policy.upgrade(), runtime.upgrade())
                else {
                    // deletes if already dropped
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };

                let runtime = runtime.borrow();
                // skips if using
                if runtime.bindings || runtime.binding_ubos.len() != 0 {
                    next_node = (*current_node).more_recently();
                    continue;
                }
                // skips if unfree
                if let MemoryPolicy::Unfree = *memory_policy.borrow() {
                    next_node = (*current_node).more_recently();
                    continue;
                }

                // free
                let (queue, memory_policy, runtime) = occupied.remove();
                let (queue, memory_policy, runtime) = (
                    queue.upgrade().unwrap(),
                    memory_policy.upgrade().unwrap(),
                    runtime.upgrade().unwrap(),
                );
                let runtime = runtime.borrow();
                match &*memory_policy.borrow() {
                    MemoryPolicy::Unfree => unreachable!(),
                    MemoryPolicy::ReadBack => {
                        let mut queue = queue.borrow_mut();

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

                        queue.items.insert(
                            0,
                            QueueItem::new(
                                BufferSource::from_uint8_array(data, 0, runtime.bytes_length),
                                0,
                            ),
                        );
                        queue.max_bytes_length = queue.max_bytes_length.max(runtime.bytes_length);
                        debug!(
                            target: "BufferStore",
                            "free buffer (default) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(runtime.bytes_length),
                            format_bytes_length(*self.used_memory)
                        );
                    }
                    MemoryPolicy::Restorable(restorer) => {
                        let mut queue = queue.borrow_mut();

                        self.gl.delete_buffer(Some(&runtime.buffer));
                        let restorer = Rc::clone(&restorer);
                        let source = BufferSource::Function {
                            callback: Box::new(move || restorer.borrow_mut().restore()),
                            data_length: runtime.bytes_length,
                            src_offset: 0,
                            src_length: runtime.bytes_length,
                        };

                        queue.items.insert(0, QueueItem::new(source, 0));
                        queue.max_bytes_length = queue.max_bytes_length.max(runtime.bytes_length);
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

    // pub fn bind_uniform_buffer_object(
    //     &mut self,
    //     descriptor: &Buffer,
    //     ubo_binding: u32,
    //     offset_and_size: Option<(usize, usize)>,
    // ) -> Result<(), Error> {
    //     unsafe {
    //         let bounded = (*self.ubos).contains_key(&ubo_binding);
    //         let mut runtime = descriptor.runtime.borrow_mut();
    //         let buffer = match (bounded, runtime.as_mut()) {
    //             (true, None) => {
    //                 return Err(Error::UniformBufferObjectIndexAlreadyBound(ubo_binding))?;
    //             }
    //             (true, Some(runtime)) => {
    //                 if &runtime.borrow().store_id != &self.id {
    //                     return Err(Error::UniformBufferObjectIndexAlreadyBound(ubo_binding));
    //                 }

    //                 drop(runtime);
    //                 self.use_buffer(descriptor, BufferTarget::UNIFORM_BUFFER)?;
    //                 return Ok(());
    //             }
    //             (false, _) => {
    //                 drop(runtime);
    //                 let buffer = self.use_buffer(descriptor, BufferTarget::UNIFORM_BUFFER)?;
    //                 buffer
    //             }
    //         };

    //         let runtime = descriptor.runtime.borrow_mut();
    //         let mut runtime = runtime.as_ref().unwrap().borrow_mut();
    //         let binding = params::uniform_buffer_binding(&self.gl);
    //         self.gl
    //             .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
    //         match offset_and_size {
    //             Some((offset, size)) => self.gl.bind_buffer_range_with_i32_and_i32(
    //                 WebGl2RenderingContext::UNIFORM_BUFFER,
    //                 ubo_binding,
    //                 Some(&buffer),
    //                 offset as i32,
    //                 size as i32,
    //             ),
    //             None => self.gl.bind_buffer_base(
    //                 WebGl2RenderingContext::UNIFORM_BUFFER,
    //                 ubo_binding,
    //                 Some(&buffer),
    //             ),
    //         };
    //         self.gl
    //             .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, binding.as_ref());

    //         (*self.ubos).insert(ubo_binding, runtime.id);
    //         runtime.binding_ubos.insert(ubo_binding);

    //         Ok(())
    //     }
    // }

    // /// Unbinds a uniform buffer object at mount point.
    // pub fn unbind_uniform_buffer_object(&mut self, binding: u32) {
    //     unsafe {
    //         let Some(id) = (*self.ubos).remove(&binding) else {
    //             return;
    //         };
    //         let Some((_, _, runtime)) = (*self.descriptors).get(&id) else {
    //             return;
    //         };
    //         let Some(runtime) = runtime.upgrade() else {
    //             return;
    //         };

    //         let mut runtime = runtime.borrow_mut();
    //         if runtime.binding_ubos.remove(&binding) {
    //             self.gl
    //                 .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, binding, None);
    //         }
    //         runtime.bindings = runtime.binding_ubos.len() != 0;
    //     }
    // }
}
