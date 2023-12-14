use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use log::{debug, warn};
use uuid::Uuid;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array,
        Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::{
    lru::{Lru, LruNode},
    utils::format_bytes_length,
};

use super::{
    conversion::{GLint, GLintptr, GLsizeiptr, GLuint, ToGlEnum},
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

/// Available size of a value get from buffer.
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
        size: GLsizeiptr,
    },
    Binary {
        data: Box<dyn AsRef<[u8]>>,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int8Array {
        data: Int8Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint8Array {
        data: Uint8Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int16Array {
        data: Int16Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint16Array {
        data: Uint16Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int32Array {
        data: Int32Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint32Array {
        data: Uint32Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Float32Array {
        data: Float32Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Float64Array {
        data: Float64Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    BigInt64Array {
        data: BigInt64Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
    BigUint64Array {
        data: BigUint64Array,
        src_offset: GLuint,
        src_length: GLuint,
    },
}

impl BufferSource {
    fn collect_typed_array_buffer(&self) -> (&Object, GLuint, GLuint) {
        match self {
            BufferSource::Preallocate { .. } | BufferSource::Binary { .. } => {
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
        let target = target.gl_enum();
        let usage = usage.gl_enum();
        match self {
            BufferSource::Preallocate { size } => gl.buffer_data_with_i32(target, *size, usage),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
            } => gl.buffer_data_with_u8_array_and_src_offset_and_length(
                target,
                data.as_ref().as_ref(),
                usage,
                *src_offset,
                *src_length,
            ),
            _ => {
                let (data, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_data_with_array_buffer_view_and_src_offset_and_length(
                    target, data, usage, src_offset, src_length,
                );
            }
        }
    }

    /// Buffers sub data to WebGL runtime.
    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: i32,
    ) {
        let target = target.gl_enum();
        match self {
            BufferSource::Preallocate { .. } => unreachable!(),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
            } => gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                target,
                dst_byte_offset,
                data.as_ref().as_ref(),
                *src_offset,
                *src_length,
            ),
            _ => {
                let (data, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target,
                    dst_byte_offset,
                    data,
                    src_offset,
                    src_length,
                )
            }
        }
    }

    fn bytes_length(&self) -> u32 {
        let (raw_length, src_offset, src_length) = match self {
            BufferSource::Preallocate { size } => (*size as u32, 0, 0),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.as_ref().as_ref().len() as u32,
                *src_offset,
                *src_length,
            ),
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
                *src_length,
            ),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length(),
                data.byte_offset() + *src_offset,
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
    pub fn preallocate(size: GLsizeiptr) -> Self {
        Self::Preallocate { size }
    }

    /// Constructs a new buffer source from WASM native buffer.
    pub fn from_binary<D: AsRef<[u8]> + 'static>(
        data: D,
        src_offset: GLuint,
        src_length: GLuint,
    ) -> Self {
        Self::Binary {
            data: Box::new(data),
            src_offset,
            src_length,
        }
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
                    src_offset: GLuint,
                    src_length: GLuint,
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

/// The final holder of the [`WebGlBuffer`] after the first use of a buffer descriptor.
///
/// [`BufferStore`] creates a [`Rc`] wrapped agency for
/// a descriptor for the first time descriptor being used.
/// Cloned descriptors share the same agency.
/// [`BufferStore`] always returns a same [`WebGlBuffer`] with a same agency.
///
/// After all referencing of an agency dropped, agency will drop [`WebGlBuffer`] automatically in [`Drop`].
#[derive(Clone)]
struct BufferAgency(Uuid, Weak<RefCell<StoreContainer>>, WebGl2RenderingContext);

impl BufferAgency {
    fn key(&self) -> &Uuid {
        &self.0
    }
}

impl PartialEq for BufferAgency {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BufferAgency {}

impl Hash for BufferAgency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Deletes associated WebGlBuffer from store(if exists) when descriptor id drops.
impl Drop for BufferAgency {
    fn drop(&mut self) {
        let Some(container) = self.1.upgrade() else {
            return;
        };
        let Ok(mut container) = (*container).try_borrow_mut() else {
            // if it is borrowed, buffer is dropping by store, skip
            return;
        };

        let Some(storage) = container.store.remove(&self.0) else {
            return;
        };
        let StorageItem {
            buffer,
            size,
            lru_node,
            id,
            ..
        } = &*storage.borrow();
        self.2.delete_buffer(Some(&buffer));
        container.used_memory -= size;

        unsafe {
            container.lru.remove(*lru_node);
        }

        debug!(target: "buffer_store", "drop buffer {}. freed memory {}, used {}", id, format_bytes_length(*size), format_bytes_length(container.used_memory));
    }
}

/// Buffer descriptor lifetime status.
///
/// Cloned descriptors share the same status.
enum BufferStatus {
    /// Buffer associated with this descriptor dropped already.
    Dropped,
    /// Buffer associated with this descriptor does not change.
    Unchanged { size: u32, agency: Rc<BufferAgency> },
    /// Buffers data into WebGL2 runtime.
    ///
    /// Drops source data after buffering into WebGL2 runtime.
    UpdateBuffer {
        source: BufferSource,
        subs: VecDeque<(BufferSource, GLintptr)>,
    },
    /// Buffers sub data into WebGL2 runtime.
    ///
    /// Drops source data after buffering into WebGL2 runtime.
    UpdateSubBuffer {
        size: u32,
        agency: Rc<BufferAgency>,
        overflow: bool,
        subs: VecDeque<(BufferSource, GLintptr)>,
    },
}

impl BufferStatus {
    /// Gets [`BufferDescriptorAgency`] associated with this buffer descriptor.
    fn agency(&self) -> Option<Rc<BufferAgency>> {
        match self {
            BufferStatus::Dropped => None,
            BufferStatus::Unchanged { agency, .. } => Some(Rc::clone(agency)),
            BufferStatus::UpdateBuffer { .. } => None,
            BufferStatus::UpdateSubBuffer { agency, .. } => Some(Rc::clone(agency)),
        }
    }
}

/// A key to share and control the [`WebGlBuffer`].
/// Checks [`BufferStore`] for more details.
#[derive(Clone)]
pub struct BufferDescriptor {
    usage: BufferUsage,
    status: Rc<RefCell<BufferStatus>>,
    memory_policy: MemoryPolicy,
}

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
        Self {
            status: Rc::new(RefCell::new(BufferStatus::UpdateBuffer {
                source,
                subs: VecDeque::new(),
            })),
            usage,
            memory_policy,
        }
    }

    /// Gets the bytes length of the data in this descriptor.
    pub fn size(&self) -> u32 {
        match &*self.status.borrow() {
            BufferStatus::Dropped => 0,
            BufferStatus::Unchanged { size, .. } => *size,
            BufferStatus::UpdateBuffer { source, .. } => source.bytes_length(),
            BufferStatus::UpdateSubBuffer { size, .. } => *size,
        }
    }

    /// Gets the [`BufferTarget`] of this buffer descriptor.
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    /// Gets the [`MemoryPolicy`] of this buffer descriptor.
    pub fn memory_policy(&self) -> &MemoryPolicy {
        &self.memory_policy
    }

    /// Sets the [`MemoryPolicy`] of this buffer descriptor.
    pub fn set_memory_policy(&mut self, policy: MemoryPolicy) {
        self.memory_policy = policy;
        self.update_memory_policy();
    }

    /// Updates memory policy to the storage item if exists.
    fn update_memory_policy(&mut self) {
        let status = (*self.status).borrow_mut();
        let Some(agency) = status.agency() else {
            return;
        };
        let Some(container) = agency.1.upgrade() else {
            return;
        };
        let mut container = (*container).borrow_mut();
        let Some(item) = container.store.get_mut(&agency.0) else {
            return;
        };
        item.borrow_mut().memory_policy_kind = self.memory_policy.to_kind();
    }

    /// Buffers new data to [`WebGlBuffer`].
    /// This operation overrides existing data forcedly.
    pub fn buffer_data(&mut self, source: BufferSource) {
        self.status.replace(BufferStatus::UpdateBuffer {
            source,
            subs: VecDeque::new(),
        });
    }

    /// Buffers sub data to [`WebGlBuffer`].
    ///
    /// # Preallocation
    ///
    /// [`BufferSource::Preallocate`] is not allowed here, even if it is not denied.
    /// It will be converted to an empty [`Uint8Array`] with a same size.
    ///
    /// # In Practice
    ///
    /// It is never a good to buffer a sub data with greater size and dest byte offset to an existing data.
    /// Under current memory management strategy, if you do such an operation,
    /// buffer store have to do a list of works to ensure the data consistency:
    ///
    /// 1. Gets data back from WebGL runtime (or restores it if [`MemoryPolicy::Restorable`]).
    /// 2. Preallocates a new buffer with the new size.
    /// 3. Buffers old data again
    /// 4. BUffers new data
    ///
    /// It is a extremely high overhead works!
    ///
    /// For the best performance, tries your best to prevent buffer sub data.
    /// If you must buffer sub data for dynamic data, it is a good practice to preallocate a large enough buffer first.
    /// And remember to set the buffer usage to [`BufferUsage::DynamicDraw`].
    pub fn buffer_sub_data(&mut self, source: BufferSource, dst_byte_offset: GLintptr) {
        // no preallocation allowed here, replaces with empty Buffer
        let source = match &source {
            BufferSource::Preallocate { size } => BufferSource::Uint8Array {
                data: Uint8Array::new_with_length(*size as u32),
                src_offset: 0,
                src_length: *size as u32,
            },
            BufferSource::Binary { .. }
            | BufferSource::Int8Array { .. }
            | BufferSource::Uint8Array { .. }
            | BufferSource::Uint8ClampedArray { .. }
            | BufferSource::Int16Array { .. }
            | BufferSource::Uint16Array { .. }
            | BufferSource::Int32Array { .. }
            | BufferSource::Uint32Array { .. }
            | BufferSource::Float32Array { .. }
            | BufferSource::Float64Array { .. }
            | BufferSource::BigInt64Array { .. }
            | BufferSource::BigUint64Array { .. } => source,
        };

        let mut status = self.status.borrow_mut();
        let status = &mut *status;
        match status {
            BufferStatus::Dropped => {
                *status = BufferStatus::UpdateBuffer {
                    source,
                    subs: VecDeque::new(),
                }
            }
            BufferStatus::Unchanged { size, agency } => {
                if size.saturating_sub(dst_byte_offset as u32) >= source.bytes_length() {
                    // update sub buffer is source bytes length smaller than available size
                    *status = BufferStatus::UpdateSubBuffer {
                        size: *size,
                        agency: agency.clone(),
                        overflow: false,
                        subs: VecDeque::from([(source, dst_byte_offset)]),
                    }
                } else {
                    if dst_byte_offset == 0 {
                        // if dst_byte_offset is 0, do a completely new buffer simply
                        *status = BufferStatus::UpdateBuffer {
                            source,
                            subs: VecDeque::new(),
                        }
                    } else {
                        // if else, heavy work need to apply
                        *status = BufferStatus::UpdateSubBuffer {
                            size: dst_byte_offset as u32 + source.bytes_length(),
                            agency: agency.clone(),
                            overflow: true,
                            subs: VecDeque::from([(source, dst_byte_offset)]),
                        }
                    }
                }
            }
            _ => {
                let (size, subs) = match status {
                    BufferStatus::Dropped | BufferStatus::Unchanged { .. } => unreachable!(),
                    BufferStatus::UpdateBuffer { source, subs } => (source.bytes_length(), subs),
                    BufferStatus::UpdateSubBuffer { size, subs, .. } => (*size, subs),
                };

                if size.saturating_sub(dst_byte_offset as u32) >= source.bytes_length() {
                    // add to subs queue if source bytes length smaller than available size
                    subs.push_back((source, dst_byte_offset));
                } else {
                    // otherwise, buffer a new preallocate source with new size first
                    // then buffer each subs in the queue in order.
                    let allocated_size =
                        (size + dst_byte_offset as u32 + source.bytes_length()) as i32;
                    let mut new_subs = subs.drain(..).collect::<VecDeque<_>>();
                    new_subs.push_back((source, dst_byte_offset));

                    *status = BufferStatus::UpdateBuffer {
                        source: BufferSource::Preallocate {
                            size: allocated_size,
                        },
                        subs: new_subs,
                    };
                }
            }
        };
    }
}

/// Buffer item lets developer always [`BufferStore`].
/// Checks [`BufferStore`] for more details.
#[derive(Clone)]
pub struct BufferItem(Rc<RefCell<StorageItem>>, BufferDescriptor);

impl BufferItem {
    /// Gets [`WebGlBuffer`].
    pub fn gl_buffer(&self) -> WebGlBuffer {
        self.0.borrow().buffer.clone()
    }

    /// Gets [`BufferTarget`].
    pub fn target(&self) -> BufferTarget {
        self.0.borrow().target
    }

    /// Gets memory in bytes size of this buffer used.
    pub fn size(&self) -> u32 {
        self.0.borrow().size
    }
}

/// Memory freeing policies.
/// Checks [`BufferStore`] for more details.
#[derive(Clone)]
pub enum MemoryPolicy {
    Default,
    Restorable(Rc<RefCell<dyn Fn() -> BufferSource>>),
    Unfree,
}

impl MemoryPolicy {
    /// Constructs a default memory policy.
    pub fn from_default() -> Self {
        Self::Default
    }

    /// Constructs a unfreeable memory policy.
    pub fn from_unfree() -> Self {
        Self::Unfree
    }

    /// Constructs a restorable memory policy.
    pub fn from_restorable<F: Fn() -> BufferSource + 'static>(f: F) -> Self {
        Self::Restorable(Rc::new(RefCell::new(f)))
    }

    fn to_kind(&self) -> MemoryPolicyKind {
        match self {
            MemoryPolicy::Default => MemoryPolicyKind::Default,
            MemoryPolicy::Restorable(_) => MemoryPolicyKind::Restorable,
            MemoryPolicy::Unfree => MemoryPolicyKind::Unfree,
        }
    }
}

/// Inner memory policy kind for checking only.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MemoryPolicyKind {
    Default,
    Restorable,
    Unfree,
}

/// Inner item of a [`BufferStore`].
struct StorageItem {
    id: Uuid,
    target: BufferTarget,
    buffer: WebGlBuffer,
    status: Weak<RefCell<BufferStatus>>,
    size: u32,
    memory_policy_kind: MemoryPolicyKind,
    lru_node: *mut LruNode<Uuid>,
}

/// Inner container of a [`BufferStore`].
struct StoreContainer {
    store: HashMap<Uuid, Rc<RefCell<StorageItem>>>,
    used_memory: u32,
    lru: Lru<Uuid>,
}

/// A centralize store managing large amount of [`WebGlBuffer`]s and its data.
///
/// # Buffer Descriptor
///
/// [`BufferDescriptor`] is the key to share and control the [`WebGlBuffer`].
/// Developer could create a descriptor, tells it the data for use in WebGL runtime
/// and even reuses the [`WebGlBuffer`] by cloning descriptor for possible purpose.
///
/// # Buffer Agency
///
/// [`BufferAgency`] is the final holder of the [`WebGlBuffer`] after the first use of a buffer descriptor.
/// [`WebGlBuffer`] lives alive as long as the agency alive and [`WebGlBuffer`] is dropped when the agency drops.
/// [`BufferAgency`] and its clones stay in different places to ensure the [`WebGlBuffer`] is always accessible.
/// Accessing the buffer agency is not allowed outside this module. Developer has no need to worries about where
/// the agencies are, they will stay in the right places ([`BufferDescriptor`] always holds a clone of buffer agency).
///
/// However, even the [`WebGlBuffer`] should be alive as long as the agency alive,
/// it still possible to be dropped and freed in background depending on [`MemoryPolicy`].
/// Checks Memory Management section for more details about memory freeing.
///
/// # Buffer Item
///
/// Memory freeing could be annoying sometimes, especially when we bind [`WebGlBuffer`] to WebGL runtime
/// and prepare for draw calls. It may unexpectedly drops our data before finishing a draw procedure.
/// [`BufferItem`] is designed for preventing such a situation. When developer holds a [`BufferItem`]
/// somewhere outside the store, memory freeing procedure never free the [`WebGlBuffer`] associated with
/// that [`BufferItem`]. Do remember to drop the [`BufferItem`] once you finish using the buffer.
///
/// # Reusing
///
/// Reusing a [`WebGlBuffer`] is easy, all you have to do is clone a descriptor you want to reuse.
///
/// # Dropping
///
/// It is easy to drop a [`WebGlBuffer`] as well, it is done by dropping the descriptor (and all the cloned ones of course).
/// Considering not accidentally dropping a reusing buffer, buffer store does not provide an explicit method to manually drop a buffer.
/// You should always remember where your descriptors are.
///
/// # Memory Management
///
/// Buffer store records memory consumption when [`WebGlBuffer`]s create or delete
/// and updates its usage status using LRU algorithm.
/// Since WebGL does not provide a way to get the max available memory of the GPU hardware,
/// developer could set a maximum available memory manually to the buffer store (or never free memory if not specified).
/// But store does not prevent creating more buffer even if memory overloaded.
///
/// When buffer store detects the used memory exceeds the max available memory,
/// a memory freeing procedure is triggered to free up WebGL memory automatically for different [`MemoryPolicy`]:
///
/// - Never drops all in use [`WebGlBuffer`]s, preventing accidentally deleting a buffer preparing for next draw call.
/// - For [`MemoryPolicy::Restorable`], buffer stores simply drops the [`WebGlBuffer`].
/// On the next time when the descriptor being used again, it is restored from the restore function.
/// - For [`MemoryPolicy::Unfree`], store never drops the [`WebGlBuffer`] even if used memory exceeds the max available memory already.
/// - For [`MemoryPolicy::Default`], store retrieves data back from WebGL runtime and stores it in CPU memory before dropping [`WebGlBuffer`].
/// It is not recommended to use this policy because getting data back from WebGL is an extremely high overhead.
/// Always considering proving a restore function for a better performance, or marking it as unfree if you sure to do that.
///
/// [`MemoryPolicy`] of a descriptor is changeable, feels free to change it and it will be used in next freeing procedure.
///
/// ## One More Thing You Should Know
///
/// Under the most implementations of WebGL, the data sent to WebGL with `bufferData` is not sent to GPU memory immediately.
/// Thus, you may discover that, the memory used recorded by the store is always greater than the actual used by the GPU.
pub struct BufferStore {
    gl: WebGl2RenderingContext,
    max_memory: u32,
    container: Rc<RefCell<StoreContainer>>,
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited memory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_max_memory(gl, u32::MAX)
    }

    /// Constructs a new buffer store with a maximum available memory.
    pub fn with_max_memory(gl: WebGl2RenderingContext, max_memory: u32) -> Self {
        Self {
            gl,
            max_memory,
            container: Rc::new(RefCell::new(StoreContainer {
                store: HashMap::new(),
                used_memory: 0,
                lru: Lru::new(),
            })),
        }
    }

    /// Gets the maximum available memory of the store.
    ///
    /// Returns `usize::MAX` if not specified.
    pub fn max_memory(&self) -> u32 {
        self.max_memory
    }

    /// Gets current used memory size.
    pub fn used_memory(&self) -> u32 {
        self.container.borrow().used_memory
    }
}

impl BufferStore {
    /// Uses a [`WebGlBuffer`] by a [`BufferDescriptor`] and buffer data to it if necessary.
    pub fn use_buffer(
        &mut self,
        buffer_descriptor: BufferDescriptor,
        target: BufferTarget,
    ) -> Result<BufferItem, Error> {
        let buffer_item = self.create_buffer(buffer_descriptor, target)?;
        self.buffer_data(&buffer_item)?;

        Ok(buffer_item)
    }

    /// Creates or gets an existing a [`WebGlBuffer`] by a [`BufferDescriptor`] only, no buffering data into it.
    pub fn create_buffer(
        &mut self,
        buffer_descriptor: BufferDescriptor,
        target: BufferTarget,
    ) -> Result<BufferItem, Error> {
        let BufferDescriptor {
            status,
            memory_policy,
            ..
        } = &buffer_descriptor;

        let mut container = (*self.container).borrow_mut();
        let container = &mut *container;

        let item = match &*status.borrow() {
            BufferStatus::Unchanged { agency, .. } => {
                let storage = container
                    .store
                    .get_mut(agency.key())
                    .ok_or(Error::BufferStorageNotFound(*agency.key()))?;

                // updates LRU
                unsafe {
                    container.lru.cache(storage.borrow_mut().lru_node);
                }

                Rc::clone(storage)
            }
            BufferStatus::Dropped | BufferStatus::UpdateBuffer { .. } => {
                // creates buffer
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };

                let id = Uuid::new_v4();
                // caches it into LRU
                let lru_node = unsafe {
                    let lru_node = LruNode::new(id);
                    container.lru.cache(lru_node);
                    lru_node
                };

                // stores it
                let storage = Rc::new(RefCell::new(StorageItem {
                    id,
                    target,
                    buffer: buffer.clone(),
                    size: 0,
                    status: Rc::downgrade(&status),
                    memory_policy_kind: memory_policy.to_kind(),
                    lru_node,
                }));
                container.store.insert(id, Rc::clone(&storage));

                debug!(target: "buffer_store", "create buffer {}", id);

                storage
            }
            BufferStatus::UpdateSubBuffer { agency, .. } => {
                let Some(storage) = container.store.get_mut(agency.key()) else {
                    return Err(Error::BufferStorageNotFound(*agency.key()));
                };
                let StorageItem { lru_node, .. } = &mut *storage.borrow_mut();

                // updates LRU
                unsafe {
                    container.lru.cache(*lru_node);
                }

                Rc::clone(&storage)
            }
        };

        Ok(BufferItem(item, buffer_descriptor))
    }

    /// Buffers a [`BufferItem`] if necessary.
    pub fn buffer_data(&mut self, buffer_item: &BufferItem) -> Result<(), Error> {
        let BufferItem(storage, descriptor) = buffer_item;

        let target = storage.borrow().target;
        let id = storage.borrow().id;
        let usage = descriptor.usage;

        let mut container_guard = (*self.container).borrow_mut();
        let container_mut = &mut *container_guard;

        let mut status_guard = descriptor.status.borrow_mut();
        let status_mut = &mut *status_guard;

        match status_mut {
            BufferStatus::Unchanged { .. } => {
                // do nothing
            }
            BufferStatus::Dropped | BufferStatus::UpdateBuffer { .. } => {
                // gets buffer source
                let tmp_source: BufferSource;
                let (source, subs) = match status_mut {
                    BufferStatus::Dropped => {
                        let MemoryPolicy::Restorable(restore) = &descriptor.memory_policy else {
                            return Err(Error::BufferUnexpectedDropped);
                        };
                        tmp_source = restore.borrow_mut()();
                        (&tmp_source, std::ptr::null_mut())
                    }
                    BufferStatus::UpdateBuffer { source, subs } => {
                        (&*source, subs as *mut VecDeque<(BufferSource, GLintptr)>)
                    }
                    _ => unreachable!(),
                };

                self.gl
                    .bind_buffer(target.gl_enum(), Some(&storage.borrow().buffer));
                source.buffer_data(&self.gl, target, usage);
                // travels and buffer each sub data
                if !subs.is_null() {
                    unsafe {
                        let subs = &mut *subs;
                        while let Some((sub, dst_byte_offset)) = subs.pop_front() {
                            sub.buffer_sub_data(&self.gl, target, dst_byte_offset);
                        }
                    }
                }
                let size = self
                    .gl
                    .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                    .as_f64()
                    .unwrap() as u32; // gets and updates memory usage
                container_mut.used_memory += size;
                storage.borrow_mut().size = size;
                self.gl.bind_buffer(target.gl_enum(), None);

                // replaces descriptor status
                *status_mut = BufferStatus::Unchanged {
                    size,
                    agency: Rc::new(BufferAgency(
                        id,
                        Rc::downgrade(&self.container),
                        self.gl.clone(),
                    )),
                };

                debug!(target: "buffer_store", "buffer data to {}. consumed memory {}, used {}", id, format_bytes_length(size), format_bytes_length(container_mut.used_memory));
            }
            BufferStatus::UpdateSubBuffer {
                size,
                agency,
                subs,
                overflow,
            } => {
                self.gl
                    .bind_buffer(target.gl_enum(), Some(&storage.borrow().buffer));
                if *overflow {
                    let osize = self
                        .gl
                        .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                        .as_f64()
                        .unwrap() as u32; // gets and updates memory usage

                    let data = Uint8Array::new_with_length(osize);
                    self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                        target.gl_enum(),
                        0,
                        &data,
                    );
                    subs.push_front((
                        BufferSource::Uint8Array {
                            data,
                            src_offset: 0,
                            src_length: osize,
                        },
                        0,
                    ));

                    // allocates for new buffer
                    self.gl
                        .buffer_data_with_i32(target.gl_enum(), *size as i32, usage.gl_enum());

                    storage.borrow_mut().size = *size;
                    container_mut.used_memory += *size - osize;

                    warn!(
                        target: "buffer_store",
                        "buffer sub data overflow {}. previous {}, current {}, used {}",
                        id,
                        format_bytes_length(osize),
                        format_bytes_length(*size),
                        format_bytes_length(container_mut.used_memory)
                    );
                }
                while let Some((sub, dst_byte_offset)) = subs.pop_front() {
                    sub.buffer_sub_data(&self.gl, target, dst_byte_offset);
                }
                self.gl.bind_buffer(target.gl_enum(), None);

                // replaces descriptor status
                let agency = Rc::clone(agency);
                *status_mut = BufferStatus::Unchanged {
                    size: *size,
                    agency: Rc::clone(&agency),
                };

                debug!(target: "buffer_store", "buffer sub data to {}. used {}", id, format_bytes_length(container_mut.used_memory));
            }
        };

        drop(container_guard);
        drop(status_guard);

        self.free();

        Ok(())
    }

    /// Frees memory if used memory exceeds the maximum available memory.
    fn free(&mut self) {
        let mut container = (*self.container).borrow_mut();

        if container.used_memory < self.max_memory {
            return;
        }

        // removes buffer from the least recently used until memory usage lower than limitation
        unsafe {
            let mut next_node = container.lru.least_recently();
            while container.used_memory >= self.max_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };

                let Some(item) = container.store.get((*current_node).data()) else {
                    next_node = (*current_node).more_recently();
                    debug!("1 {} {}", container.store.len(), container.lru.len());
                    continue;
                };

                // skips if in use
                if Rc::strong_count(item) > 1 {
                    next_node = (*current_node).more_recently();
                    debug!("2");
                    continue;
                }

                // skips if unfreeable
                if MemoryPolicyKind::Unfree == item.borrow().memory_policy_kind {
                    next_node = (*current_node).more_recently();
                    debug!("3");
                    continue;
                }

                let Some(item) = container.store.remove((*current_node).data()) else {
                    next_node = (*current_node).more_recently();
                    debug!("4");
                    continue;
                };
                let StorageItem {
                    target,
                    status,
                    buffer,
                    size,
                    lru_node,
                    memory_policy_kind,
                    id,
                } = &mut *item.borrow_mut();

                let Some(status) = status.upgrade() else {
                    next_node = (**lru_node).more_recently();
                    debug!("5");
                    continue;
                };

                match memory_policy_kind {
                    MemoryPolicyKind::Default => {
                        // default, gets buffer data back from WebGlBuffer
                        self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                        let data = Uint8Array::new_with_length(*size as u32);
                        self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                            target.gl_enum(),
                            0,
                            &data,
                        );
                        self.gl.bind_buffer(target.gl_enum(), None);

                        // updates status
                        *status.borrow_mut() = BufferStatus::UpdateBuffer {
                            source: BufferSource::from_uint8_array(data, 0, *size as u32),
                            subs: VecDeque::new(),
                        };
                    }
                    MemoryPolicyKind::Restorable => {
                        // if restorable, drops buffer directly

                        // deletes WebGlBuffer
                        self.gl.delete_buffer(Some(&buffer));

                        // updates status
                        *status.borrow_mut() = BufferStatus::Dropped;
                    }
                    MemoryPolicyKind::Unfree => unreachable!(),
                }

                // reduces memory
                container.used_memory -= *size;

                // updates LRU
                next_node = (**lru_node).more_recently();
                container.lru.remove(*lru_node);

                // logs
                match memory_policy_kind {
                    MemoryPolicyKind::Default => {
                        debug!(
                            target: "buffer_store",
                            "free buffer (default) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(*size),
                            format_bytes_length(container.used_memory)
                        );
                    }
                    MemoryPolicyKind::Restorable => {
                        debug!(
                            target: "buffer_store",
                            "free buffer (restorable) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(*size),
                            format_bytes_length(container.used_memory)
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Deletes all [`WebGlBuffer`] from WebGL runtime and
/// changes descriptors status to [`BufferDescriptorStatus::Dropped`].
impl Drop for BufferStore {
    fn drop(&mut self) {
        let gl = &self.gl;

        self.container
            .borrow_mut()
            .store
            .iter()
            .for_each(|(_, item)| {
                let StorageItem { buffer, status, .. } = &mut *item.borrow_mut();
                gl.delete_buffer(Some(&buffer));
                status.upgrade().map(|status| {
                    *(*status).borrow_mut() = BufferStatus::Dropped;
                });

                // store dropped, no need to update LRU anymore
            });
    }
}
