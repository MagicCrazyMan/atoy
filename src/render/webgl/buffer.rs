use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use uuid::Uuid;
use wasm_bindgen_test::console_log;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

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

enum BufferData {
    Preallocate {
        size: GLsizeiptr,
    },
    FromBinary {
        data: Box<dyn AsRef<[u8]>>,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    },
}

struct BufferSubData {
    data: Box<dyn AsRef<[u8]>>,
    dst_byte_offset: GLintptr,
    src_byte_offset: GLuint,
    src_byte_length: GLuint,
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
        data: BufferData,
    },
    UpdateSubBuffer {
        id: Uuid,
        runtime: Option<(Weak<RefCell<StoreContainer>>, WebGl2RenderingContext)>,
        data: BufferSubData,
    },
}

/// Deletes associated WebGlBuffer from store(if exists) when descriptor drops.
impl Drop for BufferDescriptorAgency {
    fn drop(&mut self) {
        match self {
            BufferDescriptorAgency::Dropped => {
                console_log!("buffer descriptor in Dropped status")
            }
            BufferDescriptorAgency::Unchanged { id, .. } => {
                console_log!("buffer descriptor {} in Unchanged status", id)
            }
            BufferDescriptorAgency::UpdateBuffer { old_id, .. } => {
                console_log!("buffer descriptor {:?} in UpdateBuffer status", old_id)
            }
            BufferDescriptorAgency::UpdateSubBuffer { id, .. } => {
                console_log!("buffer descriptor {} in UpdateSubBuffer status", id)
            }
        }

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

        let id = *id;

        gl.delete_buffer(Some(&buffer));

        console_log!("buffer descriptor {} dropped", id);
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
                data: BufferData::Preallocate { size },
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
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            })),
            usage,
        }
    }

    pub fn buffer_data<D: AsRef<[u8]> + 'static>(
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
                    data: BufferData::FromBinary {
                        data: Box::new(data),
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
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferDescriptorAgency::UpdateSubBuffer { id, runtime, .. } => {
                BufferDescriptorAgency::UpdateBuffer {
                    old_id: Some(*id),
                    runtime: runtime.clone(),
                    data: BufferData::FromBinary {
                        data: Box::new(data),
                        src_byte_offset,
                        src_byte_length,
                    },
                }
            }
            BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
        });
    }

    pub fn buffer_sub_data<D: AsRef<[u8]> + 'static>(
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
                    data: BufferSubData {
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
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferDescriptorAgency::Dropped => BufferDescriptorAgency::UpdateBuffer {
                old_id: None,
                runtime: None,
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
        });
    }
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
                .ok_or(Error::BufferStorageNotFount(id.clone())),
            BufferDescriptorAgency::UpdateBuffer {
                old_id: id, data, ..
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

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                match data {
                    BufferData::Preallocate { size } => {
                        self.gl
                            .buffer_data_with_i32(target.gl_enum(), *size, usage.gl_enum());
                    }
                    BufferData::FromBinary {
                        data,
                        src_byte_offset,
                        src_byte_length,
                    } => self.gl.buffer_data_with_u8_array_and_src_offset_and_length(
                        target.gl_enum(),
                        data.as_ref().as_ref(),
                        usage.gl_enum(),
                        *src_byte_offset,
                        *src_byte_length,
                    ),
                };
                self.gl.bind_buffer(target.gl_enum(), None);

                // stores it
                let id = Uuid::new_v4();
                store.insert(id, (buffer.clone(), Rc::downgrade(&status)));

                // replace descriptor status
                *status_mut = BufferDescriptorAgency::Unchanged {
                    id,
                    runtime: Some((Rc::downgrade(&self.store), self.gl.clone())),
                };

                console_log!("buffer descriptor {} update buffer", id);

                Ok(buffer)
            }
            BufferDescriptorAgency::UpdateSubBuffer { id, data, .. } => {
                let store = self.store.borrow();

                let Some((buffer, _)) = store.get(id) else {
                    return Err(Error::BufferStorageNotFount(id.clone()));
                };

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                let BufferSubData {
                    data,
                    dst_byte_offset,
                    src_byte_offset,
                    src_byte_length,
                } = data;
                self.gl
                    .buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                        target.gl_enum(),
                        *dst_byte_offset,
                        data.as_ref().as_ref(),
                        *src_byte_offset,
                        *src_byte_length,
                    );
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
        console_log!("buffer store dropping");
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
        console_log!("buffer store dropped");
    }
}
