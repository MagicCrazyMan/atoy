use std::{cell::RefCell, collections::HashMap};

use uuid::Uuid;
use wasm_bindgen::JsError;
use wasm_bindgen_test::console_log;
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

pub enum BufferData {
    FillEmpty {
        size: i32,
    },
    FillData {
        data: Box<dyn AsRef<[u8]>>,
        src_byte_offset: u32,
        src_byte_length: u32,
    },
}

impl BufferData {
    pub fn fill_empty(size: i32) -> Self {
        Self::FillEmpty { size }
    }

    pub fn fill_data<D: AsRef<[u8]> + 'static>(
        data: D,
        src_byte_offset: u32,
        src_byte_length: u32,
    ) -> Self {
        Self::FillData {
            data: Box::new(data),
            src_byte_offset,
            src_byte_length,
        }
    }
}

pub struct BufferSubData {
    data: Box<dyn AsRef<[u8]>>,
    dst_byte_offset: i32,
    src_byte_offset: u32,
    src_byte_length: u32,
}

impl BufferSubData {
    pub fn new<D: AsRef<[u8]> + 'static>(
        data: D,
        dst_byte_offset: i32,
        src_byte_offset: u32,
        src_byte_length: u32,
    ) -> Self {
        Self {
            data: Box::new(data),
            dst_byte_offset,
            src_byte_offset,
            src_byte_length,
        }
    }
}

pub enum BufferStatus {
    Unchanged {
        id: Uuid,
    },
    UpdateBuffer {
        id: Option<Uuid>,
        data: BufferData,
        usage: BufferUsage,
    },
    UpdateSubBuffer {
        id: Uuid,
        data: BufferSubData,
    },
}

pub struct BufferDescriptor {
    status: RefCell<BufferStatus>,
}

impl BufferDescriptor {
    pub fn new(status: BufferStatus) -> Self {
        Self {
            status: RefCell::new(status),
        }
    }

    pub fn buffer_data(&self, data: BufferData, usage: BufferUsage) {
        let new_status = match *self.status.borrow() {
            BufferStatus::Unchanged { id } => BufferStatus::UpdateBuffer {
                id: Some(id),
                data,
                usage,
            },
            BufferStatus::UpdateBuffer { id, .. } => BufferStatus::UpdateBuffer { id, data, usage },
            BufferStatus::UpdateSubBuffer { id, .. } => BufferStatus::UpdateBuffer {
                id: Some(id),
                data,
                usage,
            },
        };

        *self.status.borrow_mut() = new_status;
    }

    pub fn buffer_sub_data(&self, sub_data: BufferSubData) {
        let new_status = match *self.status.borrow() {
            BufferStatus::Unchanged { id } => BufferStatus::UpdateSubBuffer { id, data: sub_data },
            BufferStatus::UpdateBuffer { id, usage, .. } => BufferStatus::UpdateBuffer {
                id,
                data: BufferData::FillData {
                    data: sub_data.data,
                    src_byte_offset: sub_data.src_byte_offset,
                    src_byte_length: sub_data.src_byte_offset,
                },
                usage,
            },
            BufferStatus::UpdateSubBuffer { id, .. } => {
                BufferStatus::UpdateSubBuffer { id, data: sub_data }
            }
        };

        *self.status.borrow_mut() = new_status;
    }

    pub fn status(&self) -> &RefCell<BufferStatus> {
        &self.status
    }
}

// pub struct BufferItem {
//     buffer: WebGlBuffer,
// }

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
        descriptor: &BufferDescriptor,
        target: &BufferTarget,
    ) -> Result<&WebGlBuffer, JsError> {
        let mut status = descriptor.status().borrow_mut();
        match &*status {
            BufferStatus::Unchanged { id } => {
                let Some(buffer) = self.store.get(id) else {
                    return Err(JsError::new(&format!(
                        "failed to get buffer with id {}",
                        id
                    )));
                };

                Ok(buffer)
            }
            BufferStatus::UpdateBuffer { id, data, usage } => {
                // remove old buffer if specified
                if let Some(buffer) = id.and_then(|id| self.store.remove(&id)) {
                    self.gl.delete_buffer(Some(&buffer));
                };

                // creates buffer and buffers data into it
                let Some(buffer) = self.gl.create_buffer() else {
                    return Err(JsError::new("failed to create WebGL buffer"));
                };

                self.gl.bind_buffer(target.to_gl_enum(), Some(&buffer));
                match data {
                    BufferData::FillEmpty { size } => {
                        self.gl.buffer_data_with_i32(
                            target.to_gl_enum(),
                            *size,
                            usage.to_gl_enum(),
                        );
                    }
                    BufferData::FillData {
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
                let buffer = self.store.entry(id).or_insert(buffer.clone());

                // replace descriptor status
                *status = BufferStatus::Unchanged { id };

                Ok(buffer)
            }
            BufferStatus::UpdateSubBuffer { id, data } => {
                let Some(buffer) = self.store.get(id) else {
                    return Err(JsError::new(&format!(
                        "failed to get buffer with id {}",
                        id
                    )));
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
