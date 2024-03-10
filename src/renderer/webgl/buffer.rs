use std::{
    borrow::Cow,
    cell::RefCell,
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
        Int16Array, Int32Array, Int8Array, Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
    },
    WebGl2RenderingContext, WebGlBuffer,
};

use crate::{
    lru::{Lru, LruNode},
    renderer::webgl::params::GetWebGlParameters,
    utils::format_byte_length,
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
    pub fn byte_length(&self) -> usize {
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

/// A trait defining a buffer source for uploading data to WebGL runtime.
pub trait BufferSource: Debug {
    /// Returns bytes length of the data.
    fn byte_length(&self) -> usize;

    /// Buffers data to WebGL runtime.
    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage);

    /// Buffers sub data to WebGL runtime.
    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    );
}

impl BufferSource for &[u8] {
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        gl.buffer_data_with_u8_array(target.gl_enum(), self, usage.gl_enum())
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        gl.buffer_sub_data_with_i32_and_u8_array(target.gl_enum(), dst_byte_offset as i32, self);
    }
}

impl BufferSource for Vec<u8> {
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        self.as_slice().buffer_data(gl, target, usage)
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        self.as_slice().buffer_sub_data(gl, target, dst_byte_offset)
    }
}

impl<const N: usize> BufferSource for [u8; N] {
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        self.as_slice().buffer_data(gl, target, usage)
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        self.as_slice().buffer_sub_data(gl, target, dst_byte_offset)
    }
}

impl<const N: usize> BufferSource for &[u8; N] {
    fn byte_length(&self) -> usize {
        self.len()
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        self.as_slice().buffer_data(gl, target, usage)
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        self.as_slice().buffer_sub_data(gl, target, dst_byte_offset)
    }
}

impl BufferSource for ArrayBuffer {
    fn byte_length(&self) -> usize {
        self.byte_length() as usize
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        gl.buffer_data_with_opt_array_buffer(target.gl_enum(), Some(self), usage.gl_enum())
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        gl.buffer_sub_data_with_i32_and_array_buffer(target.gl_enum(), dst_byte_offset as i32, self)
    }
}

macro_rules! array_buffer_view_sources {
    ($($source:ident),+) => {
        $(
            impl BufferSource for $source {
                fn byte_length(&self) -> usize {
                    self.byte_length() as usize
                }

                fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
                    gl.buffer_data_with_array_buffer_view(target.gl_enum(), &self, usage.gl_enum());
                }

                fn buffer_sub_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, dst_byte_offset: usize) {
                    gl.buffer_sub_data_with_i32_and_array_buffer_view(target.gl_enum(), dst_byte_offset as i32, &self)
                }
            }
        )+
    };
}

array_buffer_view_sources!(
    DataView,
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Float32Array,
    Float64Array,
    BigInt64Array,
    BigUint64Array
);

/// Preallocated buffer source.
#[derive(Debug)]
pub struct Preallocation(usize);

impl Preallocation {
    /// Constructs a new preallocated buffer source with specified size.
    pub fn new(size: usize) -> Self {
        Self(size)
    }
}

impl BufferSource for Preallocation {
    fn byte_length(&self) -> usize {
        self.0
    }

    fn buffer_data(&self, gl: &WebGl2RenderingContext, target: BufferTarget, usage: BufferUsage) {
        gl.buffer_data_with_i32(target.gl_enum(), self.0 as i32, usage.gl_enum());
    }

    fn buffer_sub_data(
        &self,
        gl: &WebGl2RenderingContext,
        target: BufferTarget,
        dst_byte_offset: usize,
    ) {
        gl.buffer_sub_data_with_i32_and_array_buffer(
            target.gl_enum(),
            dst_byte_offset as i32,
            &ArrayBuffer::new(self.0 as u32),
        );
    }
}

#[derive(Debug)]
struct QueueItem {
    source: Box<dyn BufferSource>,
    byte_offset: usize,
}

impl QueueItem {
    fn new<S>(source: S, byte_offset: usize) -> Self
    where
        S: BufferSource + 'static,
    {
        Self {
            source: Box::new(source),
            byte_offset,
        }
    }

    fn new_boxed(source: Box<dyn BufferSource>, byte_offset: usize) -> Self {
        Self {
            source,
            byte_offset,
        }
    }
}

#[derive(Debug)]
struct Queue {
    required_byte_length: usize,
    items: Vec<QueueItem>,
}

impl Queue {
    fn new() -> Self {
        Self {
            required_byte_length: 0,
            items: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct BufferRuntime {
    gl: WebGl2RenderingContext,
    buffer: Option<WebGlBuffer>,
    buffer_byte_length: usize,
    bindings: HashSet<BufferTarget>,
    binding_ubos: HashSet<u32>,
}

impl BufferRuntime {
    fn get_or_create_buffer(&mut self) -> Result<WebGlBuffer, Error> {
        match self.buffer.as_mut() {
            Some(buffer) => Ok(buffer.clone()),
            None => {
                let buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                Ok(self.buffer.insert(buffer).clone())
            }
        }
    }

    fn read_back(&self) -> Option<ArrayBuffer> {
        let Some(buffer) = self.buffer.as_ref() else {
            return None;
        };
        if self.buffer_byte_length == 0 {
            return None;
        }

        let gl = &self.gl;
        let data = Uint8Array::new_with_length(self.buffer_byte_length as u32);
        let binding = gl.array_buffer_binding();
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        gl.get_buffer_sub_data_with_i32_and_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            &data,
        );
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        Some(data.buffer())
    }

    fn upload(
        &mut self,
        target: BufferTarget,
        usage: BufferUsage,
        queue: &mut Queue,
    ) -> (usize, usize) {
        if queue.items.len() > 0 {
            let required_byte_length = queue.required_byte_length;
            let current_byte_length = self.buffer_byte_length;

            if required_byte_length >= current_byte_length {
                self.gl.buffer_data_with_i32(
                    target.gl_enum(),
                    required_byte_length as i32,
                    usage.gl_enum(),
                );
                self.buffer_byte_length = required_byte_length;
            }

            for item in queue.items.drain(..) {
                item.source
                    .buffer_sub_data(&self.gl, target, item.byte_offset);
            }

            self.buffer_byte_length = required_byte_length;

            debug!(
                target: "BufferStore",
                "buffer new data, old length {}, new length {}",
                current_byte_length,
                required_byte_length
            );

            (required_byte_length, current_byte_length)
        } else {
            (0, 0)
        }
    }
}

#[derive(Debug)]
struct BufferRegistered {
    store: Rc<RefCell<StoreShared>>,
    store_id: Uuid,
    lru_node: *mut LruNode<Uuid>,
}

#[derive(Debug)]
struct BufferShared {
    id: Uuid,
    usage: BufferUsage,
    memory_policy: MemoryPolicy,
    queue: Queue,
    registered: Option<BufferRegistered>,
    runtime: Option<BufferRuntime>,
}

impl BufferShared {
    fn init(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        match &self.runtime {
            Some(runtime) => {
                if &runtime.gl == gl {
                    Ok(())
                } else {
                    Err(Error::BufferAlreadyInitialized)
                }
            }
            None => {
                self.runtime = Some(BufferRuntime {
                    gl: gl.clone(),
                    buffer: None,
                    buffer_byte_length: 0,
                    bindings: HashSet::new(),
                    binding_ubos: HashSet::new(),
                });
                Ok(())
            }
        }
    }

    fn bind(&mut self, target: BufferTarget) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        if let Some(registered) = &self.registered {
            if registered.store.borrow().is_occupied(target, &self.id) {
                return Err(Error::BufferTargetOccupied(target));
            }
        }

        let buffer = runtime.get_or_create_buffer()?;
        runtime.gl.bind_buffer(target.gl_enum(), Some(&buffer));
        let (new_byte_length, old_byte_length) =
            runtime.upload(target, self.usage, &mut self.queue);
        runtime.bindings.insert(target);

        if let Some(registered) = &mut self.registered {
            let mut store = registered.store.borrow_mut();
            store.update_lru(registered.lru_node);
            store.update_used_memory(new_byte_length, old_byte_length);
            store.add_binding(target, self.id);
            store.free();
        }

        Ok(())
    }

    fn bind_ubo(&mut self, index: u32) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        if let Some(registered) = &self.registered {
            if registered.store.borrow().is_occupied_ubo(index, &self.id) {
                return Err(Error::UniformBufferObjectIndexOccupied(index));
            }
        }

        let buffer = runtime.get_or_create_buffer()?;
        let binding = runtime.gl.uniform_buffer_binding();
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
        let (new_byte_length, old_byte_length) =
            runtime.upload(BufferTarget::UNIFORM_BUFFER, self.usage, &mut self.queue);
        runtime.gl.bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            index,
            runtime.buffer.as_ref(),
        );
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, binding.as_ref());
        runtime.binding_ubos.insert(index);

        if let Some(registered) = &self.registered {
            let mut store = registered.store.borrow_mut();
            store.update_lru(registered.lru_node);
            store.update_used_memory(new_byte_length, old_byte_length);
            store.add_binding_ubo(index, self.id);
            store.free();
        }

        Ok(())
    }

    fn bind_ubo_range(&mut self, index: u32, offset: i32, size: i32) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        if let Some(registered) = &self.registered {
            if registered.store.borrow().is_occupied_ubo(index, &self.id) {
                return Err(Error::UniformBufferObjectIndexOccupied(index));
            }
        }

        let buffer = runtime.get_or_create_buffer()?;
        let binding = runtime.gl.uniform_buffer_binding();
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
        let (new_byte_length, old_byte_length) =
            runtime.upload(BufferTarget::UNIFORM_BUFFER, self.usage, &mut self.queue);
        runtime.gl.bind_buffer_range_with_i32_and_i32(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            index,
            runtime.buffer.as_ref(),
            offset,
            size,
        );
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, binding.as_ref());
        runtime.binding_ubos.insert(index);

        if let Some(registered) = &self.registered {
            let mut store = registered.store.borrow_mut();
            store.update_lru(registered.lru_node);
            store.update_used_memory(new_byte_length, old_byte_length);
            store.add_binding_ubo(index, self.id);
            store.free();
        }

        Ok(())
    }

    fn unbind(&mut self, target: BufferTarget) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        if runtime.bindings.remove(&target) {
            runtime.gl.bind_buffer(target.gl_enum(), None);
        }

        if let Some(registered) = &self.registered {
            registered.store.borrow_mut().remove_binding(target);
        }

        Ok(())
    }

    fn unbind_ubo(&mut self, index: u32) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        if runtime.binding_ubos.remove(&index) {
            runtime
                .gl
                .bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);
        }

        if let Some(registered) = self.registered.as_mut() {
            registered.store.borrow_mut().remove_binding_ubo(index);
        }

        Ok(())
    }

    fn unbind_all(&mut self) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        let gl = runtime.gl.clone();
        for index in runtime.binding_ubos.drain() {
            gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);

            if let Some(registered) = &self.registered {
                registered.store.borrow_mut().remove_binding_ubo(index);
            }
        }
        for target in runtime.bindings.drain() {
            gl.bind_buffer(target.gl_enum(), None);

            if let Some(registered) = &self.registered {
                registered.store.borrow_mut().remove_binding(target);
            }
        }

        Ok(())
    }

    fn upload(&mut self) -> Result<(), Error> {
        let runtime = self.runtime.as_mut().ok_or(Error::BufferUninitialized)?;

        let buffer = runtime.get_or_create_buffer()?;
        let binding = runtime.gl.array_buffer_binding();
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        let (new_byte_length, old_byte_length) =
            runtime.upload(BufferTarget::ARRAY_BUFFER, self.usage, &mut self.queue);
        runtime
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, binding.as_ref());

        if let Some(registered) = &self.registered {
            let mut store = registered.store.borrow_mut();
            store.update_used_memory(new_byte_length, old_byte_length);
            store.free();
        }

        Ok(())
    }

    fn clear(&mut self, read_back: bool, new_usage: Option<BufferUsage>) {
        if let Some(usage) = new_usage {
            self.usage = usage;
        }

        self.queue.items.clear();
        self.queue.required_byte_length = 0;

        if let Some(runtime) = &mut self.runtime {
            if read_back {
                if let Some(data) = runtime.read_back() {
                    self.queue.items.push(QueueItem::new(data, 0));
                }
            }

            let new_byte_length = 0;
            let old_byte_length = runtime.buffer_byte_length;
            let gl = runtime.gl.clone();
            if let Some(buffer) = runtime.buffer.take() {
                for index in runtime.binding_ubos.drain() {
                    gl.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, index, None);

                    if let Some(registered) = &self.registered {
                        registered.store.borrow_mut().remove_binding_ubo(index);
                    }
                }
                for target in runtime.bindings.drain() {
                    gl.bind_buffer(target.gl_enum(), None);

                    if let Some(registered) = &self.registered {
                        registered.store.borrow_mut().remove_binding(target);
                    }
                }
                gl.delete_buffer(Some(&buffer))
            }
            runtime.buffer_byte_length = new_byte_length;

            if let Some(registered) = &self.registered {
                registered
                    .store
                    .borrow_mut()
                    .update_used_memory(new_byte_length, old_byte_length);
            }
        }
    }

    /// Overrides existing data and then buffers new data.
    fn buffer_data<S>(&mut self, source: S)
    where
        S: BufferSource + 'static,
    {
        self.queue.required_byte_length = source.byte_length();
        self.queue.items.clear();
        self.queue.items.push(QueueItem::new(source, 0));
    }

    /// Buffers sub data.
    fn buffer_sub_data<S>(&mut self, source: S, dst_byte_offset: usize)
    where
        S: BufferSource + 'static,
    {
        let byte_length = dst_byte_offset + source.byte_length();

        if dst_byte_offset == 0 {
            if byte_length >= self.queue.required_byte_length {
                // overrides sources in queue if new source covers all
                self.queue.required_byte_length = byte_length;
                self.queue.items.clear();
                self.queue.items.push(QueueItem::new(source, 0));
            } else {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            }
        } else {
            if byte_length <= self.queue.required_byte_length {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            } else {
                // heavy job!
                if let Some(readback) = self
                    .runtime
                    .as_ref()
                    .and_then(|runtime| runtime.read_back())
                {
                    self.queue
                        .items
                        .insert(0, QueueItem::new(Preallocation(byte_length), 0));
                    self.queue.items.insert(1, QueueItem::new(readback, 0));
                    self.queue
                        .items
                        .push(QueueItem::new(source, dst_byte_offset));
                } else {
                    self.queue
                        .items
                        .insert(0, QueueItem::new(Preallocation(byte_length), 0));
                    self.queue
                        .items
                        .push(QueueItem::new(source, dst_byte_offset));
                }
                self.queue.required_byte_length = byte_length;
            }
        }
    }

    fn free(&mut self) {
        let Some(runtime) = self.runtime.as_mut() else {
            return;
        };

        // skips if using
        if runtime.bindings.len() + runtime.binding_ubos.len() != 0 {
            return;
        }

        // free
        match &self.memory_policy {
            MemoryPolicy::Unfree => {}
            MemoryPolicy::ReadBack => {
                let byte_length = runtime.buffer_byte_length;
                if let Some(buffer) = runtime.buffer.take() {
                    if let Some(readback) = runtime.read_back() {
                        self.queue.items.insert(0, QueueItem::new(readback, 0));
                    }
                    self.queue.required_byte_length = self.queue.required_byte_length.max(byte_length);
                    runtime.gl.delete_buffer(Some(&buffer));
                    runtime.buffer_byte_length = 0;
                }

                if let Some(registered) = self.registered.as_mut() {
                    let mut store = registered.store.borrow_mut();
                    store.used_memory -= byte_length;
                    unsafe {
                        store.lru.remove(registered.lru_node);
                    }

                    debug!(
                        target: "BufferStore",
                        "free buffer (readback) {}. freed memory {}, used {}",
                        self.id,
                        format_byte_length(byte_length),
                        format_byte_length(store.used_memory)
                    );
                } else {
                    debug!(
                        target: "BufferStore",
                        "free buffer (readback) {}. freed memory {}",
                        self.id,
                        format_byte_length(byte_length),
                    );
                }
            }
            MemoryPolicy::Restorable(restorer) => {
                let byte_length = runtime.buffer_byte_length;

                if let Some(buffer) = runtime.buffer.take() {
                    runtime.gl.delete_buffer(Some(&buffer));
                    runtime.buffer_byte_length = 0;
                }

                let source = restorer.restore();
                self.queue.items.insert(0, QueueItem::new_boxed(source, 0));
                self.queue.required_byte_length = self.queue.required_byte_length.max(byte_length);

                if let Some(registered) = self.registered.as_mut() {
                    let mut store = registered.store.borrow_mut();
                    store.used_memory -= byte_length;
                    unsafe {
                        store.lru.remove(registered.lru_node);
                    }

                    debug!(
                        target: "BufferStore",
                        "free buffer (restorable) {}. freed memory {}, used {}",
                        self.id,
                        format_byte_length(byte_length),
                        format_byte_length(store.used_memory)
                    );
                } else {
                    debug!(
                        target: "BufferStore",
                        "free buffer (restorable) {}. freed memory {}",
                        self.id,
                        format_byte_length(byte_length),
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    name: Option<Cow<'static, str>>,
    shared: Rc<RefCell<BufferShared>>,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let mut buffer_shared = self.shared.borrow_mut();
        let id = buffer_shared.id;

        if let Some(registered) = buffer_shared.registered.as_mut() {
            let mut store_shared = registered.store.borrow_mut();
            store_shared.items.remove(&id);
            unsafe {
                store_shared.lru.remove(registered.lru_node);
            }
        }

        if let Some(runtime) = buffer_shared.runtime.as_mut() {
            if let Some(buffer) = runtime.buffer.take() {
                runtime.gl.delete_buffer(Some(&buffer));
            }
        }
    }
}

impl Buffer {
    /// Constructs a new buffer with specified name, [`BufferSource`], [`BufferUsage`] and [`MemoryPolicy`].
    pub fn new(
        name: Option<Cow<'static, str>>,
        usage: BufferUsage,
        memory_policy: MemoryPolicy,
    ) -> Self {
        let shared = BufferShared {
            id: Uuid::new_v4(),
            memory_policy,
            usage,
            queue: Queue::new(),
            registered: None,
            runtime: None,
        };
        Self {
            name,
            shared: Rc::new(RefCell::new(shared)),
        }
    }

    /// Returns id of this buffer.
    pub fn id(&self) -> Uuid {
        self.shared.borrow().id
    }

    /// Returns buffer name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Sets buffer name.
    pub fn set_name(&mut self, name: Option<Cow<'static, str>>) {
        self.name = name;
    }

    /// Returns [`BufferUsage`].
    pub fn usage(&self) -> BufferUsage {
        self.shared.borrow().usage
    }

    /// Returns [`MemoryPolicyKind`] associated with the [`MemoryPolicy`].
    pub fn memory_policy(&self) -> MemoryPolicyKind {
        self.shared.borrow().memory_policy.kind()
    }

    /// Sets [`MemoryPolicy`].
    pub fn set_memory_policy(&mut self, memory_policy: MemoryPolicy) {
        self.shared.borrow_mut().memory_policy = memory_policy;
    }

    /// Initializes this buffer by a [`WebGl2RenderingContext`].
    pub fn init(&self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        self.shared.borrow_mut().init(gl)
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind(&self, target: BufferTarget) -> Result<(), Error> {
        self.shared.borrow_mut().bind(target)
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind_ubo(&self, index: u32) -> Result<(), Error> {
        self.shared.borrow_mut().bind_ubo(index)
    }

    /// Binds buffer to specified [`BufferTarget`].
    pub fn bind_ubo_range(&self, index: u32, offset: i32, size: i32) -> Result<(), Error> {
        self.shared.borrow_mut().bind_ubo_range(index, offset, size)
    }

    /// Unbinds buffer from specified [`BufferTarget`].
    pub fn unbind(&self, target: BufferTarget) -> Result<(), Error> {
        self.shared.borrow_mut().unbind(target)
    }

    /// Unbinds buffer from specified uniform buffer object index.
    pub fn unbind_ubo(&self, index: u32) -> Result<(), Error> {
        self.shared.borrow_mut().unbind_ubo(index)
    }

    /// Unbinds buffer from all bindings, including uniform buffer objects.
    pub fn unbind_all(&self) -> Result<(), Error> {
        self.shared.borrow_mut().unbind_all()
    }

    /// Uploads data to WebGL runtime.
    pub fn upload(&self) -> Result<(), Error> {
        self.shared.borrow_mut().upload()
    }

    /// Clears and unbinds buffer from WebGL runtime as well as replacing a new [`BufferUsage`].
    /// Data will be read back from WebGL runtime and
    /// insert to the first place of the queue if `read_back` is `true`.
    pub fn clear(&self, read_back: bool, new_usage: Option<BufferUsage>) {
        self.shared.borrow_mut().clear(read_back, new_usage);
    }

    /// Reads buffer data back from WebGL runtime and stores it to an [`ArrayBuffer`].
    pub fn read_back(&self) -> Result<Option<ArrayBuffer>, Error> {
        let shared = self.shared.borrow();
        let Some(runtime) = &shared.runtime else {
            return Err(Error::BufferUninitialized);
        };

        Ok(runtime.read_back())
    }

    /// Overrides existing data and then buffers new data.
    pub fn buffer_data<S>(&self, source: S)
    where
        S: BufferSource + 'static,
    {
        self.shared.borrow_mut().buffer_data(source);
    }

    /// Buffers sub data.
    pub fn buffer_sub_data<S>(&self, source: S, dst_byte_offset: usize)
    where
        S: BufferSource + 'static,
    {
        self.shared
            .borrow_mut()
            .buffer_sub_data(source, dst_byte_offset)
    }
}

pub trait Restorer: Debug {
    fn restore(&self) -> Box<dyn BufferSource>;
}

/// Memory policies kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryPolicyKind {
    Unfree,
    ReadBack,
    Restorable,
}

/// Memory policies.
#[derive(Debug)]
pub enum MemoryPolicy {
    Unfree,
    ReadBack,
    Restorable(Box<dyn Restorer>),
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
        Self::Restorable(Box::new(restorer))
    }

    /// Returns [`MemoryPolicyKind`] associated with the [`MemoryPolicy`].
    pub fn kind(&self) -> MemoryPolicyKind {
        match self {
            MemoryPolicy::Unfree => MemoryPolicyKind::Unfree,
            MemoryPolicy::ReadBack => MemoryPolicyKind::ReadBack,
            MemoryPolicy::Restorable(_) => MemoryPolicyKind::Restorable,
        }
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
    pub fn buffer_data<S>(mut self, source: S) -> Self
    where
        S: BufferSource + 'static,
    {
        self.queue.required_byte_length = source.byte_length();
        self.queue.items.clear();
        self.queue.items.push(QueueItem::new(source, 0));
        self
    }

    /// Buffers sub data.
    pub fn buffer_sub_data<S>(mut self, source: S, dst_byte_offset: usize) -> Self
    where
        S: BufferSource + 'static,
    {
        let byte_length = dst_byte_offset + source.byte_length();
        if dst_byte_offset == 0 {
            if byte_length >= self.queue.required_byte_length {
                // overrides sources in queue if new source covers all
                self.queue.required_byte_length = byte_length;
                self.queue.items.clear();
                self.queue.items.push(QueueItem::new(source, 0));
            } else {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            }
        } else {
            if byte_length <= self.queue.required_byte_length {
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
            } else {
                self.queue
                    .items
                    .insert(0, QueueItem::new(Preallocation(byte_length), 0));
                self.queue
                    .items
                    .push(QueueItem::new(source, dst_byte_offset));
                self.queue.required_byte_length = byte_length;
            }
        }
        self
    }

    pub fn build(self) -> Buffer {
        let shared = BufferShared {
            id: Uuid::new_v4(),
            usage: self.usage,
            memory_policy: self.memory_policy,
            queue: self.queue,
            registered: None,
            runtime: None,
        };
        Buffer {
            name: self.name,
            shared: Rc::new(RefCell::new(shared)),
        }
    }
}

struct StoreShared {
    gl: WebGl2RenderingContext,

    available_memory: usize,
    used_memory: usize,

    lru: Lru<Uuid>,
    items: HashMap<Uuid, Weak<RefCell<BufferShared>>>,
    bindings: HashMap<BufferTarget, Uuid>,
    binding_ubos: HashMap<u32, Uuid>,
}

impl Debug for StoreShared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreShared")
            .field("available_memory", &self.available_memory)
            .field("used_memory", &self.used_memory)
            .field("items", &self.items)
            .field("bindings", &self.bindings)
            .field("binding_ubos", &self.binding_ubos)
            .finish()
    }
}

impl StoreShared {
    fn update_lru(&mut self, lru_node: *mut LruNode<Uuid>) {
        unsafe {
            self.lru.cache(lru_node);
        }
    }

    fn update_used_memory(&mut self, new_byte_length: usize, old_byte_length: usize) {
        self.used_memory = self.used_memory - old_byte_length + new_byte_length;
    }

    fn add_binding(&mut self, target: BufferTarget, id: Uuid) {
        self.bindings.insert(target, id);
    }

    fn add_binding_ubo(&mut self, index: u32, id: Uuid) {
        self.binding_ubos.insert(index, id);
    }

    fn remove_binding(&mut self, target: BufferTarget) {
        self.bindings.remove(&target);
    }

    fn remove_binding_ubo(&mut self, index: u32) {
        self.binding_ubos.remove(&index);
    }

    fn is_occupied(&self, target: BufferTarget, id: &Uuid) -> bool {
        self.bindings.get(&target).map(|v| v != id).unwrap_or(false)
    }

    fn is_occupied_ubo(&self, index: u32, id: &Uuid) -> bool {
        self.binding_ubos
            .get(&index)
            .map(|v| v != id)
            .unwrap_or(false)
    }

    /// Frees memory if used memory exceeds the maximum available memory.
    fn free(&mut self) {
        // removes buffer from the least recently used until memory usage lower than limitation
        unsafe {
            if self.used_memory <= self.available_memory {
                return;
            }

            let mut next_node = self.lru.least_recently();
            while self.used_memory > self.available_memory {
                let Some(current_node) = next_node.take() else {
                    break;
                };
                let id = (*current_node).data();

                let Entry::Occupied(occupied) = self.items.entry(*id) else {
                    next_node = (*current_node).more_recently();
                    continue;
                };
                let item = occupied.get();
                let Some(item) = item.upgrade() else {
                    // deletes if already dropped
                    occupied.remove();
                    next_node = (*current_node).more_recently();
                    continue;
                };

                if let Ok(mut item) = item.try_borrow_mut() {
                    item.free();
                }

                occupied.remove();
                next_node = (*current_node).more_recently();
            }
        }
    }
}

pub struct BufferStore {
    id: Uuid,
    shared: Rc<RefCell<StoreShared>>,
}

impl Drop for BufferStore {
    fn drop(&mut self) {
        let mut shared = self.shared.borrow_mut();
        for item in shared.items.values_mut() {
            let Some(item) = item.upgrade() else {
                continue;
            };
            item.borrow_mut().registered = None;
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
        let stored = StoreShared {
            gl,

            available_memory: available_memory.min(i32::MAX as usize),
            used_memory: 0,

            lru: Lru::new(),
            items: HashMap::new(),
            bindings: HashMap::new(),
            binding_ubos: HashMap::new(),
        };

        Self {
            id: Uuid::new_v4(),
            shared: Rc::new(RefCell::new(stored)),
        }
    }

    /// Returns the maximum available memory in bytes.
    /// Returns [`i32::MAX`] if not specified.
    pub fn available_memory(&self) -> usize {
        self.shared.borrow().available_memory
    }

    /// Returns current used memory in bytes.
    pub fn used_memory(&self) -> usize {
        self.shared.borrow().used_memory
    }

    /// Inits and registers a buffer to buffer store.
    pub fn register(&mut self, buffer: &Buffer) -> Result<(), Error> {
        unsafe {
            let mut store_shared = self.shared.borrow_mut();
            let mut buffer_shared = buffer.shared.borrow_mut();

            if let Some(registered) = buffer_shared.registered.as_ref() {
                if &registered.store_id != &self.id {
                    return Err(Error::RegisterBufferToMultipleStore);
                } else {
                    return Ok(());
                }
            }

            buffer_shared.init(&store_shared.gl)?;

            let runtime = buffer_shared.runtime.as_ref().unwrap();
            store_shared.used_memory += runtime.buffer_byte_length;
            for binding in &runtime.bindings {
                if store_shared.bindings.contains_key(binding) {
                    return Err(Error::BufferTargetOccupied(*binding));
                }
                store_shared.bindings.insert(*binding, buffer_shared.id);
            }
            for binding in &runtime.binding_ubos {
                if store_shared.binding_ubos.contains_key(binding) {
                    return Err(Error::UniformBufferObjectIndexOccupied(*binding));
                }
                store_shared.binding_ubos.insert(*binding, buffer_shared.id);
            }

            buffer_shared.registered = Some(BufferRegistered {
                store: Rc::clone(&self.shared),
                store_id: self.id,
                lru_node: LruNode::new(buffer_shared.id),
            });

            store_shared
                .items
                .insert(buffer_shared.id, Rc::downgrade(&buffer.shared));

            Ok(())
        }
    }

    /// Unregisters a buffer from buffer store.
    pub fn unregister(&mut self, buffer: &Buffer) {
        unsafe {
            let mut store_shared = self.shared.borrow_mut();
            let mut buffer_shared = buffer.shared.borrow_mut();

            if store_shared.items.remove(&buffer_shared.id).is_none() {
                return;
            }

            let runtime = buffer_shared.runtime.as_ref().unwrap();
            store_shared.used_memory -= runtime.buffer_byte_length;
            for binding in &runtime.bindings {
                if let Entry::Occupied(entry) = store_shared.bindings.entry(*binding) {
                    if &buffer_shared.id == entry.get() {
                        entry.remove();
                    }
                }
            }
            for binding in &runtime.binding_ubos {
                if let Entry::Occupied(entry) = store_shared.binding_ubos.entry(*binding) {
                    if &buffer_shared.id == entry.get() {
                        entry.remove();
                    }
                }
            }

            if let Some(registered) = buffer_shared.registered.take() {
                store_shared.lru.remove(registered.lru_node);
            }
        }
    }
}
