use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::debug;
use uuid::Uuid;
use web_sys::{
    js_sys::{
        ArrayBuffer, BigInt64Array, BigUint64Array, DataView, Float32Array, Float64Array,
        Int16Array, Int32Array, Int8Array, Object, Uint16Array, Uint32Array, Uint8Array,
        Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::{
    lru::{Lru, LruNode},
    renderer::webgl::params::GetWebGlParameters,
    utils::format_bytes_length,
};

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

/// Available component size of a value get from buffer.
/// According to WebGL definition, it should only be `1`, `2`, `3` or `4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
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
    FLOAT,
    BYTE,
    SHORT,
    INT,
    UNSIGNED_BYTE,
    UNSIGNED_SHORT,
    UNSIGNED_INT,
    HALF_FLOAT,
    INT_2_10_10_10_REV,
    UNSIGNED_INT_2_10_10_10_REV,
}

impl BufferDataType {
    /// Gets bytes length of a data type.
    pub fn bytes_length(&self) -> usize {
        match self {
            BufferDataType::FLOAT => 4,
            BufferDataType::BYTE => 1,
            BufferDataType::SHORT => 2,
            BufferDataType::INT => 4,
            BufferDataType::UNSIGNED_BYTE => 1,
            BufferDataType::UNSIGNED_SHORT => 2,
            BufferDataType::UNSIGNED_INT => 4,
            BufferDataType::HALF_FLOAT => 2,
            BufferDataType::INT_2_10_10_10_REV => 4,
            BufferDataType::UNSIGNED_INT_2_10_10_10_REV => 4,
        }
    }
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

/// Available buffer sources.
///
/// # Note
///
/// Since the linear memory of WASM runtime is impossible to shrink for now,
/// high memory usage could happen if developer create a large WASM native buffer, for example, `Vec<u8>`.
/// It is always a good idea to avoid creating native buffer, use `TypedArrayBuffer` from JavaScript instead.
pub enum BufferSource {
    Preallocate {
        bytes_length: usize,
    },
    Function {
        callback: Box<dyn Fn() -> BufferSource>,
        data_length: usize,
        src_offset: usize,
        src_length: usize,
    },
    Binary {
        data: Box<dyn AsRef<[u8]>>,
        src_offset: usize,
        src_length: usize,
    },
    ArrayBuffer {
        data: ArrayBuffer,
    },
    DataView {
        data: DataView,
        src_offset: usize,
        src_length: usize,
    },
    Int8Array {
        data: Int8Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint8Array {
        data: Uint8Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint8ClampedArray {
        data: Uint8ClampedArray,
        src_offset: usize,
        src_length: usize,
    },
    Int16Array {
        data: Int16Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint16Array {
        data: Uint16Array,
        src_offset: usize,
        src_length: usize,
    },
    Int32Array {
        data: Int32Array,
        src_offset: usize,
        src_length: usize,
    },
    Uint32Array {
        data: Uint32Array,
        src_offset: usize,
        src_length: usize,
    },
    Float32Array {
        data: Float32Array,
        src_offset: usize,
        src_length: usize,
    },
    Float64Array {
        data: Float64Array,
        src_offset: usize,
        src_length: usize,
    },
    BigInt64Array {
        data: BigInt64Array,
        src_offset: usize,
        src_length: usize,
    },
    BigUint64Array {
        data: BigUint64Array,
        src_offset: usize,
        src_length: usize,
    },
}

impl BufferSource {
    fn collect_typed_array_buffer(&self) -> (&Object, usize, usize) {
        match self {
            BufferSource::Preallocate { .. }
            | BufferSource::Function { .. }
            | BufferSource::Binary { .. }
            | BufferSource::ArrayBuffer { .. } => {
                unreachable!()
            }
            BufferSource::DataView {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint8Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
            } => (data, *src_offset, *src_length),
        }
    }

    // /// Buffers data to WebGL runtime.
    // fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
    //     match self {
    //         BufferSource::Preallocate { bytes_length } => {
    //             gl.buffer_data_with_i32(target.gl_enum(), *bytes_length as i32, usage.gl_enum())
    //         }
    //         BufferSource::Function { callback, .. } => {
    //             let source = callback();
    //             if let BufferSource::Function { .. } = source {
    //                 panic!("recursive BufferSource::Function is not allowed");
    //             }
    //             if self.bytes_length() != source.bytes_length() {
    //                 panic!(
    //                     "source returned from BufferSource::Function should have same bytes length"
    //                 );
    //             }
    //             source.buffer_data(gl, target, usage);
    //         }
    //         BufferSource::Binary {
    //             data,
    //             src_offset,
    //             src_length,
    //         } => gl.buffer_data_with_u8_array_and_src_offset_and_length(
    //             target.gl_enum(),
    //             data.as_ref().as_ref(),
    //             usage.gl_enum(),
    //             *src_offset as u32,
    //             *src_length as u32,
    //         ),
    //         BufferSource::ArrayBuffer { data } => {
    //             gl.buffer_data_with_opt_array_buffer(target.gl_enum(), Some(data), usage.gl_enum())
    //         }
    //         _ => {
    //             let (data, src_offset, src_length) = self.collect_typed_array_buffer();
    //             gl.buffer_data_with_array_buffer_view_and_src_offset_and_length(
    //                 target.gl_enum(),
    //                 data,
    //                 usage.gl_enum(),
    //                 src_offset as u32,
    //                 src_length as u32,
    //             );
    //         }
    //     }
    // }

    /// Buffers sub data to WebGL runtime.
    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        match self {
            BufferSource::Preallocate { bytes_length } => {
                let src_data = Uint8Array::new_with_length(*bytes_length as u32);
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    &src_data,
                    0,
                    *bytes_length as u32,
                )
            }
            BufferSource::Function { callback, .. } => {
                let source = callback();
                if let BufferSource::Function { .. } = source {
                    panic!("recursively BufferSource::Function is not allowed");
                }
                if self.bytes_length() != source.bytes_length() {
                    panic!(
                        "source returned from BufferSource::Function should have same bytes length"
                    );
                }
                source.buffer_sub_data(gl, target, dst_byte_offset);
            }
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
            } => gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                target.gl_enum(),
                dst_byte_offset as i32,
                data.as_ref().as_ref(),
                *src_offset as u32,
                *src_length as u32,
            ),
            BufferSource::ArrayBuffer { data } => {
                log::info!("{} {}", data.byte_length(), dst_byte_offset);
                gl.buffer_sub_data_with_i32_and_array_buffer(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    data,
                )
            }
            _ => {
                let (data, src_offset, src_length) = self.collect_typed_array_buffer();
                gl.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length(
                    target.gl_enum(),
                    dst_byte_offset as i32,
                    data,
                    src_offset as u32,
                    src_length as u32,
                )
            }
        }
    }

    /// Returns the length of data in bytes.
    pub fn bytes_length(&self) -> usize {
        let (raw_length, src_offset, src_length) = match self {
            BufferSource::Preallocate { bytes_length } => (*bytes_length, 0, 0),
            BufferSource::Function {
                data_length,
                src_offset,
                src_length,
                ..
            } => (*data_length, *src_offset, *src_length),
            BufferSource::Binary {
                data,
                src_offset,
                src_length,
                ..
            } => (data.as_ref().as_ref().len(), *src_offset, *src_length),
            BufferSource::ArrayBuffer { data } => (data.byte_length() as usize, 0, 0),
            BufferSource::DataView {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int8Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8Array {
                data,
                src_offset,
                src_length,
                ..
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int16Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint16Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Int32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Uint32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float32Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::Float64Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigInt64Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
                *src_length,
            ),
            BufferSource::BigUint64Array {
                data,
                src_offset,
                src_length,
            } => (
                data.byte_length() as usize,
                data.byte_offset() as usize + *src_offset,
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
    pub fn preallocate(bytes_length: usize) -> Self {
        Self::Preallocate { bytes_length }
    }

    /// Constructs a new buffer source from a callback function.
    /// Preflies information is required and it should be same as the callback value.
    pub fn from_function<F>(
        callback: F,
        data_length: usize,
        src_offset: usize,
        src_length: usize,
    ) -> Self
    where
        F: Fn() -> BufferSource + 'static,
    {
        Self::Function {
            callback: Box::new(callback),
            data_length,
            src_offset,
            src_length,
        }
    }

    /// Constructs a new buffer source from WASM native buffer.
    pub fn from_binary<D>(data: D, src_offset: usize, src_length: usize) -> Self
    where
        D: AsRef<[u8]> + 'static,
    {
        Self::Binary {
            data: Box::new(data),
            src_offset,
            src_length,
        }
    }

    /// Constructs a new buffer source from [`ArrayBuffer`].
    pub fn from_array_buffer(data: ArrayBuffer) -> Self {
        Self::ArrayBuffer { data }
    }
}

macro_rules! impl_typed_array {
    ($(($from: ident, $source: tt, $kind: ident, $name: expr)),+) => {
        impl BufferSource {
            $(
                #[doc = "Constructs a new buffer source from "]
                #[doc = $name]
                #[doc = "."]
                pub fn $from(
                    data: $source,
                    src_offset: usize,
                    src_length: usize,
                ) -> Self {
                    Self::$kind {
                        data,
                        src_offset,
                        src_length
                    }
                }
            )+
        }
    };
}

impl_typed_array! {
    (from_int8_array, Int8Array, Int8Array, "[`Int8Array`]"),
    (from_uint8_array, Uint8Array, Uint8Array, "[`Uint8Array`]"),
    (from_uint8_clamped_array, Uint8ClampedArray, Uint8ClampedArray, "[`Uint8ClampedArray`]"),
    (from_int16_array, Int16Array, Int16Array, "[`Int16Array`]"),
    (from_uint16_array, Uint16Array, Uint16Array, "[`Uint16Array`]"),
    (from_int32_array, Int32Array, Int32Array, "[`Int32Array`]"),
    (from_uint32_array, Uint32Array, Uint32Array, "[`Uint32Array`]"),
    (from_float32_array, Float32Array, Float32Array, "[`Float32Array`]"),
    (from_float64_array, Float64Array, Float64Array, "[`Float64Array`]"),
    (from_big_int64_array, BigInt64Array, BigInt64Array, "[`BigInt64Array`]"),
    (from_big_uint64_array, BigUint64Array, BigUint64Array, "[`BigUint64Array`]")
}

impl Debug for BufferSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preallocate { bytes_length } => f
                .debug_struct("Preallocate")
                .field("bytes_length", bytes_length)
                .finish(),
            Self::Function {
                data_length,
                src_offset,
                src_length,
                ..
            } => f
                .debug_struct("Function")
                .field("data_length", data_length)
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Binary {
                src_offset,
                src_length,
                ..
            } => f
                .debug_struct("Binary")
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::ArrayBuffer { data } => f
                .debug_struct("ArrayBuffer")
                .field("data_length", &data.byte_length())
                .finish(),
            Self::DataView {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("DataView")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Int8Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Int8Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Uint8Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Uint8Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Uint8ClampedArray {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Uint8ClampedArray")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Int16Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Int16Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Uint16Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Uint16Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Int32Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Int32Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Uint32Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Uint32Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Float32Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Float32Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::Float64Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("Float64Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::BigInt64Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("BigInt64Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
            Self::BigUint64Array {
                data,
                src_offset,
                src_length,
            } => f
                .debug_struct("BigUint64Array")
                .field("data_length", &data.byte_length())
                .field("src_offset", src_offset)
                .field("src_length", src_length)
                .finish(),
        }
    }
}

#[derive(Debug)]
struct Registered {
    gl: WebGl2RenderingContext,
    lru_node: *mut LruNode<Uuid>,
    available_memory: *mut usize,
    used_memory: *mut usize,
    items: *mut HashMap<Uuid, BufferItem>,
    bindings: *mut HashMap<BufferTarget, Uuid>,
    binding_ubos: *mut HashMap<u32, Uuid>,
    lru: *mut Lru<Uuid>,
}

impl Registered {
    fn update_lru(&mut self) {
        unsafe {
            (*self.lru).cache(self.lru_node);
        }
    }

    fn update_used_memory(&mut self, new_bytes_length: usize, old_bytes_length: usize) {
        unsafe {
            *self.used_memory = *self.used_memory - old_bytes_length + new_bytes_length;
        }
    }

    fn add_binding(&mut self, target: BufferTarget, id: Uuid) {
        unsafe {
            (*self.bindings).insert(target, id);
        }
    }

    fn add_binding_ubo(&mut self, index: u32, id: Uuid) {
        unsafe {
            (*self.binding_ubos).insert(index, id);
        }
    }

    fn remove_binding(&mut self, target: BufferTarget) {
        unsafe {
            (*self.bindings).remove(&target);
        }
    }

    fn remove_binding_ubo(&mut self, index: u32) {
        unsafe {
            (*self.binding_ubos).remove(&index);
        }
    }

    fn is_occupied(&self, target: BufferTarget, id: &Uuid) -> bool {
        unsafe {
            (*self.bindings)
                .get(&target)
                .map(|v| v != id)
                .unwrap_or(false)
        }
    }

    fn is_occupied_ubo(&self, index: u32, id: &Uuid) -> bool {
        unsafe {
            (*self.binding_ubos)
                .get(&index)
                .map(|v| v != id)
                .unwrap_or(false)
        }
    }

    /// Frees memory if used memory exceeds the maximum available memory.
    fn free(&mut self) {
        // removes buffer from the least recently used until memory usage lower than limitation
        unsafe {
            if *self.used_memory <= *self.available_memory {
                return;
            }

            let mut next_node = (*self.lru).least_recently();
            while *self.used_memory > *self.available_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };
                let id = (*current_node).data();

                let Entry::Occupied(occupied) = (*self.items).entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let item = occupied.get();
                let (Some(queue), Some(memory_policy), Some(registered), Some(runtime)) = (
                    item.queue.upgrade(),
                    item.memory_policy.upgrade(),
                    item.registered.upgrade(),
                    item.runtime.upgrade(),
                ) else {
                    // deletes if already dropped
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };

                let (mut queue, memory_policy, mut registered, mut runtime) = (
                    queue.borrow_mut(),
                    memory_policy.borrow_mut(),
                    registered.borrow_mut(),
                    runtime.borrow_mut(),
                );
                let (Some(registered), Some(runtime)) = (registered.as_mut(), runtime.as_mut())
                else {
                    next_node = (*current_node).more_recently();
                    continue;
                };

                // skips if using
                if runtime.bindings.len() + runtime.binding_ubos.len() != 0 {
                    next_node = (*current_node).more_recently();
                    continue;
                }

                // free
                match &*memory_policy {
                    MemoryPolicy::Unfree => {
                        next_node = (*current_node).more_recently();
                        continue;
                    }
                    MemoryPolicy::ReadBack => {
                        if runtime.buffer.is_some() {
                            queue.items.insert(
                                0,
                                QueueItem::new(
                                    BufferSource::from_array_buffer(runtime.read_back()),
                                    0,
                                ),
                            );
                            queue.max_bytes_length =
                                queue.max_bytes_length.max(runtime.bytes_length);
                        }

                        debug!(
                            target: "BufferStore",
                            "free buffer (readback) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(runtime.bytes_length),
                            format_bytes_length(*self.used_memory)
                        );
                    }
                    MemoryPolicy::Restorable(restorer) => {
                        if let Some(buffer) = runtime.buffer.as_ref() {
                            self.gl.delete_buffer(Some(&buffer));
                        }

                        let restorer = Rc::clone(&restorer);
                        let source = BufferSource::Function {
                            callback: Box::new(move || restorer.borrow_mut().restore()),
                            data_length: runtime.bytes_length,
                            src_offset: 0,
                            src_length: runtime.bytes_length,
                        };

                        queue.items.insert(0, QueueItem::new(source, 0));
                        queue.max_bytes_length = queue.max_bytes_length.max(runtime.bytes_length);
                        debug!(
                            target: "BufferStore",
                            "free buffer (restorable) {}. freed memory {}, used {}",
                            id,
                            format_bytes_length(runtime.bytes_length),
                            format_bytes_length(*self.used_memory)
                        );
                    }
                }

                (*registered.used_memory) -= runtime.bytes_length;
                (*registered.lru).remove(registered.lru_node);
                runtime.bytes_length = 0;

                occupied.remove();

                next_node = (*current_node).more_recently();
            }
        }
    }
}

#[derive(Debug)]
struct Runtime {
    gl: WebGl2RenderingContext,
    buffer: Option<WebGlBuffer>,
    bytes_length: usize,
    bindings: HashSet<BufferTarget>,
    binding_ubos: HashSet<u32>,
}

impl Runtime {
    fn read_back(&self) -> ArrayBuffer {
        let Some(buffer) = self.buffer.as_ref() else {
            return ArrayBuffer::new(0);
        };
        if self.bytes_length == 0 {
            return ArrayBuffer::new(0);
        }

        let gl = &self.gl;
        let data = ArrayBuffer::new(self.bytes_length as u32);
        let binding = gl.array_buffer_binding();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            &data,
        );
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        data
    }

    fn get_or_create_buffer(&mut self) -> Result<WebGlBuffer, Error> {
        match self.buffer.as_mut() {
            Some(buffer) => Ok(buffer.clone()),
            None => {
                let buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                Ok(self.buffer.insert(buffer).clone())
            }
        }
    }

    fn bind_and_upload(
        &mut self,
        target: BufferTarget,
        usage: BufferUsage,
        queue: RefMut<'_, Queue>,
    ) -> Result<(usize, usize), Error> {
        let buffer = self.get_or_create_buffer()?;
        self.gl.bind_buffer(target.gl_enum(), Some(&buffer));
        let memory = self.upload(target, usage, queue);
        self.bindings.insert(target);
        Ok(memory)
    }

    fn bind_and_upload_ubo(
        &mut self,
        usage: BufferUsage,
        index: u32,
        queue: RefMut<'_, Queue>,
    ) -> Result<(usize, usize), Error> {
        let buffer = self.get_or_create_buffer()?;
        let binding = self.gl.uniform_buffer_binding();
        self.gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
        let memory = self.upload(BufferTarget::UNIFORM_BUFFER, usage, queue);
        self.gl.bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            index,
            self.buffer.as_ref(),
        );
        self.gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, binding.as_ref());
        self.binding_ubos.insert(index);
        Ok(memory)
    }

    fn bind_and_upload_ubo_range(
        &mut self,
        usage: BufferUsage,
        index: u32,
        offset: i32,
        size: i32,
        queue: RefMut<'_, Queue>,
    ) -> Result<(usize, usize), Error> {
        let buffer = self.get_or_create_buffer()?;
        let binding = self.gl.uniform_buffer_binding();
        self.gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
        let memory = self.upload(BufferTarget::UNIFORM_BUFFER, usage, queue);
        self.gl.bind_buffer_range_with_i32_and_i32(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            index,
            self.buffer.as_ref(),
            offset,
            size,
        );
        self.gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, binding.as_ref());
        self.binding_ubos.insert(index);
        Ok(memory)
    }

    fn unbind(&mut self, target: BufferTarget) {
        if self.bindings.remove(&target) {
            self.gl.bind_buffer(target.gl_enum(), None);
        }
    }

    fn unbind_ubo(&mut self, index: u32) {
        if self.binding_ubos.remove(&index) {
            self.gl
                .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);
        }
    }

    fn upload(
        &mut self,
        target: BufferTarget,
        usage: BufferUsage,
        mut queue: RefMut<'_, Queue>,
    ) -> (usize, usize) {
        if queue.items.len() > 0 {
            let new_bytes_length = queue.max_bytes_length;
            let old_bytes_length = self.bytes_length;

            if new_bytes_length >= old_bytes_length {
                self.gl.buffer_data_with_i32(
                    target.gl_enum(),
                    new_bytes_length as i32,
                    usage.gl_enum(),
                );
            }

            for item in queue.items.drain(..) {
                item.source
                    .buffer_sub_data(&self.gl, target, item.bytes_offset);
            }

            self.bytes_length = new_bytes_length;

            debug!(
                target: "BufferStore",
                "buffer new data, old length {}, new length {}",
                old_bytes_length,
                new_bytes_length
            );

            (new_bytes_length, old_bytes_length)
        } else {
            (0, 0)
        }
    }
}

#[derive(Debug)]
struct QueueItem {
    source: BufferSource,
    bytes_offset: usize,
}

impl QueueItem {
    fn new(source: BufferSource, bytes_offset: usize) -> Self {
        Self {
            source,
            bytes_offset,
        }
    }
}

#[derive(Debug)]
struct Queue {
    max_bytes_length: usize,
    items: Vec<QueueItem>,
}

impl Queue {
    fn new() -> Self {
        Self {
            max_bytes_length: 0,
            items: Vec::new(),
        }
    }

    /// Overrides existing data and then buffers new data.
    fn buffer_data(&mut self, source: BufferSource) {
        self.max_bytes_length = source.bytes_length();
        self.items.clear();
        self.items.push(QueueItem::new(source, 0));
    }

    /// Buffers sub data.
    fn buffer_sub_data(
        &mut self,
        runtime: Option<Ref<'_, Runtime>>,
        source: BufferSource,
        dst_byte_offset: usize,
    ) {
        let bytes_length = dst_byte_offset + source.bytes_length();
        if dst_byte_offset == 0 {
            if bytes_length >= self.max_bytes_length {
                // overrides sources in queue if new source covers all
                self.max_bytes_length = bytes_length;
                self.items.clear();
                self.items.push(QueueItem::new(source, 0));
            } else {
                self.items.push(QueueItem::new(source, dst_byte_offset));
            }
        } else {
            if bytes_length <= self.max_bytes_length {
                self.items.push(QueueItem::new(source, dst_byte_offset));
            } else {
                if let Some(runtime) = runtime.as_ref() {
                    // heavy job!
                    let data = runtime.read_back();

                    self.items.insert(
                        0,
                        QueueItem::new(BufferSource::preallocate(bytes_length), 0),
                    );
                    self.items
                        .insert(1, QueueItem::new(BufferSource::from_array_buffer(data), 0));
                    self.items.push(QueueItem::new(source, dst_byte_offset));
                } else {
                    self.items.insert(
                        0,
                        QueueItem::new(BufferSource::preallocate(bytes_length), 0),
                    );
                    self.items.push(QueueItem::new(source, dst_byte_offset));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    id: Uuid,
    name: Option<Cow<'static, str>>,
    usage: BufferUsage,
    memory_policy: Rc<RefCell<MemoryPolicy>>,
    queue: Rc<RefCell<Queue>>,
    registered: Rc<RefCell<Option<Registered>>>,
    runtime: Rc<RefCell<Option<Runtime>>>,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let _ = self.clear(false);

        if let Some(mut registered) = self.registered_mut() {
            unsafe {
                (*registered.items).remove(&self.id);
                (*registered.lru).remove(registered.lru_node);
            }
        }
    }
}

impl Buffer {
    /// Constructs a new buffer descriptor with specified name, [`BufferSource`], [`BufferUsage`] and [`MemoryPolicy`].
    pub fn new(
        name: Option<Cow<'static, str>>,
        usage: BufferUsage,
        memory_policy: MemoryPolicy,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            usage,
            memory_policy: Rc::new(RefCell::new(memory_policy)),
            queue: Rc::new(RefCell::new(Queue::new())),
            registered: Rc::new(RefCell::new(None)),
            runtime: Rc::new(RefCell::new(None)),
        }
    }

    /// Returns id of this buffer.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns buffer descriptor name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets buffer descriptor name.
    pub fn set_name(&mut self, name: Option<Cow<'static, str>>) {
        self.name = name;
    }

    /// Returns [`BufferUsage`].
    pub fn usage(&self) -> BufferUsage {
        self.usage
    }

    // /// Returns current [`MemoryPolicy`].
    // pub fn memory_policy(&self) -> &MemoryPolicy {
    //     &self.memory_policy
    // }

    /// Sets the [`MemoryPolicy`] of this buffer descriptor.
    pub fn set_memory_policy(&mut self, memory_policy: MemoryPolicy) {
        *self.memory_policy.borrow_mut() = memory_policy;
    }

    /// Initializes this buffer by a [`WebGl2RenderingContext`].
    pub fn init(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        let mut runtime = self.runtime.borrow_mut();
        match runtime.as_mut() {
            Some(runtime) => {
                if &runtime.gl == gl {
                    Ok(())
                } else {
                    Err(Error::BufferAlreadyInitialized)
                }
            }
            None => {
                *runtime = Some(Runtime {
                    gl: gl.clone(),
                    buffer: None,
                    bytes_length: 0,
                    bindings: HashSet::new(),
                    binding_ubos: HashSet::new(),
                });
                Ok(())
            }
        }
    }

    fn queue_mut(&self) -> RefMut<'_, Queue> {
        self.queue.borrow_mut()
    }

    fn runtime(&self) -> Result<Ref<'_, Runtime>, Error> {
        let runtime = self.runtime.borrow();
        let runtime = Ref::filter_map(runtime, |runtime| runtime.as_ref())
            .map_err(|_| Error::BufferAlreadyInitialized);
        runtime
    }

    fn runtime_mut(&self) -> Result<RefMut<'_, Runtime>, Error> {
        let runtime = self.runtime.borrow_mut();
        let runtime = RefMut::filter_map(runtime, |runtime| runtime.as_mut())
            .map_err(|_| Error::BufferAlreadyInitialized);
        runtime
    }

    fn registered(&self) -> Option<Ref<'_, Registered>> {
        let registered = self.registered.borrow();
        let registered = Ref::filter_map(registered, |registered| registered.as_ref()).ok();
        registered
    }

    fn registered_mut(&self) -> Option<RefMut<'_, Registered>> {
        let registered = self.registered.borrow_mut();
        let registered = RefMut::filter_map(registered, |registered| registered.as_mut()).ok();
        registered
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind(&self, target: BufferTarget) -> Result<(), Error> {
        if let Some(registered) = self.registered() {
            if registered.is_occupied(target, &self.id) {
                return Err(Error::BufferTargetOccupied(target));
            }
        }

        let (new_bytes_length, old_bytes_length) =
            self.runtime_mut()?
                .bind_and_upload(target, self.usage, self.queue_mut())?;

        if let Some(mut registered) = self.registered_mut() {
            registered.update_lru();
            registered.update_used_memory(new_bytes_length, old_bytes_length);
            registered.add_binding(target, self.id);
            registered.free();
        }

        Ok(())
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind_ubo(&self, index: u32) -> Result<(), Error> {
        if let Some(registered) = self.registered() {
            if registered.is_occupied_ubo(index, &self.id) {
                return Err(Error::UniformBufferObjectIndexOccupied(index));
            }
        }

        let (new_bytes_length, old_bytes_length) =
            self.runtime_mut()?
                .bind_and_upload_ubo(self.usage, index, self.queue_mut())?;

        if let Some(mut registered) = self.registered_mut() {
            registered.update_lru();
            registered.update_used_memory(new_bytes_length, old_bytes_length);
            registered.add_binding_ubo(index, self.id);
            registered.free();
        }

        Ok(())
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind_ubo_range(&self, index: u32, offset: i32, size: i32) -> Result<(), Error> {
        if let Some(registered) = self.registered() {
            if registered.is_occupied_ubo(index, &self.id) {
                return Err(Error::UniformBufferObjectIndexOccupied(index));
            }
        }

        let (new_bytes_length, old_bytes_length) = self.runtime_mut()?.bind_and_upload_ubo_range(
            self.usage,
            index,
            offset,
            size,
            self.queue_mut(),
        )?;

        if let Some(mut registered) = self.registered_mut() {
            registered.update_lru();
            registered.update_used_memory(new_bytes_length, old_bytes_length);
            registered.add_binding_ubo(index, self.id);
            registered.free();
        }

        Ok(())
    }

    /// Unbinds buffer at specified [`BufferTarget`].
    pub fn unbind(&self, target: BufferTarget) -> Result<(), Error> {
        self.runtime_mut()?.unbind(target);

        if let Some(mut registered) = self.registered_mut() {
            registered.remove_binding(target);
        }

        Ok(())
    }

    /// Unbinds buffer from specified uniform buffer object index.
    pub fn unbind_ubo(&self, index: u32) -> Result<(), Error> {
        self.runtime_mut()?.unbind_ubo(index);

        if let Some(mut registered) = self.registered_mut() {
            registered.remove_binding_ubo(index);
        }

        Ok(())
    }

    /// Unbinds buffer from all bindings, including uniform buffer objects.
    pub fn unbind_all(&self) -> Result<(), Error> {
        let mut runtime = self.runtime_mut()?;
        let mut registered = self.registered_mut();

        let gl = runtime.gl.clone();
        for index in runtime.binding_ubos.drain() {
            gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);

            if let Some(registered) = registered.as_mut() {
                registered.remove_binding_ubo(index);
            }
        }
        for target in runtime.bindings.drain() {
            gl.bind_buffer(target.gl_enum(), None);

            if let Some(registered) = registered.as_mut() {
                registered.remove_binding(target);
            }
        }

        Ok(())
    }

    /// Uploads data to WebGL runtime.
    pub fn upload(&mut self) -> Result<(), Error> {
        let mut runtime = self.runtime_mut()?;
        let mut registered = self.registered_mut();

        let binding = runtime.gl.array_buffer_binding();
        let (new_bytes_length, old_bytes_length) =
            runtime.bind_and_upload(BufferTarget::ARRAY_BUFFER, self.usage, self.queue_mut())?;
        runtime.unbind(BufferTarget::ARRAY_BUFFER);
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        if let Some(registered) = registered.as_mut() {
            registered.update_used_memory(new_bytes_length, old_bytes_length);
            registered.free();
        }

        Ok(())
    }

    /// Clears and unbinds buffer from WebGL runtime.
    /// Data will be read back from WebGL runtime and
    /// insert to the first place of the queue if `read_back` is `true`.
    pub fn clear(&mut self, read_back: bool) -> Result<(), Error> {
        let mut runtime = self.runtime_mut()?;
        let mut queue = self.queue_mut();
        let mut registered = self.registered_mut();

        queue.items.clear();
        queue.max_bytes_length = 0;

        if read_back {
            let data = runtime.read_back();
            if data.byte_length() != 0 {
                queue
                    .items
                    .push(QueueItem::new(BufferSource::from_array_buffer(data), 0));
            }
        }

        let new_bytes_length = 0;
        let old_bytes_length = runtime.bytes_length;
        let gl = runtime.gl.clone();
        if let Some(buffer) = runtime.buffer.take() {
            for index in runtime.binding_ubos.drain() {
                gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);

                if let Some(registered) = registered.as_mut() {
                    registered.remove_binding_ubo(index);
                }
            }
            for target in runtime.bindings.drain() {
                gl.bind_buffer(target.gl_enum(), None);

                if let Some(registered) = registered.as_mut() {
                    registered.remove_binding(target);
                }
            }
            gl.delete_buffer(Some(&buffer))
        }
        runtime.bytes_length = new_bytes_length;

        if let Some(registered) = registered.as_mut() {
            registered.update_used_memory(new_bytes_length, old_bytes_length);
        }

        Ok(())
    }

    /// Reads buffer data back from WebGL runtime and stores it to an [`ArrayBuffer`].
    pub fn read_back(&self) -> Result<ArrayBuffer, Error> {
        Ok(self.runtime_mut()?.read_back())
    }

    /// Overrides existing data and then buffers new data.
    pub fn buffer_data(&mut self, source: BufferSource) {
        self.queue_mut().buffer_data(source)
    }

    /// Buffers sub data.
    pub fn buffer_sub_data(&mut self, source: BufferSource, dst_byte_offset: usize) {
        self.queue_mut()
            .buffer_sub_data(self.runtime().ok(), source, dst_byte_offset)
    }
}

pub trait Restorer: Debug {
    fn restore(&self) -> BufferSource;
}

/// Memory freeing policies.
#[derive(Debug)]
pub enum MemoryPolicy {
    Unfree,
    ReadBack,
    Restorable(Rc<RefCell<dyn Restorer>>),
}

impl MemoryPolicy {
    /// Constructs a unfree-able memory policy.
    pub fn unfree() -> Self {
        Self::Unfree
    }

    /// Constructs a read back memory policy.
    pub fn read_back() -> Self {
        Self::ReadBack
    }

    /// Constructs a restorable memory policy.
    pub fn restorable<R>(restorer: R) -> Self
    where
        R: Restorer + 'static,
    {
        Self::Restorable(Rc::new(RefCell::new(restorer)))
    }
}

pub struct Builder {
    name: Option<Cow<'static, str>>,
    usage: BufferUsage,
    memory_policy: MemoryPolicy,
    queue: Queue,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            name: None,
            usage: BufferUsage::STATIC_DRAW,
            memory_policy: MemoryPolicy::ReadBack,
            queue: Queue::new(),
        }
    }
}

impl Builder {
    /// Constructs a new buffer builder.
    pub fn new(usage: BufferUsage) -> Self {
        Self {
            name: None,
            usage,
            memory_policy: MemoryPolicy::ReadBack,
            queue: Queue::new(),
        }
    }

    /// Sets name.
    pub fn set_name(mut self, name: String) -> Self {
        self.name = Some(Cow::Owned(name));
        self
    }

    /// Sets name.
    pub fn set_name_str(mut self, name: &'static str) -> Self {
        self.name = Some(Cow::Borrowed(name));
        self
    }

    /// Sets [`BufferUsage`].
    pub fn set_usage(mut self, usage: BufferUsage) -> Self {
        self.usage = usage;
        self
    }

    /// Sets [`MemoryPolicy`].
    pub fn set_memory_policy(mut self, memory_policy: MemoryPolicy) -> Self {
        self.memory_policy = memory_policy;
        self
    }

    /// Overrides existing data and then buffers new data.
    pub fn buffer_data(mut self, source: BufferSource) -> Self {
        self.queue.buffer_data(source);
        self
    }

    /// Buffers sub data.
    pub fn buffer_sub_data(mut self, source: BufferSource, dst_byte_offset: usize) -> Self {
        self.queue.buffer_sub_data(None, source, dst_byte_offset);
        self
    }

    pub fn build(self) -> Buffer {
        Buffer {
            id: Uuid::new_v4(),
            name: self.name,
            usage: self.usage,
            memory_policy: Rc::new(RefCell::new(self.memory_policy)),
            queue: Rc::new(RefCell::new(self.queue)),
            registered: Rc::new(RefCell::new(None)),
            runtime: Rc::new(RefCell::new(None)),
        }
    }
}

struct BufferItem {
    queue: Weak<RefCell<Queue>>,
    memory_policy: Weak<RefCell<MemoryPolicy>>,
    registered: Weak<RefCell<Option<Registered>>>,
    runtime: Weak<RefCell<Option<Runtime>>>,
}

pub struct BufferStore {
    gl: WebGl2RenderingContext,
    available_memory: *mut usize,
    used_memory: *mut usize,
    items: *mut HashMap<Uuid, BufferItem>,
    bindings: *mut HashMap<BufferTarget, Uuid>,
    binding_ubos: *mut HashMap<u32, Uuid>,
    lru: *mut Lru<Uuid>,
}

impl Drop for BufferStore {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.available_memory));
            drop(Box::from_raw(self.used_memory));
            drop(Box::from_raw(self.binding_ubos));
            drop(Box::from_raw(self.bindings));
            drop(Box::from_raw(self.lru));

            for item in Box::from_raw(self.items).values_mut() {
                let Some(registered) = item.registered.upgrade() else {
                    continue;
                };
                *registered.borrow_mut() = None;
            }
        }
    }
}

impl BufferStore {
    /// Constructs a new buffer store with unlimited memory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self::with_available_memory(gl, i32::MAX as usize)
    }

    /// Constructs a new buffer store with a maximum available memory.
    /// Maximum available memory is clamped to [`i32::MAX`] if larger than [`i32::MAX`];
    pub fn with_available_memory(gl: WebGl2RenderingContext, available_memory: usize) -> Self {
        Self {
            gl,
            available_memory: Box::leak(Box::new(available_memory.min(i32::MAX as usize))),
            used_memory: Box::leak(Box::new(0)),
            items: Box::leak(Box::new(HashMap::new())),
            bindings: Box::leak(Box::new(HashMap::new())),
            binding_ubos: Box::leak(Box::new(HashMap::new())),
            lru: Box::leak(Box::new(Lru::new())),
        }
    }

    /// Returns the maximum available memory in bytes.
    /// Returns [`i32::MAX`] if not specified.
    pub fn available_memory(&self) -> usize {
        unsafe { *self.available_memory }
    }

    /// Returns current used memory in bytes.
    pub fn used_memory(&self) -> usize {
        unsafe { *self.used_memory }
    }

    /// Inits and registers a buffer to buffer store.
    pub fn register(&mut self, buffer: &Buffer) -> Result<(), Error> {
        unsafe {
            if (*self.items).contains_key(buffer.id()) {
                return Ok(());
            }

            buffer.init(&self.gl)?;

            let runtime = buffer.runtime().unwrap();
            (*self.used_memory) += runtime.bytes_length;
            for binding in &runtime.bindings {
                if (*self.bindings).contains_key(binding) {
                    return Err(Error::BufferTargetOccupied(*binding));
                }
                (*self.bindings).insert(*binding, buffer.id);
            }
            for binding in &runtime.binding_ubos {
                if (*self.binding_ubos).contains_key(binding) {
                    return Err(Error::UniformBufferObjectIndexOccupied(*binding));
                }
                (*self.binding_ubos).insert(*binding, buffer.id);
            }

            *buffer.registered.borrow_mut() = Some(Registered {
                gl: self.gl.clone(),
                lru_node: LruNode::new(buffer.id),
                available_memory: self.available_memory,
                used_memory: self.used_memory,
                items: self.items,
                bindings: self.bindings,
                binding_ubos: self.binding_ubos,
                lru: self.lru,
            });

            (*self.items).insert(
                buffer.id,
                BufferItem {
                    queue: Rc::downgrade(&buffer.queue),
                    memory_policy: Rc::downgrade(&buffer.memory_policy),
                    registered: Rc::downgrade(&buffer.registered),
                    runtime: Rc::downgrade(&buffer.runtime),
                },
            );

            Ok(())
        }
    }

    /// Unregisters a buffer from buffer store.
    pub fn unregister(&mut self, buffer: &Buffer) {
        unsafe {
            if (*self.items).remove(buffer.id()).is_none() {
                return;
            }

            let runtime = buffer.runtime().unwrap();
            (*self.used_memory) -= runtime.bytes_length;
            for binding in &runtime.bindings {
                if let Entry::Occupied(entry) = (*self.bindings).entry(*binding) {
                    if buffer.id() == entry.get() {
                        entry.remove();
                    }
                }
            }
            for binding in &runtime.binding_ubos {
                if let Entry::Occupied(entry) = (*self.binding_ubos).entry(*binding) {
                    if buffer.id() == entry.get() {
                        entry.remove();
                    }
                }
            }

            if let Some(registered) = buffer.registered.borrow_mut().take() {
                (*self.lru).remove(registered.lru_node);
            }
        }
    }
}
