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
}

macro_rules! impl_typed_array {
    ($(($from: ident, $buffer: ident, $buffer_sub: ident, $source: tt, $kind: ident)),+) => {
        impl BufferSource {
            $(
                pub fn $from(
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

                // pub fn $buffer(
                //     &mut self,
                //     data: $source,
                //     src_offset: GLuint,
                //     src_length: GLuint,
                // ) {
                //     self.status.replace_with(|old| match old {
                //         BufferDescriptorStatus::Unchanged { id } => {
                //             BufferDescriptorStatus::UpdateBuffer {
                //                 old_id: Some(id.clone()),
                //                 source: BufferSource::$kind {
                //                     data,
                //                     dst_byte_offset: 0,
                //                     src_offset,
                //                     src_length,
                //                 },
                //             }
                //         }
                //         BufferDescriptorStatus::UpdateBuffer {
                //             old_id, ..
                //         } => BufferDescriptorStatus::UpdateBuffer {
                //             old_id: old_id.clone(),
                //             source: BufferSource::$kind {
                //                 data,
                //                 dst_byte_offset: 0,
                //                 src_offset,
                //                 src_length,
                //             },
                //         },
                //         BufferDescriptorStatus::UpdateSubBuffer { id, .. } => {
                //             BufferDescriptorStatus::UpdateBuffer {
                //                 old_id: Some(id.clone()),
                //                 source: BufferSource::$kind {
                //                     data,
                //                     dst_byte_offset: 0,
                //                     src_offset,
                //                     src_length,
                //                 },
                //             }
                //         }
                //         BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer {
                //             old_id: None,
                //             source: BufferSource::$kind {
                //                 data,
                //                 dst_byte_offset: 0,
                //                 src_offset,
                //                 src_length,
                //             },
                //         },
                //     });
                // }

                // pub fn $buffer_sub(
                //     &mut self,
                //     data: $source,
                //     dst_byte_offset: GLintptr,
                //     src_offset: GLuint,
                //     src_length: GLuint,
                // ) {
                //     self.status.replace_with(|old| match old {
                //         BufferDescriptorStatus::Unchanged { id }
                //         | BufferDescriptorStatus::UpdateSubBuffer { id, .. } => {
                //             BufferDescriptorStatus::UpdateSubBuffer {
                //                 id: id.clone(),
                //                 source: BufferSource::$kind {
                //                     data,
                //                     dst_byte_offset,
                //                     src_offset,
                //                     src_length,
                //                 },
                //             }
                //         }
                //         BufferDescriptorStatus::UpdateBuffer {
                //             old_id, ..
                //         } => BufferDescriptorStatus::UpdateBuffer {
                //             old_id: old_id.clone(),
                //             source: BufferSource::$kind {
                //                 data,
                //                 dst_byte_offset: 0,
                //                 src_offset,
                //                 src_length,
                //             },
                //         },
                //         BufferDescriptorStatus::Dropped => BufferDescriptorStatus::UpdateBuffer {
                //             old_id: None,
                //             source: BufferSource::$kind {
                //                 data,
                //                 dst_byte_offset: 0,
                //                 src_offset,
                //                 src_length,
                //             },
                //         },
                //     });
                // }
            )+
        }
    };
}

impl_typed_array! {
    (from_int8_array, buffer_int8_array, buffer_sub_int8_array, Int8Array, FromInt8Array),
    (from_uint8_array, buffer_uint8_array, buffer_sub_uint8_array, Uint8Array, FromUint8Array),
    (from_uint8_clamped_array, buffer_uint8_clamped_array, buffer_sub_uint8_clamped_array, Uint8ClampedArray, FromUint8ClampedArray),
    (from_int16_array, buffer_int16_array, buffer_sub_int16_array, Int16Array, FromInt16Array),
    (from_uint16_array, buffer_uint16_array, buffer_sub_uint16_array, Uint16Array, FromUint16Array),
    (from_int32_array, buffer_int32_array, buffer_sub_int32_array, Int32Array, FromInt32Array),
    (from_uint32_array, buffer_uint32_array, buffer_sub_uint32_array, Uint32Array, FromUint32Array),
    (from_float32_array, buffer_float32_array, buffer_sub_float32_array, Float32Array, FromFloat32Array),
    (from_float64_array, buffer_float64_array, buffer_sub_float64_array, Float64Array, FromFloat64Array),
    (from_big_int64_array, buffer_big_int64_array, buffer_sub_big_int64_array, BigInt64Array, FromBigInt64Array),
    (from_big_uint64_array, buffer_big_uint64_array, buffer_sub_big_uint64_array, BigUint64Array, FromBigUint64Array)
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
        let Some(store) = self.1.upgrade() else {
            return;
        };
        let mut store = (*store).borrow_mut();

        let Some((buffer, _)) = store.remove(&self.0) else {
            return;
        };

        self.2.delete_buffer(Some(&buffer));

        console_log!("buffer {} dropped", &self.0);
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

type StoreContainer = HashMap<Uuid, (WebGlBuffer, Weak<RefCell<BufferDescriptorStatus>>)>;

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    store: Rc<RefCell<StoreContainer>>,
}

impl BufferStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl BufferStore {
    /// gets a [`WebGlBuffer`] and binds it to [`BufferTarget`] with the associated [`BufferDescriptor`].
    /// If WebGlBuffer not created yet, buffer store creates and one and buffers data into it, then caches it.
    /// On the next time
    pub fn use_buffer(
        &mut self,
        BufferDescriptor { status, usage, .. }: &BufferDescriptor,
        target: BufferTarget,
    ) -> Result<(), Error> {
        let mut status_mut = (**status).borrow_mut();
        match &mut *status_mut {
            BufferDescriptorStatus::Dropped => Err(Error::BufferUnexpectedDropped),
            BufferDescriptorStatus::Unchanged { id, .. } => {
                let buffer = self
                    .store
                    .borrow()
                    .get(id.raw_id())
                    .map(|(buffer, _)| buffer.clone())
                    .ok_or(Error::BufferStorageNotFound(*id.raw_id()))?;

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                Ok(())
            }
            BufferDescriptorStatus::UpdateBuffer { old_id, source, .. } => {
                let mut store = (*self.store).borrow_mut();

                // remove old buffer if specified
                if let Some((buffer, _)) = old_id.as_ref().and_then(|id| store.remove(id.raw_id()))
                {
                    self.gl.delete_buffer(Some(&buffer));
                };

                // creates buffer and buffers data into it
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };

                // buffer data
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_data(&self.gl, target, *usage);

                // stores it
                let raw_id = Uuid::new_v4();
                store.insert(raw_id, (buffer.clone(), Rc::downgrade(&status)));

                // replace descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged {
                    id: Rc::new(BufferDescriptorAgency(
                        raw_id,
                        Rc::downgrade(&self.store),
                        self.gl.clone(),
                    )),
                };

                Ok(())
            }
            BufferDescriptorStatus::UpdateSubBuffer { id, source, .. } => {
                let store = self.store.borrow();

                let Some((buffer, _)) = store.get(id.raw_id()) else {
                    return Err(Error::BufferStorageNotFound(*id.raw_id()));
                };

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_sub_data(&self.gl, target);

                // replace descriptor status
                *status_mut = BufferDescriptorStatus::Unchanged { id: id.clone() };

                Ok(())
            }
        }
    }
}

/// Deletes all WebGlBuffer from WebGL runtime and
/// changes all buffer descriptor status to [`BufferDescriptorStatus::Dropped`]
/// when buffer store drops.
impl Drop for BufferStore {
    fn drop(&mut self) {
        let gl = &self.gl;
        (*self.store)
            .borrow()
            .iter()
            .for_each(|(_, (buffer, status))| {
                gl.delete_buffer(Some(&buffer));
                status.upgrade().map(|status| {
                    *(*status).borrow_mut() = BufferDescriptorStatus::Dropped;
                });
            });
    }
}
