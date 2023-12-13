use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use log::debug;
use uuid::Uuid;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array,
        Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::utils::format_bytes_length;

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
        dst_byte_offset: GLintptr,
    },
    Binary {
        data: Box<dyn AsRef<[u8]>>,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int8Array {
        data: Int8Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint8Array {
        data: Uint8Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int16Array {
        data: Int16Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint16Array {
        data: Uint16Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Int32Array {
        data: Int32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Uint32Array {
        data: Uint32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Float32Array {
        data: Float32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    Float64Array {
        data: Float64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    BigInt64Array {
        data: BigInt64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    BigUint64Array {
        data: BigUint64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
}

impl BufferSource {
    fn collect_typed_array_buffer(&self) -> (&Object, GLintptr, GLuint, GLuint) {
        match self {
            BufferSource::Preallocate { .. } | BufferSource::Binary { .. } => {
                unreachable!()
            }
            BufferSource::Int8Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Uint8Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Uint8ClampedArray {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Int16Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Uint16Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Int32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Uint32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Float32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::Float64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::BigInt64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::BigUint64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data, *dst_byte_offset, *src_offset, *src_length),
        }
    }

    /// Buffers data to WebGL runtime.
    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        let target = target.gl_enum();
        let usage = usage.gl_enum();
        match self {
            BufferSource::Preallocate { size, dst_byte_offset } => gl.buffer_data_with_i32(target, *size, usage),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
                ..
            } => gl.buffer_data_with_u8_array_and_src_offset_and_length(
                target,
                data.as_ref().as_ref(),
                usage,
                *src_offset,
                *src_length,
            ),
            _ => {
                let (data, _, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_data_with_array_buffer_view_and_src_offset_and_length(
                    target, data, usage, src_offset, src_length,
                );
            }
        }
    }

    /// Buffers sub data to WebGL runtime.
    fn buffer_sub_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget) {
        let target = target.gl_enum();
        match self {
            BufferSource::Preallocate { size } => gl
                .buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target,
                    dst_byte_offset,
                    src_data,
                    src_offset,
                    length,
                ),
            BufferSource::Binary {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                target,
                *dst_byte_offset,
                data.as_ref().as_ref(),
                *src_offset,
                *src_length,
            ),
            _ => {
                let (data, dst_byte_offset, src_offset, src_length) =
                    self.collect_typed_array_buffer();
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

    fn dst_src_offset(&self) -> u32 {
        match self {
            BufferSource::Preallocate { size } => 0,
            BufferSource::Binary {
                dst_byte_offset, ..
            }
            | BufferSource::Int8Array {
                dst_byte_offset, ..
            }
            | BufferSource::Uint8Array {
                dst_byte_offset, ..
            }
            | BufferSource::Uint8ClampedArray {
                dst_byte_offset, ..
            }
            | BufferSource::Int16Array {
                dst_byte_offset, ..
            }
            | BufferSource::Uint16Array {
                dst_byte_offset, ..
            }
            | BufferSource::Int32Array {
                dst_byte_offset, ..
            }
            | BufferSource::Uint32Array {
                dst_byte_offset, ..
            }
            | BufferSource::Float32Array {
                dst_byte_offset, ..
            }
            | BufferSource::Float64Array {
                dst_byte_offset, ..
            }
            | BufferSource::BigInt64Array {
                dst_byte_offset, ..
            }
            | BufferSource::BigUint64Array {
                dst_byte_offset, ..
            } => *dst_byte_offset as u32,
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
        Self::from_binary_with_dst_byte_offset(data, src_offset, src_length, 0)
    }

    /// Constructs a new buffer source from WASM native buffer with dest byte offset.
    pub fn from_binary_with_dst_byte_offset<D: AsRef<[u8]> + 'static>(
        data: D,
        src_offset: GLuint,
        src_length: GLuint,
        dst_byte_offset: GLsizeiptr,
    ) -> Self {
        Self::Binary {
            data: Box::new(data),
            dst_byte_offset,
            src_offset,
            src_length,
        }
    }
}

macro_rules! impl_typed_array {
    ($(($from: ident, $from_with: ident, $source: tt, $kind: ident, $name: expr)),+) => {
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
                    Self::$from_with(data, src_offset, src_length, 0)
                }

                #[doc = "Constructs a new buffer source from "]
                #[doc = $name]
                #[doc = " with dest byte offset."]
                pub fn $from_with(
                    data: $source,
                    src_offset: GLuint,
                    src_length: GLuint,
                    dst_byte_offset: GLsizeiptr,
                ) -> Self {
                    Self::$kind {
                        data,
                        dst_byte_offset,
                        src_offset,
                        src_length,
                    }
                }
            )+
        }
    };
}

impl_typed_array! {
    (from_int8_array, from_int8_array_with_dst_byte_length, Int8Array, Int8Array, "[`Int8Array`]"),
    (from_uint8_array, from_uint8_array_with_dst_byte_length, Uint8Array, Uint8Array, "[`Uint8Array`]"),
    (from_uint8_clamped_array, from_uint8_clamped_array_with_dst_byte_length, Uint8ClampedArray, Uint8ClampedArray, "[`Uint8ClampedArray`]"),
    (from_int16_array, from_int16_array_with_dst_byte_length, Int16Array, Int16Array, "[`Int16Array`]"),
    (from_uint16_array, from_uint16_array_with_dst_byte_length, Uint16Array, Uint16Array, "[`Uint16Array`]"),
    (from_int32_array, from_int32_array_with_dst_byte_length, Int32Array, Int32Array, "[`Int32Array`]"),
    (from_uint32_array, from_uint32_array_with_dst_byte_length, Uint32Array, Uint32Array, "[`Uint32Array`]"),
    (from_float32_array, from_float32_array_with_dst_byte_length, Float32Array, Float32Array, "[`Float32Array`]"),
    (from_float64_array, from_float64_array_with_dst_byte_length, Float64Array, Float64Array, "[`Float64Array`]"),
    (from_big_int64_array, from_big_int64_array_with_dst_byte_length, BigInt64Array, BigInt64Array, "[`BigInt64Array`]"),
    (from_big_uint64_array, from_big_uint64_array_with_dst_byte_length, BigUint64Array, BigUint64Array, "[`BigUint64Array`]")
}

/// The thing for achieving [`BufferDescriptor`] reusing and automatic dropping purpose.
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

        // updates most and least LRU
        if let Some(most_recently) = container.most_recently {
            let most_recently = unsafe { &*most_recently };
            if most_recently.id == lru_node.id {
                container.most_recently = lru_node.less_recently;
            }
        }
        if let Some(least_recently) = container.least_recently {
            let least_recently = unsafe { &*least_recently };
            if least_recently.id == lru_node.id {
                container.least_recently = lru_node.more_recently;
            }
        }
        // updates self connecting LRU
        if let Some(less_recently) = lru_node.less_recently {
            let less_recently = unsafe { &mut *less_recently };
            less_recently.more_recently = lru_node.more_recently;
        }
        if let Some(more_recently) = lru_node.more_recently {
            let more_recently = unsafe { &mut *more_recently };
            more_recently.less_recently = lru_node.less_recently;
        }

        debug!(target: "buffer_store", "drop buffer {}. freed memory {}, used memory {}", id, format_bytes_length(*size), format_bytes_length(container.used_memory));
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
        subs: VecDeque<BufferSource>,
    },
    /// Buffers sub data into WebGL2 runtime.
    ///
    /// Drops source data after buffering into WebGL2 runtime.
    UpdateSubBuffer {
        size: u32,
        agency: Rc<BufferAgency>,
        subs: VecDeque<BufferSource>,
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

/// Buffer descriptor is a reuseable key to set and get [`WebGlBuffer`] from [`BufferStore`].
///
/// # Reusing
///
/// `BufferDescriptor` is cloneable for reusing purpose.
/// Cloning a descriptor lets you reuse the [`WebGlBuffer`] associated with this descriptor.
///
/// # Dropping
///
/// [`WebGlBuffer`] associated with the descriptor will be automatically deleted when the
/// buffer descriptor eventually dropped.
///
/// # Memory Freeing Policy
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

    /// Buffers new data to WebGL runtime.
    pub fn buffer_data(&mut self, source: BufferSource) {
        self.status.replace(BufferStatus::UpdateBuffer {
            source,
            subs: VecDeque::new(),
        });
    }

    /// Buffers sub data to WebGL runtime.
    pub fn buffer_sub_data(&mut self, source: BufferSource) {
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
                *status = match &source {
                    BufferSource::Preallocate { size } => BufferStatus::UpdateBuffer {
                        source,
                        subs: VecDeque::new(),
                    },
                    _ => {
                        if size.saturating_sub(source.dst_src_offset()) >= source.bytes_length() {
                            BufferStatus::UpdateSubBuffer {
                                size: *size,
                                agency: agency.clone(),
                                subs: VecDeque::from([source]),
                            }
                        } else {
                            BufferStatus::UpdateBuffer {
                                source,
                                subs: VecDeque::new(),
                            }
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

                if size.saturating_sub(source.dst_src_offset()) >= source.bytes_length() {
                    subs.push_back(source);
                } else {
                    let allocated_size =
                        (size + source.dst_src_offset() + source.bytes_length()) as i32;
                    let mut new_subs = subs.drain(..).collect::<VecDeque<BufferSource>>();
                    new_subs.push_back(source);

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

/// Buffer item usable for outside the [`BufferStore`].
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

/// Memory policy.
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
    lru_node: Box<LruNode>,
    memory_policy_kind: MemoryPolicyKind,
}

/// Inner container of a [`BufferStore`].
struct StoreContainer {
    store: HashMap<Uuid, Rc<RefCell<StorageItem>>>,
    used_memory: u32,
    most_recently: Option<*mut LruNode>,
    least_recently: Option<*mut LruNode>,
}

/// LRU node for GPU memory management.
struct LruNode {
    id: Uuid,
    less_recently: Option<*mut LruNode>,
    more_recently: Option<*mut LruNode>,
}

macro_rules! to_most_recently_lru {
    ($container: expr, $lru: expr) => {
        'to_most_recently_lru: {
            match ($lru.more_recently, $lru.less_recently) {
                (Some(more_recently), Some(less_recently)) => {
                    // i am a node in the middle of the LRU, chains up prev and next nodes
                    let more_recently = unsafe { &mut *more_recently };
                    let less_recently = unsafe { &mut *less_recently };

                    more_recently.less_recently = Some(less_recently);
                    less_recently.more_recently = Some(more_recently);
                }
                (Some(more_recently), None) => {
                    // i must be the least recently node, let prev node to be the least node
                    let more_recently = unsafe { &mut *more_recently };
                    more_recently.less_recently = None;
                    $container.least_recently = Some(more_recently);
                }
                (None, Some(_)) => {
                    // i must be the most recently node, do nothing!
                    break 'to_most_recently_lru;
                }
                (None, None) => {
                    // i am a new node or the single node in LRU.

                    if $container.most_recently.is_none() && $container.least_recently.is_none() {
                        // i am the first node in LRU cache! it is ok to just set the most and least recently to me.
                        $container.most_recently = Some($lru.as_mut());
                        $container.least_recently = Some($lru.as_mut());
                        break 'to_most_recently_lru;
                    } else if unsafe { &mut *$container.most_recently.unwrap() }.id == $lru.id {
                        // i am the single node in LRU, do nothing!
                        break 'to_most_recently_lru;
                    } else {
                        // for any other situations, step next to be the most recently node
                    }

                    // note, most recently and least recently should both carry a node or both not carry any node at the same time.
                }
            }

            // sets myself as the most recently node
            let most_recently = unsafe { &mut *$container.most_recently.unwrap() }; // if reach here, there must be something in most recently.

            $lru.more_recently = None;
            $lru.less_recently = $container.most_recently;
            most_recently.more_recently = Some($lru.as_mut());
            $container.most_recently = Some($lru.as_mut());
        };
    };
}

/// A centralize store managing large amount of [`WebGlBuffer`]s and its data.
///
/// # Buffer Descriptor
///
/// [`BufferDescriptor`] is the key to control the [`WebGlBuffer`].
/// Developer could create a descriptor, tells it the data for use in WebGL runtime
/// and even reuses the [`WebGlBuffer`] by cloning descriptor for possible purpose.
/// Checks [`BufferDescriptor`] struct for more details.
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
                most_recently: None,
                least_recently: None,
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
                let lru_node = &mut storage.borrow_mut().lru_node;
                to_most_recently_lru!(container, lru_node);

                Rc::clone(storage)
            }
            BufferStatus::Dropped | BufferStatus::UpdateBuffer { .. } => {
                // creates buffer
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };

                let id = Uuid::new_v4();
                // caches it into LRU
                let mut lru_node = Box::new(LruNode {
                    id,
                    less_recently: None,
                    more_recently: None,
                });
                to_most_recently_lru!(container, lru_node);

                // stores it
                let storage = Rc::new(RefCell::new(StorageItem {
                    id,
                    target,
                    buffer: buffer.clone(),
                    size: 0,
                    status: Rc::downgrade(&status),
                    lru_node,
                    memory_policy_kind: memory_policy.to_kind(),
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
                to_most_recently_lru!(container, lru_node);

                Rc::clone(&storage)
            }
        };

        Ok(BufferItem(item, buffer_descriptor))
    }

    /// Buffers a [`BufferItem`] if necessary.
    pub fn buffer_data(&mut self, buffer_item: &BufferItem) -> Result<(), Error> {
        let BufferItem(storage, descriptor) = buffer_item;

        let target = storage.borrow().target;
        let buffer = &storage.borrow().buffer;
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
                let tmp_subs: *mut VecDeque<BufferSource>;
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
                        (&*source, subs as *mut VecDeque<BufferSource>)
                    }
                    _ => unreachable!(),
                };

                self.gl.bind_buffer(target.gl_enum(), Some(buffer));
                source.buffer_data(&self.gl, target, usage);
                // travels and buffer each sub data
                if !subs.is_null() {
                    unsafe {
                        let subs = &mut *subs;
                        while let Some(sub) = subs.pop_front() {
                            sub.buffer_sub_data(&self.gl, target);
                        }
                    }
                }
                let size = self
                    .gl
                    .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                    .as_f64()
                    .unwrap() as u32; // gets and updates memory usage
                container_mut.used_memory += size;
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

                debug!(target: "buffer_store", "buffer data to {}. consumed memory {}, used memory {}", id, format_bytes_length(size), format_bytes_length(container_mut.used_memory));
            }
            BufferStatus::UpdateSubBuffer {
                size, agency, subs, ..
            } => {
                // buffer sub data may not change the allocated memory size
                self.gl.bind_buffer(target.gl_enum(), Some(buffer));
                while let Some(sub) = subs.pop_front() {
                    sub.buffer_sub_data(&self.gl, target);
                }
                self.gl.bind_buffer(target.gl_enum(), None);

                // replaces descriptor status
                let agency = Rc::clone(agency);
                *status_mut = BufferStatus::Unchanged {
                    size: *size,
                    agency: Rc::clone(&agency),
                };

                debug!(target: "buffer_store", "buffer sub data to {}. used memory {}", id, format_bytes_length(container_mut.used_memory));
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
        let mut next_node = container.least_recently;
        while container.used_memory >= self.max_memory {
            let Some(current_node) = next_node else {
                break;
            };
            let current_node = unsafe { &*current_node };

            let Some(item) = container.store.get(&current_node.id) else {
                next_node = current_node.more_recently;
                continue;
            };

            // skips if in use
            if Rc::strong_count(item) > 1 {
                next_node = current_node.more_recently;
                continue;
            }

            // skips if unfreeable
            if MemoryPolicyKind::Unfree == item.borrow().memory_policy_kind {
                next_node = current_node.more_recently;
                continue;
            }

            let Some(item) = container.store.remove(&current_node.id) else {
                next_node = current_node.more_recently;
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

            // skips if status not exists any more
            let Some(status) = status.upgrade() else {
                next_node = current_node.more_recently;
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
            if let Some(more_recently) = lru_node.more_recently {
                let more_recently = unsafe { &mut *more_recently };
                more_recently.less_recently = None;
                container.least_recently = Some(more_recently);
                next_node = Some(more_recently);
            } else {
                container.least_recently = None;
                next_node = None;
            }

            debug!(target: "buffer_store", "free buffer {}. freed memory {}, used memory {}", id, format_bytes_length(*size), format_bytes_length(container.used_memory));
        }

        // console_log!("len {}", container.store.len());

        // let mut ids = Vec::new();
        // let mut node = container.most_recently;
        // while let Some(lru_node) = node {
        //     let lru_node = unsafe { &*lru_node };
        //     // console_log!("{}", lru_node.raw_id);

        //     ids.push(lru_node.raw_id.to_string());
        //     node = lru_node.less_recently;
        // }

        // console_log!("{}", ids.join(", "));
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
