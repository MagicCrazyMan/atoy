use std::{
    borrow::BorrowMut,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BufferComponentSize {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

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

/// Since the linear memory of WASM runtime is impossible to shrink (for now),
/// high memory usage could happen if you create a large WASM native buffer, for example, Vec\<u8\>.
/// Thus, always creates TypedArrayBuffer from JavaScript and buffer data via
/// From\[TypedArray\] to prevent creating large WASM buffer.
///
/// Note: Feels freely to clone the [`ArrayBuffer`], it only clones a reference from JavaScript heap.
pub enum BufferSource {
    Preallocate {
        size: GLsizeiptr,
    },
    FromBinary {
        data: Box<dyn AsRef<[u8]>>,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromInt8Array {
        data: Int8Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromUint8Array {
        data: Uint8Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromUint8ClampedArray {
        data: Uint8ClampedArray,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromInt16Array {
        data: Int16Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromUint16Array {
        data: Uint16Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromInt32Array {
        data: Int32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromUint32Array {
        data: Uint32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromFloat32Array {
        data: Float32Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromFloat64Array {
        data: Float64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromBigInt64Array {
        data: BigInt64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
    FromBigUint64Array {
        data: BigUint64Array,
        dst_byte_offset: GLintptr,
        src_offset: GLuint,
        src_length: GLuint,
    },
}

impl BufferSource {
    fn collect_typed_array_buffer(&self) -> (&Object, GLintptr, GLuint, GLuint) {
        match self {
            BufferSource::Preallocate { .. } | BufferSource::FromBinary { .. } => {
                unreachable!()
            }
            BufferSource::FromInt8Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromUint8Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromUint8ClampedArray {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromInt16Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromUint16Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromInt32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromUint32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromFloat32Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromFloat64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromBigInt64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
            BufferSource::FromBigUint64Array {
                data,
                dst_byte_offset,
                src_offset,
                src_length,
            } => (data as &Object, *dst_byte_offset, *src_offset, *src_length),
        }
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        let target = target.gl_enum();
        let usage = usage.gl_enum();
        match self {
            BufferSource::Preallocate { size } => gl.buffer_data_with_i32(target, *size, usage),
            BufferSource::FromBinary {
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

    fn buffer_sub_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget) {
        let target = target.gl_enum();
        match self {
            BufferSource::Preallocate { .. } => unreachable!(),
            BufferSource::FromBinary {
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
    /// Constructs a new buffer descriptor with preallocate buffer only.
    pub fn preallocate(size: GLsizeiptr) -> Self {
        Self::Preallocate { size }
    }

    /// Constructs a new buffer descriptor with data from WASM binary.
    ///
    /// DO NOT buffer a huge binary data using this method,
    /// use [`from_uint8_array()`](Self@from_uint8_array) or those `from_[TypedArray]()` methods instead.
    pub fn from_binary<D: AsRef<[u8]> + 'static>(
        data: D,
        src_offset: GLuint,
        src_length: GLuint,
    ) -> Self {
        Self::FromBinary {
            data: Box::new(data),
            dst_byte_offset: 0,
            src_offset,
            src_length,
        }
    }

    /// Constructs a new buffer descriptor with data from WASM binary.
    ///
    /// DO NOT buffer a huge binary data using this method,
    /// use [`from_uint8_array()`](Self@from_uint8_array) or those `from_[TypedArray]()` methods instead.
    pub fn from_binary_with_dst_byte_offset<D: AsRef<[u8]> + 'static>(
        data: D,
        src_offset: GLuint,
        src_length: GLuint,
        dst_byte_offset: GLsizeiptr,
    ) -> Self {
        Self::FromBinary {
            data: Box::new(data),
            dst_byte_offset,
            src_offset,
            src_length,
        }
    }
}

macro_rules! impl_typed_array {
    ($(($from: ident, $from_with: ident, $source: tt, $kind: ident)),+) => {
        impl BufferSource {
            $(
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
    (from_int8_array, from_int8_array_with_dst_byte_length, Int8Array, FromInt8Array),
    (from_uint8_array, from_uint8_array_with_dst_byte_length, Uint8Array, FromUint8Array),
    (from_uint8_clamped_array, from_uint8_clamped_array_with_dst_byte_length, Uint8ClampedArray, FromUint8ClampedArray),
    (from_int16_array, from_int16_array_with_dst_byte_length, Int16Array, FromInt16Array),
    (from_uint16_array, from_uint16_array_with_dst_byte_length, Uint16Array, FromUint16Array),
    (from_int32_array, from_int32_array_with_dst_byte_length, Int32Array, FromInt32Array),
    (from_uint32_array, from_uint32_array_with_dst_byte_length, Uint32Array, FromUint32Array),
    (from_float32_array, from_float32_array_with_dst_byte_length, Float32Array, FromFloat32Array),
    (from_float64_array, from_float64_array_with_dst_byte_length, Float64Array, FromFloat64Array),
    (from_big_int64_array, from_big_int64_array_with_dst_byte_length, BigInt64Array, FromBigInt64Array),
    (from_big_uint64_array, from_big_uint64_array_with_dst_byte_length, BigUint64Array, FromBigUint64Array)
}

/// An buffer agency is an unique identifier to save the runtime status of a buffer descriptor.
/// `BufferDescriptorAgency` is aimed for doing automatic WebGlBuffer cleanup when a buffer no more usable.
#[derive(Clone)]
struct BufferDescriptorAgency(Uuid, Weak<RefCell<StoreContainer>>, WebGl2RenderingContext);

impl BufferDescriptorAgency {
    fn raw_id(&self) -> &Uuid {
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
        let mut container = (*container).borrow_mut();

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
        container.memory_usage -= size;

        // updates most and least LRU
        // if let Some(most_recently) = container.most_recently {
        //     let most_recently = unsafe { &*most_recently };
        //     if most_recently.raw_id == lru_node.raw_id {
        //         container.most_recently = lru_node.less_recently;
        //     }
        // }
        // if let Some(least_recently) = container.least_recently {
        //     let least_recently = unsafe { &*least_recently };
        //     if least_recently.raw_id == lru_node.raw_id {
        //         container.least_recently = lru_node.more_recently;
        //     }
        // }
        // // updates self connecting LRU
        // if let Some(less_recently) = lru_node.less_recently {
        //     let less_recently = unsafe { &mut *less_recently };
        //     less_recently.more_recently = lru_node.more_recently;
        // }
        // if let Some(more_recently) = lru_node.more_recently {
        //     let more_recently = unsafe { &mut *more_recently };
        //     more_recently.less_recently = lru_node.less_recently;
        // }

        console_log!("buffer {} dropped by itself", &self.0);
    }
}

/// An enum describing current status of a descriptor.
enum BufferDescriptorStatus {
    Dropped,
    Unchanged {
        id: Rc<BufferDescriptorAgency>,
    },
    UpdateBuffer {
        old_id: Option<Rc<BufferDescriptorAgency>>,
        source: BufferSource,
    },
    UpdateSubBuffer {
        id: Rc<BufferDescriptorAgency>,
        source: BufferSource,
    },
}

/// An identifier telling [`BufferStore`] what to do with WebGlBuffer.
///
/// `BufferDescriptor` instance is cloneable, developer could reuse a
/// `BufferDescriptor` and its associated WebGlBuffer by cloning it
/// (cloning a descriptor does not create a new WebGlBuffer).
///
/// Dropping all `BufferDescriptor` instances will drop associated WebGlBuffer as well.
#[derive(Clone)]
pub struct BufferDescriptor {
    status: Rc<RefCell<BufferDescriptorStatus>>,
    usage: BufferUsage,
    restore: Option<Rc<RefCell<Box<dyn FnMut() -> BufferSource>>>>,
}

impl BufferDescriptor {
    /// Constructs a new buffer descriptor with specified [`BufferSource`] and [`BufferUsage`]
    pub fn new(source: BufferSource, usage: BufferUsage) -> Self {
        Self {
            status: Rc::new(RefCell::new(BufferDescriptorStatus::UpdateBuffer {
                old_id: None,
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

    /// Sets this buffer descriptor should not be dropped
    /// when buffer storage tries to free GPU memory.
    pub fn disable_free(&mut self) {
        self.restore = None;
    }

    /// Sets this buffer descriptor droppable
    /// when buffer storage tries to free GPU memory.
    /// A restore function returning a new [`BufferSource`]
    /// should be specified for restoring data when
    /// this buffer descriptor being used again.
    pub fn enable_free<F>(&mut self, restore: F)
    where
        F: FnMut() -> BufferSource + 'static,
    {
        self.restore = Some(Rc::new(RefCell::new(Box::new(restore))));
    }

    pub fn buffer_data(&mut self, source: BufferSource) {
        self.status.replace_with(|old| match old {
            BufferDescriptorStatus::Unchanged { id } => BufferDescriptorStatus::UpdateBuffer {
                old_id: Some(id.clone()),
                source,
            },
            BufferDescriptorStatus::UpdateBuffer { old_id, .. } => {
                BufferDescriptorStatus::UpdateBuffer {
                    old_id: old_id.clone(),
                    source,
                }
            }
            BufferDescriptorStatus::UpdateSubBuffer { id, .. } => {
                BufferDescriptorStatus::UpdateBuffer {
                    old_id: Some(id.clone()),
                    source,
                }
            }
            BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer {
                old_id: None,
                source,
            },
        });
    }

    pub fn buffer_sub_data(&mut self, source: BufferSource) {
        self.status.replace_with(|old| match old {
            BufferDescriptorStatus::Unchanged { id }
            | BufferDescriptorStatus::UpdateSubBuffer { id, .. } => {
                BufferDescriptorStatus::UpdateSubBuffer {
                    id: id.clone(),
                    source,
                }
            }
            BufferDescriptorStatus::UpdateBuffer { old_id, .. } => {
                BufferDescriptorStatus::UpdateBuffer {
                    old_id: old_id.clone(),
                    source,
                }
            }
            BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer {
                old_id: None,
                source,
            },
        });
    }
}

struct StorageItem {
    buffer: WebGlBuffer,
    status: Weak<RefCell<BufferDescriptorStatus>>,
    size: usize,
    last_used_timestamp: f64,
    lru_node: Box<LruNode>,
}

struct StoreContainer {
    store: HashMap<Uuid, StorageItem>,
    memory_usage: usize,
    most_recently: Option<*mut LruNode>,
    least_recently: Option<*mut LruNode>,
}

struct LruNode {
    raw_id: Uuid,
    less_recently: Option<*mut LruNode>,
    more_recently: Option<*mut LruNode>,
}

/// Rust lifetime limitation makes it unable to do the job inside a function.
/// Maybe somebody could find some other better solutions.
// macro_rules! to_most_recently_lru {
//     ($container: expr, $lru: expr) => {
//         // sets more recently of the incoming node to least recently if incoming node is least recently
//         if let Some(least_recently) = $container.least_recently {
//             let least_recently = unsafe { &mut *least_recently };

//             if least_recently.raw_id == $lru.raw_id {
//                 if let Some(more_recently) = $lru.more_recently {
//                     let more_recently = unsafe { &mut *more_recently };
//                     more_recently.less_recently = None;
//                     $container.least_recently = Some(more_recently);
//                 } else {
//                     // lets incoming node being the least node
//                 }
//             } else {
//                 if let (Some(more_recently), Some(less_recently)) = ($lru.more_recently, $lru.less_recently) {
//                     let more_recently = unsafe { &mut *more_recently };
//                     let less_recently = unsafe { &mut *less_recently };

//                     more_recently.less_recently = Some(less_recently);
//                     less_recently.more_recently = Some(more_recently);
//                 }
//             }
//         } else {
//             // sets incoming node as least recently if least recently node is none
//             $container.least_recently = Some($lru.as_mut());
//         }

//         // sets incoming node as most recently node if most recently node is not incoming node itself
//         if let Some(most_recently) = $container.most_recently {
//             let most_recently = unsafe { &mut *most_recently };

//             if most_recently.raw_id != $lru.raw_id {
//                 $lru.more_recently = None;
//                 $lru.less_recently = $container.most_recently;
//                 most_recently.more_recently = Some($lru.as_mut());
//                 $container.most_recently = Some($lru.as_mut());
//             }
//         } else {
//             // sets incoming node as most recently if most recently node is none
//             $container.most_recently = Some($lru.as_mut());
//         }
//     };
// }
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
                    // i am a new node.

                    if $container.most_recently.is_none() && $container.least_recently.is_none() {
                        // i am the first node in LRU cache! it is ok to just set the most and least recently to me.
                        $container.most_recently = Some($lru.as_mut());
                        $container.least_recently = Some($lru.as_mut());
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

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    max_memory: usize,
    container: Rc<RefCell<StoreContainer>>,
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited GPU memory size.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_max_memory(gl, usize::MAX)
    }

    /// Constructs a new buffer store with maximum GPU memory size.
    pub fn with_max_memory(gl: WebGl2RenderingContext, max_memory: usize) -> Self {
        Self {
            gl,
            max_memory,
            container: Rc::new(RefCell::new(StoreContainer {
                store: HashMap::new(),
                memory_usage: 0,
                most_recently: None,
                least_recently: None,
            })),
        }
    }
}

impl BufferStore {
    /// gets a [`WebGlBuffer`] and binds it to [`BufferTarget`] with the associated [`BufferDescriptor`].
    /// If WebGlBuffer not created yet, buffer store creates and one and buffers data into it, then caches it.
    /// On the next time
    pub fn use_buffer(
        &mut self,
        BufferDescriptor {
            status,
            usage,
            mut restore,
        }: BufferDescriptor,
        target: BufferTarget,
        timestamp: f64,
    ) -> Result<(), Error> {
        let mut container_guard = (*self.container).borrow_mut();
        let container_mut = &mut *container_guard;

        let mut status_guard = (*status).borrow_mut();
        let status_mut = &mut *status_guard;
        match status_mut {
            BufferDescriptorStatus::Unchanged { id, .. } => {
                let StorageItem {
                    buffer,
                    last_used_timestamp,
                    lru_node,
                    ..
                } = container_mut
                    .store
                    .get_mut(id.raw_id())
                    .ok_or(Error::BufferStorageNotFound(*id.raw_id()))?;

                self.gl.bind_buffer(target.gl_enum(), Some(buffer));

                // updates last used timestamp
                *last_used_timestamp = timestamp;
                // updates LRU
                to_most_recently_lru!(container_mut, lru_node);
            }
            BufferDescriptorStatus::Dropped | BufferDescriptorStatus::UpdateBuffer { .. } => {
                // gets buffer source
                let tmp_source: BufferSource;
                let source = match status_mut {
                    // gets buffer source from restore in Dropped if exists, or throws error otherwise.
                    BufferDescriptorStatus::Dropped => {
                        let Some(restore) = restore.borrow_mut() else {
                            return Err(Error::BufferUnexpectedDropped);
                        };
                        let mut restore = (**restore).borrow_mut();
                        let restore = restore.as_mut();

                        tmp_source = restore();
                        &tmp_source
                    }
                    // gets buffer source from status in UpdateBuffer, and delete an old buffer if exists.
                    BufferDescriptorStatus::UpdateBuffer { old_id, source } => {
                        // remove old buffer if specified
                        if let Some(StorageItem { buffer, .. }) = old_id
                            .as_ref()
                            .and_then(|id| container_mut.store.remove(id.raw_id()))
                        {
                            self.gl.delete_buffer(Some(&buffer));
                        };

                        source
                    }
                    _ => unreachable!(),
                };

                // creates buffer and buffers data into it
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };

                // buffers data
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_data(&self.gl, target, usage);

                // gets and updates memory usage
                let size = self
                    .gl
                    .get_buffer_parameter(target.gl_enum(), WebGl2RenderingContext::BUFFER_SIZE)
                    .as_f64()
                    .unwrap() as usize;
                container_mut.memory_usage += size;

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
                        buffer,
                        size,
                        status: Rc::downgrade(&status),
                        last_used_timestamp: timestamp,
                        lru_node,
                    },
                );

                // replace descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged {
                    id: Rc::new(BufferDescriptorAgency(
                        raw_id,
                        Rc::downgrade(&self.container),
                        self.gl.clone(),
                    )),
                };
            }
            BufferDescriptorStatus::UpdateSubBuffer { id, source, .. } => {
                let Some(StorageItem {
                    buffer,
                    last_used_timestamp,
                    lru_node,
                    ..
                }) = container_mut.store.get_mut(id.raw_id())
                else {
                    return Err(Error::BufferStorageNotFound(*id.raw_id()));
                };

                // binds and buffers sub data
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_sub_data(&self.gl, target);

                // replace descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged { id: id.clone() };

                // updates last used timestamp
                *last_used_timestamp = timestamp;

                // updates LRU
                to_most_recently_lru!(container_mut, lru_node);
            }
        };

        drop(container_guard);
        drop(status_guard);

        self.free();

        Ok(())
    }

    /// Tries to free memory if current memory usage exceeds the maximum memory limitation.
    fn free(&mut self) {
        let container = (*self.container).borrow_mut();

        if container.memory_usage < self.max_memory {
            return;
        }

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

/// Deletes all WebGlBuffer from WebGL runtime and
/// changes all buffer descriptor status to [`BufferDescriptorStatus::Dropped`]
/// when buffer store drops.
impl Drop for BufferStore {
    fn drop(&mut self) {
        let gl = &self.gl;
        let mut container = (*self.container).borrow_mut();
        let container = &mut *container;
        let memory_usage = &mut container.memory_usage;
        let store = &mut container.store;

        store.iter().for_each(
            |(
                _,
                StorageItem {
                    buffer,
                    status,
                    size,
                    ..
                },
            )| {
                gl.delete_buffer(Some(&buffer));
                *memory_usage -= size;

                status.upgrade().map(|status| {
                    *(*status).borrow_mut() = BufferDescriptorStatus::Dropped;
                });

                // store dropped, no need to update LRU anymore
            },
        );
    }
}
