use std::{cell::RefCell, rc::Rc};

use hashbrown::{hash_map::Entry, HashMap};
use js_sys::{
    ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array, Int16Array,
    Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};
use uuid::Uuid;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

use super::{conversion::ToGlEnum, error::Error};

/// Available buffer targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferTarget {
    ARRAY_BUFFER,
    ELEMENT_ARRAY_BUFFER,
    COPY_READ_BUFFER,
    COPY_WRITE_BUFFER,
    TRANSFORM_FEEDBACK_BUFFER,
    UNIFORM_BUFFER,
    PIXEL_PACK_BUFFER,
    PIXEL_UNPACK_BUFFER,
}

/// Available buffer usages mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    STATIC_DRAW,
    DYNAMIC_DRAW,
    STREAM_DRAW,
    STATIC_READ,
    DYNAMIC_READ,
    STREAM_READ,
    STATIC_COPY,
    DYNAMIC_COPY,
    STREAM_COPY,
}

/// Buffer data.
#[derive(Debug, Clone)]
pub enum BufferData {
    ArrayBuffer {
        data: ArrayBuffer,
    },
    DataView {
        data: DataView,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int8Array {
        data: Int8Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint8Array {
        data: Uint8Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int16Array {
        data: Int16Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint16Array {
        data: Uint16Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Int32Array {
        data: Int32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Uint32Array {
        data: Uint32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Float32Array {
        data: Float32Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    Float64Array {
        data: Float64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    BigInt64Array {
        data: BigInt64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
    BigUint64Array {
        data: BigUint64Array,
        src_element_offset: Option<usize>,
        src_element_length: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct Buffer {
    id: Rc<Uuid>,
    version: Rc<RefCell<usize>>,
    queue: Rc<RefCell<Vec<BufferData>>>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            id: Rc::new(Uuid::new_v4()),
            version: Rc::new(RefCell::new(usize::MIN)),
            queue: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn id(&self) -> &Uuid {
        self.id.as_ref()
    }

    pub fn version(&self) -> usize {
        *self.version.borrow()
    }

    pub fn update_version(&mut self) {
        let mut version = self.version.borrow_mut();
        *version = version.saturating_add(1);
    }

    pub fn upload(&mut self, data: BufferData) {
        self.queue.borrow_mut().push(data);
    }
}

impl BufferData {
    fn byte_length(&self) -> usize {
        match self {
            BufferData::ArrayBuffer { data, .. } => data.byte_length() as usize,
            BufferData::DataView { data, .. } => data.byte_length() as usize,
            BufferData::Int8Array { data, .. } => data.byte_length() as usize,
            BufferData::Uint8Array { data, .. } => data.byte_length() as usize,
            BufferData::Uint8ClampedArray { data, .. } => data.byte_length() as usize,
            BufferData::Int16Array { data, .. } => data.byte_length() as usize,
            BufferData::Uint16Array { data, .. } => data.byte_length() as usize,
            BufferData::Int32Array { data, .. } => data.byte_length() as usize,
            BufferData::Uint32Array { data, .. } => data.byte_length() as usize,
            BufferData::Float32Array { data, .. } => data.byte_length() as usize,
            BufferData::Float64Array { data, .. } => data.byte_length() as usize,
            BufferData::BigInt64Array { data, .. } => data.byte_length() as usize,
            BufferData::BigUint64Array { data, .. } => data.byte_length() as usize,
        }
    }

    fn upload(
        &self,
        gl: &WebGl2RenderingContext,
        gl_buffer: &WebGlBuffer,
        target: BufferTarget,
        usage: BufferUsage,
        dst_byte_offset: usize,
    ) {
        match self {
            BufferData::DataView { .. }
            | BufferData::Int8Array { .. }
            | BufferData::Uint8Array { .. }
            | BufferData::Uint8ClampedArray { .. }
            | BufferData::Int16Array { .. }
            | BufferData::Uint16Array { .. }
            | BufferData::Int32Array { .. }
            | BufferData::Uint32Array { .. }
            | BufferData::Float32Array { .. }
            | BufferData::Float64Array { .. }
            | BufferData::BigInt64Array { .. }
            | BufferData::BigUint64Array { .. } => {
                let (data, src_element_offset, src_element_length) = match self {
                    BufferData::DataView {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int8Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint8Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint8ClampedArray {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int16Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint16Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Int32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Uint32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Float32Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::Float64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::BigInt64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    BufferData::BigUint64Array {
                        data,
                        src_element_offset,
                        src_element_length,
                    } => (
                        data.as_ref() as &Object,
                        src_element_offset,
                        src_element_length,
                    ),
                    _ => unreachable!(),
                };
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    &data,
                    src_element_offset.unwrap_or(0) as u32,
                    src_element_length.unwrap_or(0) as u32,
                );
            }
            BufferData::ArrayBuffer { data } => {
                gl.buffer_sub_data_with_i32_and_array_buffer(target.gl_enum(), dst_byte_offset as i32, &data);
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct BufferStore {
    gl: WebGl2RenderingContext,
    gl_buffers: Rc<RefCell<HashMap<Uuid, WebGlBuffer>>>,

    used_memory: Rc<RefCell<usize>>,
}

impl BufferStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            gl_buffers: Rc::new(RefCell::new(HashMap::new())),
            used_memory: Rc::new(RefCell::new(usize::MIN)),
        }
    }

    pub fn use_buffer(&self, buffer: &Buffer) -> Result<WebGlBuffer, Error> {
        let mut gl_buffers = self.gl_buffers.borrow_mut();
        let gl_buffer = match gl_buffers.entry(*buffer.id) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let Some(gl_buffer) = self.gl.create_buffer() else {
                    return Err(Error::CreateBufferFailure);
                };
                v.insert(gl_buffer.clone());

                gl_buffer
            }
        };

        // uploads data to webgl if necessary
        let mut queue = buffer.queue.borrow_mut();
        let queue = queue.drain(..);
        for item in queue {
            let memory_size = item.byte_length();
        }

        Ok(gl_buffer)
    }
}
