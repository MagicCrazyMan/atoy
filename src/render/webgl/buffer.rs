use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use uuid::Uuid;
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
enum BufferSource {
    Preallocate {
        size: GLsizeiptr,
    },
    FromBinary {
        data: Box<dyn AsRef<[u8]>>,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromInt8Array {
        data: Int8Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromUint8Array {
        data: Uint8Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromUint8ClampedArray {
        data: Uint8ClampedArray,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromInt16Array {
        data: Int16Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromUint16Array {
        data: Uint16Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromInt32Array {
        data: Int32Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromUint32Array {
        data: Uint32Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromFloat32Array {
        data: Float32Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromFloat64Array {
        data: Float64Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromBigInt64Array {
        data: BigInt64Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
    FromBigUint64Array {
        data: BigUint64Array,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
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
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromUint8Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromUint8ClampedArray {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromInt16Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromUint16Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromInt32Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromUint32Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromFloat32Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromFloat64Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromBigInt64Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
            BufferSource::FromBigUint64Array {
                data,
                dst_byte_offset,
                src_byte_offset,
                src_byte_length,
            } => (
                data as &Object,
                *dst_byte_offset,
                *src_byte_offset,
                *src_byte_length,
            ),
        }
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        let target = target.gl_enum();
        let usage = usage.gl_enum();
        match self {
            BufferSource::Preallocate { size } => gl.buffer_data_with_i32(target, *size, usage),
            BufferSource::FromBinary {
                data,
                src_byte_offset,
                src_byte_length,
                ..
            } => gl.buffer_data_with_u8_array_and_src_offset_and_length(
                target,
                data.as_ref().as_ref(),
                usage,
                *src_byte_offset,
                *src_byte_length,
            ),
            _ => {
                let (data, _, src_byte_offset, src_byte_length) = self.collect_typed_array_buffer();
                gl.buffer_data_with_array_buffer_view_and_src_offset_and_length(
                    target,
                    data,
                    usage,
                    src_byte_offset,
                    src_byte_length,
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
                src_byte_offset,
                src_byte_length,
            } => gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                target,
                *dst_byte_offset,
                data.as_ref().as_ref(),
                *src_byte_offset,
                *src_byte_length,
            ),
            _ => {
                let (data, dst_byte_offset, src_byte_offset, src_byte_length) =
                    self.collect_typed_array_buffer();
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target,
                    dst_byte_offset,
                    data,
                    src_byte_offset,
                    src_byte_length,
                )
            }
        }
    }
}

enum BufferDescriptorAgency {
    Dropped,
    Unchanged {
        id: Uuid,
        runtime: Option<(Weak<RefCell<StoreContainer>>, WebGl2RenderingContext)>,
    },
    UpdateBuffer {
        old_id: Option<Uuid>,
        runtime: Option<(Weak<RefCell<StoreContainer>>, WebGl2RenderingContext)>,
        source: BufferSource,
    },
    UpdateSubBuffer {
        id: Uuid,
        runtime: Option<(Weak<RefCell<StoreContainer>>, WebGl2RenderingContext)>,
        source: BufferSource,
    },
}

/// Deletes associated WebGlBuffer from store(if exists) when descriptor drops.
impl Drop for BufferDescriptorAgency {
    fn drop(&mut self) {
        let v = match self {
            BufferDescriptorAgency::Dropped => return,
            BufferDescriptorAgency::Unchanged { id, runtime } => (Some(id), runtime),
            BufferDescriptorAgency::UpdateBuffer {
                old_id, runtime, ..
            } => (old_id.as_mut(), runtime),
            BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => (Some(id), runtime),
        };
        let (Some(id), Some((store, gl))) = v else {
            return;
        };
        let Some(store) = store.upgrade() else {
            return;
        };
        let Some(mut store) = store.try_borrow_mut().ok() else {
            // if borrowed, buffer is deleting from buffer store, thus, do nothing here.
            return;
        };

        let Some((buffer, _)) = store.borrow_mut().remove(id) else {
            return;
        };

        gl.delete_buffer(Some(&buffer));
    }
}

#[derive(Clone)]
#[crate::render::webgl::wasm_bindgen]
pub struct BufferDescriptor {
    status: Rc<RefCell<BufferDescriptorAgency>>,
    usage: BufferUsage,
}

impl BufferDescriptor {
    pub fn preallocate(size: GLsizeiptr, usage: BufferUsage) -> Self {
        Self {
            status: Rc::new(RefCell::new(BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                source: BufferSource::Preallocate { size },
            })),
            usage,
        }
    }

    pub fn from_binary<D: AsRef<[u8]> + 'static>(
        data: D,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
        usage: BufferUsage,
    ) -> Self {
        Self {
            status: Rc::new(RefCell::new(BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                source: BufferSource::FromBinary {
                    data: Box::new(data),
                    dst_byte_offset: 0,
                    src_byte_offset,
                    src_byte_length,
                },
            })),
            usage,
        }
    }

    pub fn buffer_binary<D: AsRef<[u8]> + 'static>(
        &mut self,
        data: D,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    ) {
        self.status.replace_with(|old| match old {
            BufferDescriptorAgency::Unchanged { id, runtime } => {
                BufferDescriptorAgency::UpdateBuffer {
                    old_id: Some(*id),
                    runtime: runtime.clone(),
                    source: BufferSource::FromBinary {
                        data: Box::new(data),
                        dst_byte_offset: 0,
                        src_byte_offset,
                        src_byte_length,
                    },
                }
            }
            BufferDescriptorAgency::UpdateBuffer {
                old_id: id,
                runtime,
                ..
            } => BufferDescriptorAgency::UpdateBuffer {
                old_id: id.clone(),
                runtime: runtime.clone(),
                source: BufferSource::FromBinary {
                    data: Box::new(data),
                    dst_byte_offset: 0,
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => {
                BufferDescriptorAgency::UpdateBuffer {
                    old_id: Some(*id),
                    runtime: runtime.clone(),
                    source: BufferSource::FromBinary {
                        data: Box::new(data),
                        dst_byte_offset: 0,
                        src_byte_offset,
                        src_byte_length,
                    },
                }
            }
            BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                source: BufferSource::FromBinary {
                    data: Box::new(data),
                    dst_byte_offset: 0,
                    src_byte_offset,
                    src_byte_length,
                },
            },
        });
    }

    pub fn buffer_sub_binary<D: AsRef<[u8]> + 'static>(
        &mut self,
        data: D,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    ) {
        self.status.replace_with(|old| match old {
            BufferDescriptorAgency::Unchanged { id, runtime }
            | BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => {
                BufferDescriptorAgency::UpdateSubBuffer {
                    id: *id,
                    runtime: runtime.clone(),
                    source: BufferSource::FromBinary {
                        data: Box::new(data),
                        dst_byte_offset,
                        src_byte_offset,
                        src_byte_length,
                    },
                }
            }
            BufferDescriptorAgency::UpdateBuffer {
                old_id, runtime, ..
            } => BufferDescriptorAgency::UpdateBuffer {
                old_id: old_id.clone(),
                runtime: runtime.clone(),
                source: BufferSource::FromBinary {
                    data: Box::new(data),
                    dst_byte_offset: 0,
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                source: BufferSource::FromBinary {
                    data: Box::new(data),
                    dst_byte_offset: 0,
                    src_byte_offset,
                    src_byte_length,
                },
            },
        });
    }
}

macro_rules! impl_typed_array {
    ($(($from: ident, $buffer: ident, $buffer_sub: ident, $source: tt, $kind: ident)),+) => {
        impl BufferDescriptor {
            $(
                pub fn $from(
                    data: $source,
                    src_byte_offset: GLuint,
                    src_byte_length: GLuint,
                    usage: BufferUsage,
                ) -> Self {
                    Self {
                        status: Rc::new(RefCell::new(BufferDescriptorAgency::UpdateBuffer {
                            old_id: None,
                            runtime: None,
                            source: BufferSource::$kind {
                                data,
                                dst_byte_offset: 0,
                                src_byte_offset,
                                src_byte_length,
                            },
                        })),
                        usage,
                    }
                }

                pub fn $buffer(
                    &mut self,
                    data: $source,
                    src_byte_offset: GLuint,
                    src_byte_length: GLuint,
                ) {
                    self.status.replace_with(|old| match old {
                        BufferDescriptorAgency::Unchanged { id, runtime } => {
                            BufferDescriptorAgency::UpdateBuffer {
                                old_id: Some(*id),
                                runtime: runtime.clone(),
                                source: BufferSource::$kind {
                                    data,
                                    dst_byte_offset: 0,
                                    src_byte_offset,
                                    src_byte_length,
                                },
                            }
                        }
                        BufferDescriptorAgency::UpdateBuffer {
                            old_id: id,
                            runtime,
                            ..
                        } => BufferDescriptorAgency::UpdateBuffer {
                            old_id: id.clone(),
                            runtime: runtime.clone(),
                            source: BufferSource::$kind {
                                data,
                                dst_byte_offset: 0,
                                src_byte_offset,
                                src_byte_length,
                            },
                        },
                        BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => {
                            BufferDescriptorAgency::UpdateBuffer {
                                old_id: Some(*id),
                                runtime: runtime.clone(),
                                source: BufferSource::$kind {
                                    data,
                                    dst_byte_offset: 0,
                                    src_byte_offset,
                                    src_byte_length,
                                },
                            }
                        }
                        BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                            old_id: None,
                            runtime: None,
                            source: BufferSource::$kind {
                                data,
                                dst_byte_offset: 0,
                                src_byte_offset,
                                src_byte_length,
                            },
                        },
                    });
                }

                pub fn $buffer_sub(
                    &mut self,
                    data: $source,
                    dst_byte_offset: GLintptr,
                    src_byte_offset: GLuint,
                    src_byte_length: GLuint,
                ) {
                    self.status.replace_with(|old| match old {
                        BufferDescriptorAgency::Unchanged { id, runtime }
                        | BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => {
                            BufferDescriptorAgency::UpdateSubBuffer {
                                id: *id,
                                runtime: runtime.clone(),
                                source: BufferSource::$kind {
                                    data,
                                    dst_byte_offset,
                                    src_byte_offset,
                                    src_byte_length,
                                },
                            }
                        }
                        BufferDescriptorAgency::UpdateBuffer {
                            old_id, runtime, ..
                        } => BufferDescriptorAgency::UpdateBuffer {
                            old_id: old_id.clone(),
                            runtime: runtime.clone(),
                            source: BufferSource::$kind {
                                data,
                                dst_byte_offset: 0,
                                src_byte_offset,
                                src_byte_length,
                            },
                        },
                        BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                            old_id: None,
                            runtime: None,
                            source: BufferSource::$kind {
                                data,
                                dst_byte_offset: 0,
                                src_byte_offset,
                                src_byte_length,
                            },
                        },
                    });
                }
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

type StoreContainer = HashMap<Uuid, (WebGlBuffer, Weak<RefCell<BufferDescriptorAgency>>)>;

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
    pub fn use_buffer(
        &mut self,
        BufferDescriptor { status, usage }: &BufferDescriptor,
        target: BufferTarget,
    ) -> Result<WebGlBuffer, Error> {
        let mut status_mut = (**status).borrow_mut();
        match &mut *status_mut {
            BufferDescriptorAgency::Dropped => Err(Error::BufferUnexpectedDropped),
            BufferDescriptorAgency::Unchanged { id, .. } => self
                .store
                .borrow()
                .get(id)
                .map(|(buffer, _)| buffer.clone())
                .ok_or(Error::BufferStorageNotFound(id.clone())),
            BufferDescriptorAgency::UpdateBuffer {
                old_id: id, source, ..
            } => {
                let mut store = (*self.store).borrow_mut();

                // remove old buffer if specified
                if let Some((buffer, _)) = id.and_then(|id| store.remove(&id)) {
                    self.gl.delete_buffer(Some(&buffer));
                };

                // creates buffer and buffers data into it
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };

                // buffer data
                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_data(&self.gl, target, *usage);
                self.gl.bind_buffer(target.gl_enum(), None);

                // stores it
                let id = Uuid::new_v4();
                store.insert(id, (buffer.clone(), Rc::downgrade(&status)));

                // replace descriptor status
                *status_mut = BufferDescriptorAgency::Unchanged {
                    id,
                    runtime: Some((Rc::downgrade(&self.store), self.gl.clone())),
                };

                Ok(buffer)
            }
            BufferDescriptorAgency::UpdateSubBuffer { id, source, .. } => {
                let store = self.store.borrow();

                let Some((buffer, _)) = store.get(id) else {
                    return Err(Error::BufferStorageNotFound(id.clone()));
                };

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                source.buffer_sub_data(&self.gl, target);
                self.gl.bind_buffer(target.gl_enum(), None);

                // replace descriptor status
                *status_mut = BufferDescriptorAgency::Unchanged {
                    id: id.clone(),
                    runtime: Some((Rc::downgrade(&self.store), self.gl.clone())),
                };

                Ok(buffer.clone())
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
            .borrow_mut()
            .drain()
            .for_each(|(_, (buffer, status))| {
                gl.delete_buffer(Some(&buffer));
                status.upgrade().map(|status| {
                    *(*status).borrow_mut() = BufferDescriptorAgency::Dropped;
                });
            });
    }
}
