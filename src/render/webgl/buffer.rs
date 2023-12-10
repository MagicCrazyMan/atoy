use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    rc::{Rc, Weak},
};

use uuid::Uuid;
use wasm_bindgen_test::console_log;
use web_sys::{
    js_sys::{
        BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array,
        Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
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
/// So, it is a good idea to avoid creating native buffer, and use `TypedArrayBuffer` from JavaScript instead.
pub enum BufferSource {
    Preallocate {
        size: GLsizeiptr,
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
            BufferSource::Preallocate { size } => gl.buffer_data_with_i32(target, *size, usage),
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
            BufferSource::Preallocate { .. } => unreachable!(),
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
            dst_byte_offset: 0,
            src_offset,
            src_length,
        }
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
                    Self::$kind {
                        data,
                        dst_byte_offset: 0,
                        src_offset,
                        src_length,
                    }
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

/// `BufferDescriptorAgency` is the thing for achieving [`BufferDescriptor`] reusing and automatic dropping purpose.
///
/// [`BufferStore`] creates a [`Rc`] wrapped agency for
/// a descriptor for the first time descriptor being used.
/// Cloned descriptors share the same agency.
/// [`BufferStore`] always returns a same [`WebGlBuffer`] with a same agency.
///
/// After all referencing of an agency dropped, agency will drop [`WebGlBuffer`] automatically in [`Drop`].
#[derive(Clone)]
struct BufferDescriptorAgency(Uuid, Weak<RefCell<StoreContainer>>, WebGl2RenderingContext);

impl BufferDescriptorAgency {
    fn key(&self) -> &Uuid {
        &self.0
    }
}

impl PartialEq for BufferDescriptorAgency {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BufferDescriptorAgency {}

impl Hash for BufferDescriptorAgency {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Deletes associated WebGlBuffer from store(if exists) when descriptor id drops.
impl Drop for BufferDescriptorAgency {
    fn drop(&mut self) {
        let Some(container) = self.1.upgrade() else {
            return;
        };
        let Ok(mut container) = (*container).try_borrow_mut() else {
            // if it is borrowed, buffer is dropping by store, skip
            return;
        };

        let Some(StorageItem {
            buffer,
            size,
            lru_node,
            ..
        }) = container.store.remove(&self.0)
        else {
            return;
        };

        self.2.delete_buffer(Some(&buffer));
        container.used_memory -= size;

        // updates most and least LRU
        if let Some(most_recently) = container.most_recently {
            let most_recently = unsafe { &*most_recently };
            if most_recently.raw_id == lru_node.raw_id {
                container.most_recently = lru_node.less_recently;
            }
        }
        if let Some(least_recently) = container.least_recently {
            let least_recently = unsafe { &*least_recently };
            if least_recently.raw_id == lru_node.raw_id {
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

        console_log!("buffer {} dropped by itself", &self.0);
    }
}

/// Buffer descriptor lifetime status.
///
/// Cloned descriptors share the same status.
enum BufferDescriptorStatus {
    /// Buffer associated with this descriptor dropped already.
    Dropped,
    /// Buffer associated with this descriptor does not change.
    Unchanged { agency: Rc<BufferDescriptorAgency> },
    /// Buffers data into WebGL2 runtime.
    ///
    /// Drops source data after buffering into WebGL2 runtime.
    UpdateBuffer { source: BufferSource },
    /// Buffers sub data into WebGL2 runtime.
    ///
    /// Drops source data after buffering into WebGL2 runtime.
    UpdateSubBuffer {
        agency: Rc<BufferDescriptorAgency>,
        source: BufferSource,
    },
}

impl BufferDescriptorStatus {
    /// Gets [`BufferDescriptorAgency`] associated with this buffer descriptor.
    fn agency(&self) -> Option<Rc<BufferDescriptorAgency>> {
        match self {
            BufferDescriptorStatus::Dropped => None,
            BufferDescriptorStatus::Unchanged { agency } => Some(Rc::clone(agency)),
            BufferDescriptorStatus::UpdateBuffer { .. } => None,
            BufferDescriptorStatus::UpdateSubBuffer { agency, .. } => Some(Rc::clone(agency)),
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
    status: Rc<RefCell<BufferDescriptorStatus>>,
    restore: Option<Rc<RefCell<Box<dyn FnMut() -> BufferSource>>>>,
}

impl BufferDescriptor {
    /// Constructs a new buffer descriptor with specified [`BufferSource`] and [`BufferUsage`]
    pub fn new(source: BufferSource, usage: BufferUsage) -> Self {
        Self {
            status: Rc::new(RefCell::new(BufferDescriptorStatus::UpdateBuffer {
                source,
            })),
            usage,
            restore: None,
        }
    }

    /// Gets the [`BufferTarget`] of this buffer descriptor.
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    /// Sets this buffer descriptor not restorable.
    ///
    /// If not restorable, buffer store retrieves data back from WebGlBuffer when freeing GPU memory.
    /// This is a fallback operation, usually resulting a worser performance.
    pub fn disable_restore(&mut self) {
        self.restore = None;
        self.update_restorable();
    }

    /// Sets this buffer descriptor restorable.
    ///
    /// If restorable, buffer store does not get data back from WebGlBuffer when freeing GPU memory.
    /// A restore function returning a new [`BufferSource`] is required for restoring data when
    /// this buffer descriptor being used again.
    pub fn enable_restore<F>(&mut self, restore: F)
    where
        F: FnMut() -> BufferSource + 'static,
    {
        self.restore = Some(Rc::new(RefCell::new(Box::new(restore))));
        self.update_restorable();
    }

    /// Tells storage item whether restorable if already buffered.
    fn update_restorable(&mut self) {
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
        item.restorable = self.restore.is_some();
    }

    /// Buffers new data to WebGL runtime.
    pub fn buffer_data(&mut self, source: BufferSource) {
        self.status.replace_with(|old| match old {
            BufferDescriptorStatus::Unchanged { .. } => {
                BufferDescriptorStatus::UpdateBuffer { source }
            }
            BufferDescriptorStatus::UpdateBuffer { .. } => {
                BufferDescriptorStatus::UpdateBuffer { source }
            }
            BufferDescriptorStatus::UpdateSubBuffer { .. } => {
                BufferDescriptorStatus::UpdateBuffer { source }
            }
            BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer { source },
        });
    }

    /// Buffers sub data to WebGL runtime.
    pub fn buffer_sub_data(&mut self, source: BufferSource) {
        self.status.replace_with(|old| match old {
            BufferDescriptorStatus::Unchanged { agency }
            | BufferDescriptorStatus::UpdateSubBuffer { agency, .. } => {
                BufferDescriptorStatus::UpdateSubBuffer {
                    agency: agency.clone(),
                    source,
                }
            }
            BufferDescriptorStatus::UpdateBuffer { .. } => {
                BufferDescriptorStatus::UpdateBuffer { source }
            }
            BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer { source },
        });
    }
}

pub enum MemoryPolicy {
    Default,
    Restorable,
    Unfree,
}

/// Inner item of a [`BufferStore`].
struct StorageItem {
    target: BufferTarget,
    buffer: WebGlBuffer,
    status: Weak<RefCell<BufferDescriptorStatus>>,
    size: usize,
    lru_node: Box<LruNode>,
    restorable: bool,
}

/// Inner container of a [`BufferStore`].
struct StoreContainer {
    store: HashMap<Uuid, StorageItem>,
    used_memory: usize,
    most_recently: Option<*mut LruNode>,
    least_recently: Option<*mut LruNode>,
}

/// LRU node for GPU memory management.
struct LruNode {
    raw_id: Uuid,
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
                    } else if unsafe { &mut *$container.most_recently.unwrap() }.raw_id == $lru.raw_id {
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
/// - For [`MemoryPolicy::Restorable`], buffer stores simply drops the [`WebGlBuffer`].
/// On the next time when the descriptor being used again, it is restored from the restore function.
/// - For [`MemoryPolicy::Unfree`], store never drops the [`WebGlBuffer`] even if used memory exceeds the max available memory already.
/// - For [`MemoryPolicy::Default`], store retrieves data back from WebGL runtime and stores it in CPU memory before dropping [`WebGlBuffer`].
/// It is not recommended to use this policy because getting data back from WebGL is an extremely high overhead.
/// Always considering proving a restore function for a better performance, or marking it as unfree if you sure to do that.
///
/// [`MemoryPolicy`] of a descriptor is changeable, feels free to change it and it use be used in next freeing procedure.
///
/// ## One More Thing You Should Know
///
/// Under the most implementations of WebGL, the data sent to WebGL with `bufferData` is not sent to GPU memory immediately.
/// Thus, you may discover that, the memory used recorded by the store is always greater than the actual used by the GPU.
pub struct BufferStore {
    gl: WebGl2RenderingContext,
    max_memory: usize,
    container: Rc<RefCell<StoreContainer>>,
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited memory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_max_memory(gl, usize::MAX)
    }

    /// Constructs a new buffer store with a maximum available memory.
    pub fn with_max_memory(gl: WebGl2RenderingContext, max_memory: usize) -> Self {
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
    pub fn max_memory(&self) -> usize {
        self.max_memory
    }

    /// Gets current used memory size.
    pub fn used_memory(&self) -> usize {
        self.container.borrow().used_memory
    }
}

impl BufferStore {
    /// Gets a [`WebGlBuffer`] by a [`BufferDescriptor`] and buffer data to it if necessary.
    pub fn use_buffer(
        &mut self,
        buffer_descriptor: BufferDescriptor,
        target: BufferTarget,
    ) -> Result<WebGlBuffer, Error> {
        let BufferDescriptor {
            status,
            usage,
            restore,
        } = buffer_descriptor;

        let mut container_guard = (*self.container).borrow_mut();
        let container_mut = &mut *container_guard;

        let mut status_guard = (*status).borrow_mut();
        let status_mut = &mut *status_guard;

        let buffer = match status_mut {
            BufferDescriptorStatus::Unchanged { agency, .. } => {
                let StorageItem {
                    buffer, lru_node, ..
                } = container_mut
                    .store
                    .get_mut(agency.key())
                    .ok_or(Error::BufferStorageNotFound(*agency.key()))?;

                // updates LRU
                to_most_recently_lru!(container_mut, lru_node);

                buffer.clone()
            }
            BufferDescriptorStatus::Dropped | BufferDescriptorStatus::UpdateBuffer { .. } => {
                // gets buffer source
                let tmp_source: BufferSource;
                let source = match status_mut {
                    // gets buffer source from restore in Dropped if exists, or throws error otherwise.
                    BufferDescriptorStatus::Dropped => {
                        let Some(restore) = restore.clone() else {
                            return Err(Error::BufferUnexpectedDropped);
                        };
                        tmp_source = restore.borrow_mut()();
                        &tmp_source
                    }
                    // gets buffer source from status in UpdateBuffer, and delete an old buffer if exists.
                    BufferDescriptorStatus::UpdateBuffer { source } => source,
                    _ => unreachable!(),
                };

                // creates buffer
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_data(&self.gl, target, usage);
                let size = self
                    .gl
                    .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                    .as_f64()
                    .unwrap() as usize; // gets and updates memory usage
                container_mut.used_memory += size;
                self.gl.bind_buffer(target.gl_enum(), None);

                // stores it and caches it into LRU
                let raw_id = Uuid::new_v4();
                let mut lru_node = Box::new(LruNode {
                    raw_id,
                    less_recently: None,
                    more_recently: None,
                });
                to_most_recently_lru!(container_mut, lru_node);
                container_mut.store.insert(
                    raw_id,
                    StorageItem {
                        target,
                        buffer: buffer.clone(),
                        size,
                        status: Rc::downgrade(&status),
                        lru_node,
                        restorable: restore.is_some(),
                    },
                );

                // replaces descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged {
                    agency: Rc::new(BufferDescriptorAgency(
                        raw_id,
                        Rc::downgrade(&self.container),
                        self.gl.clone(),
                    )),
                };
                

                buffer
            }
            BufferDescriptorStatus::UpdateSubBuffer { agency, source, .. } => {
                let Some(StorageItem {
                    buffer, lru_node, ..
                }) = container_mut.store.get_mut(agency.key())
                else {
                    return Err(Error::BufferStorageNotFound(*agency.key()));
                };

                // binds and buffers sub data.
                // buffer sub data may not change the allocated memory size
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_sub_data(&self.gl, target);
                self.gl.bind_buffer(target.gl_enum(), None);

                // replace descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged {
                    agency: agency.clone(),
                };

                // updates LRU
                to_most_recently_lru!(container_mut, lru_node);

                buffer.clone()
            }
        };

        drop(container_guard);
        drop(status_guard);

        self.free();

        Ok(buffer)
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

            let Some(StorageItem {
                target,
                status,
                buffer,
                size,
                lru_node,
                restorable,
            }) = container.store.remove(&current_node.raw_id)
            else {
                next_node = current_node.more_recently;
                continue;
            };

            // checks if buffer restorable
            if restorable {
                // if restorable, drops buffer directly

                // deletes WebGlBuffer
                self.gl.delete_buffer(Some(&buffer));

                // updates status
                if let Some(status) = status.upgrade() {
                    *status.borrow_mut() = BufferDescriptorStatus::Dropped;
                }
            } else {
                // if not restorable, gets buffer data back from WebGlBuffer
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                let data = Uint8Array::new_with_length(size as u32);
                self.gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
                    target.gl_enum(),
                    0,
                    &data,
                );
                self.gl.bind_buffer(target.gl_enum(), None);

                // updates status
                if let Some(status) = status.upgrade() {
                    *status.borrow_mut() = BufferDescriptorStatus::UpdateBuffer {
                        source: BufferSource::from_uint8_array(data, 0, size as u32),
                    };
                };
            }

            // reduces memory usage
            container.used_memory -= size;

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

            // console_log!(
            //     "buffer {} dropped, {} memory freed, {} total",
            //     node.raw_id,
            //     size,
            //     container.memory_usage
            // );
        }

        // let mut ids = Vec::new();
        // let mut node = (*self.container).borrow_mut().most_recently;
        // while let Some(lru_node) = node {
        //     let lru_node = unsafe { &*lru_node };
        //     // console_log!("{}", lru_node.raw_id);

        //     ids.push(lru_node.raw_id.to_string());
        //     node = lru_node.less_recently;
        // }

        // console_log!("{}", ids.join(", "));
    }
}

/// Deletes all WebGlBuffer from WebGL runtime and
/// changes all buffer descriptor status to [`BufferDescriptorStatus::Dropped`]
/// when buffer store drops.
impl Drop for BufferStore {
    fn drop(&mut self) {
        let gl = &self.gl;

        self.container.borrow_mut().store.iter().for_each(
            |(_, StorageItem { buffer, status, .. })| {
                gl.delete_buffer(Some(&buffer));
                status.upgrade().map(|status| {
                    *(*status).borrow_mut() = BufferDescriptorStatus::Dropped;
                });

                // store dropped, no need to update LRU anymore
            },
        );
    }
}
