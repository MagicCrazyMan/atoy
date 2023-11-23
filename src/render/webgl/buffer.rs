use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::conversion::{GLintptr, GLsizeiptr, GLuint, ToGlEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    Buffer,
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
    pub fn bytes_length(&self) -> i32 {
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

impl Debug for BufferData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preallocate { size } => {
                f.debug_struct("Preallocate").field("size", size).finish()
            }
            Self::FromBinary {
                data,
                src_byte_offset,
                src_byte_length,
            } => f
                .debug_struct("FromBinary")
                .field("data_length", &data.as_ref().as_ref().len())
                .field("src_byte_offset", src_byte_offset)
                .field("src_byte_length", src_byte_length)
                .finish(),
        }
    }
}

struct BufferSubData {
    data: Box<dyn AsRef<[u8]>>,
    dst_byte_offset: GLintptr,
    src_byte_offset: GLuint,
    src_byte_length: GLuint,
}

impl Debug for BufferSubData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferSubData")
            .field("data_length", &self.data.as_ref().as_ref().len())
            .field("dst_byte_offset", &self.dst_byte_offset)
            .field("src_byte_offset", &self.src_byte_offset)
            .field("src_byte_length", &self.src_byte_length)
            .finish()
    }
}

#[derive(Debug)]
enum BufferStatus {
    Unchanged { id: Uuid },
    UpdateBuffer { id: Option<Uuid>, data: BufferData },
    UpdateSubBuffer { id: Uuid, data: BufferSubData },
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    status: Rc<RefCell<BufferStatus>>,
    usage: BufferUsage,
}

impl BufferDescriptor {
    pub fn preallocate(size: GLsizeiptr, usage: BufferUsage) -> Self {
        Self {
            status: Rc::new(RefCell::new(BufferStatus::UpdateBuffer {
                id: None,
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
            status: Rc::new(RefCell::new(BufferStatus::UpdateBuffer {
                id: None,
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
        let new_status = match *self.status.borrow() {
            BufferStatus::Unchanged { id } => BufferStatus::UpdateBuffer {
                id: Some(id),
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferStatus::UpdateBuffer { id, .. } => BufferStatus::UpdateBuffer {
                id,
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
            BufferStatus::UpdateSubBuffer { id, .. } => BufferStatus::UpdateBuffer {
                id: Some(id),
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
        };

        *self.status.borrow_mut() = new_status;
    }

    pub fn buffer_sub_data<D: AsRef<[u8]> + 'static>(
        &mut self,
        data: D,
        dst_byte_offset: GLintptr,
        src_byte_offset: GLuint,
        src_byte_length: GLuint,
    ) {
        let new_status = match *self.status.borrow() {
            BufferStatus::Unchanged { id } | BufferStatus::UpdateSubBuffer { id, .. } => {
                BufferStatus::UpdateSubBuffer {
                    id,
                    data: BufferSubData {
                        data: Box::new(data),
                        dst_byte_offset,
                        src_byte_offset,
                        src_byte_length,
                    },
                }
            }
            BufferStatus::UpdateBuffer { id, .. } => BufferStatus::UpdateBuffer {
                id,
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            },
        };

        *self.status.borrow_mut() = new_status;
    }
}

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    store: HashMap<Uuid, WebGlBuffer>,
}

impl BufferStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: HashMap::new(),
        }
    }
}

impl BufferStore {
    pub fn buffer_or_create(
        &mut self,
        BufferDescriptor { status, usage }: &BufferDescriptor,
        target: BufferTarget,
    ) -> Result<&WebGlBuffer, String> {
        let mut status = status.borrow_mut();
        match &*status {
            BufferStatus::Unchanged { id } => {
                let Some(buffer) = self.store.get(id) else {
                    return Err(format!("failed to get buffer with id {}", id));
                };

                Ok(buffer)
            }
            BufferStatus::UpdateBuffer { id, data } => {
                // remove old buffer if specified
                if let Some(buffer) = id.and_then(|id| self.store.remove(&id)) {
                    self.gl.delete_buffer(Some(&buffer));
                };

                // creates buffer and buffers data into it
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(String::from("failed to create WebGL buffer"));
                };

                self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                match data {
                    BufferData::Preallocate { size } => {
                        self.gl.buffer_data_with_i32(
                            target.gl_enum(),
                            *size,
                            usage.gl_enum(),
                        );
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
                let buffer = self.store.entry(id).or_insert(buffer);

                // replace descriptor status
                *status = BufferStatus::Unchanged { id };

                Ok(buffer)
            }
            BufferStatus::UpdateSubBuffer { id, data } => {
                let Some(buffer) = self.store.get(id) else {
                    return Err(format!("failed to get buffer with id {}", id));
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
                *status = BufferStatus::Unchanged { id: id.clone() };

                Ok(buffer)
            }
        }
    }
}
