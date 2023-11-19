use std::{cell::RefCell, collections::HashMap};

use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl BufferTarget {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BufferTarget::Buffer => WebGl2RenderingContext::ARRAY_BUFFER,
            BufferTarget::ElementArrayBuffer => WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            BufferTarget::CopyReadBuffer => WebGl2RenderingContext::COPY_READ_BUFFER,
            BufferTarget::CopyWriteBuffer => WebGl2RenderingContext::COPY_WRITE_BUFFER,
            BufferTarget::TransformFeedbackBuffer => {
                WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER
            }
            BufferTarget::UniformBuffer => WebGl2RenderingContext::UNIFORM_BUFFER,
            BufferTarget::PixelPackBuffer => WebGl2RenderingContext::PIXEL_PACK_BUFFER,
            BufferTarget::PixelUnpackBuffer => WebGl2RenderingContext::PIXEL_UNPACK_BUFFER,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferComponentSize {
    One,
    Two,
    Three,
    Four,
}

impl BufferComponentSize {
    pub fn to_i32(&self) -> i32 {
        match self {
            BufferComponentSize::One => 1,
            BufferComponentSize::Two => 2,
            BufferComponentSize::Three => 3,
            BufferComponentSize::Four => 4,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BufferDataType::Float => WebGl2RenderingContext::FLOAT,
            BufferDataType::Byte => WebGl2RenderingContext::BYTE,
            BufferDataType::Short => WebGl2RenderingContext::SHORT,
            BufferDataType::Int => WebGl2RenderingContext::INT,
            BufferDataType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            BufferDataType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            BufferDataType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
            BufferDataType::HalfFloat => WebGl2RenderingContext::HALF_FLOAT,
            BufferDataType::Int_2_10_10_10_Rev => WebGl2RenderingContext::INT_2_10_10_10_REV,
            BufferDataType::UnsignedInt_2_10_10_10_Rev => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl BufferUsage {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BufferUsage::StaticDraw => WebGl2RenderingContext::STATIC_DRAW,
            BufferUsage::DynamicDraw => WebGl2RenderingContext::DYNAMIC_DRAW,
            BufferUsage::StreamDraw => WebGl2RenderingContext::STREAM_DRAW,
            BufferUsage::StaticRead => WebGl2RenderingContext::STATIC_READ,
            BufferUsage::DynamicRead => WebGl2RenderingContext::DYNAMIC_READ,
            BufferUsage::StreamRead => WebGl2RenderingContext::STREAM_READ,
            BufferUsage::StaticCopy => WebGl2RenderingContext::STATIC_COPY,
            BufferUsage::DynamicCopy => WebGl2RenderingContext::DYNAMIC_COPY,
            BufferUsage::StreamCopy => WebGl2RenderingContext::STATIC_COPY,
        }
    }
}

enum BufferData {
    Preallocate {
        size: i32,
    },
    FromBinary {
        data: Box<dyn AsRef<[u8]>>,
        src_byte_offset: u32,
        src_byte_length: u32,
    },
}

struct BufferSubData {
    data: Box<dyn AsRef<[u8]>>,
    dst_byte_offset: i32,
    src_byte_offset: u32,
    src_byte_length: u32,
}

enum BufferStatus {
    Unchanged { id: Uuid },
    UpdateBuffer { id: Option<Uuid>, data: BufferData },
    UpdateSubBuffer { id: Uuid, data: BufferSubData },
}

pub struct BufferDescriptor {
    status: RefCell<BufferStatus>,
    usage: BufferUsage,
}

impl BufferDescriptor {
    pub fn preallocate(size: i32, usage: BufferUsage) -> Self {
        Self {
            status: RefCell::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::Preallocate { size },
            }),
            usage,
        }
    }

    pub fn from_binary<D: AsRef<[u8]> + 'static>(
        data: D,
        src_byte_offset: u32,
        src_byte_length: u32,
        usage: BufferUsage,
    ) -> Self {
        Self {
            status: RefCell::new(BufferStatus::UpdateBuffer {
                id: None,
                data: BufferData::FromBinary {
                    data: Box::new(data),
                    src_byte_offset,
                    src_byte_length,
                },
            }),
            usage,
        }
    }

    pub fn buffer_data<D: AsRef<[u8]> + 'static>(
        &mut self,
        data: D,
        src_byte_offset: u32,
        src_byte_length: u32,
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
        dst_byte_offset: i32,
        src_byte_offset: u32,
        src_byte_length: u32,
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

                self.gl.bind_buffer(target.to_gl_enum(), Some(&buffer));
                match data {
                    BufferData::Preallocate { size } => {
                        self.gl.buffer_data_with_i32(
                            target.to_gl_enum(),
                            *size,
                            usage.to_gl_enum(),
                        );
                    }
                    BufferData::FromBinary {
                        data,
                        src_byte_offset,
                        src_byte_length,
                    } => self.gl.buffer_data_with_u8_array_and_src_offset_and_length(
                        target.to_gl_enum(),
                        data.as_ref().as_ref(),
                        usage.to_gl_enum(),
                        *src_byte_offset,
                        *src_byte_length,
                    ),
                };
                self.gl.bind_buffer(target.to_gl_enum(), None);

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

                self.gl.bind_buffer(target.to_gl_enum(), Some(&buffer));
                let BufferSubData {
                    data,
                    dst_byte_offset,
                    src_byte_offset,
                    src_byte_length,
                } = data;
                self.gl
                    .buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                        target.to_gl_enum(),
                        *dst_byte_offset,
                        data.as_ref().as_ref(),
                        *src_byte_offset,
                        *src_byte_length,
                    );
                self.gl.bind_buffer(target.to_gl_enum(), None);

                // replace descriptor status
                *status = BufferStatus::Unchanged { id: id.clone() };

                Ok(buffer)
            }
        }
    }
}
