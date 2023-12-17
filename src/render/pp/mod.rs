pub mod error;
pub mod graph;

use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
};

use uuid::Uuid;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::collection::EntityCollection};

use self::{error::Error, graph::DirectedGraph};

use super::webgl::{buffer::BufferStore, program::ProgramStore, texture::TextureStore};

/// Pipeline rendering stuff.
pub trait Stuff {
    /// Gets entity collection that should be draw on current frame.
    fn entity_collection(&self) -> &EntityCollection;

    /// Gets mutable entity collection that should be draw on current frame.
    fn entity_collection_mut(&mut self) -> &mut EntityCollection;

    /// Gets the main camera for current frame.
    fn camera(&self) -> &dyn Camera;

    /// Gets the mutable main camera for current frame.
    fn camera_mut(&mut self) -> &mut dyn Camera;
}

/// Pipeline rendering state.
pub struct State<'a> {
    program_store: &'a mut ProgramStore,
    buffer_store: &'a mut BufferStore,
    texture_store: &'a mut TextureStore,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    timestamp: f64,
}

impl<'a> State<'a> {
    /// Constructs a new rendering state.
    pub(crate) fn new(
        gl: WebGl2RenderingContext,
        canvas: HtmlCanvasElement,
        timestamp: f64,
        program_store: &'a mut ProgramStore,
        buffer_store: &'a mut BufferStore,
        texture_store: &'a mut TextureStore,
    ) -> Self {
        Self {
            gl,
            canvas,
            timestamp,
            program_store,
            buffer_store,
            texture_store,
        }
    }

    /// Gets the [`WebGl2RenderingContext`] associated with the canvas.
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Gets the [`HtmlCanvasElement`] to be drawn to.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Gets the `requestAnimationFrame` timestamp.
    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Gets the [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store(&self) -> &ProgramStore {
        &self.program_store
    }

    /// Gets the mutable [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        &mut self.program_store
    }

    /// Gets the [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    /// Gets the mutable [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }

    /// Gets the [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store(&self) -> &TextureStore {
        &self.texture_store
    }

    /// Gets the mutable [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store_mut(&mut self) -> &mut TextureStore {
        &mut self.texture_store
    }

    /// Resets WebGl state
    fn reset_gl(&self) {
        self.gl.use_program(None);
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::COPY_READ_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::COPY_WRITE_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::PIXEL_PACK_BUFFER, None);
        self.gl
            .bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, None);
        for index in 0..32 {
            self.gl
                .active_texture(WebGl2RenderingContext::TEXTURE0 + index);
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, None);
        }
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        self.gl.bind_vertex_array(None);
        self.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        self.gl.disable(WebGl2RenderingContext::CULL_FACE);
        self.gl.disable(WebGl2RenderingContext::BLEND);
        self.gl.disable(WebGl2RenderingContext::DITHER);
        self.gl.disable(WebGl2RenderingContext::POLYGON_OFFSET_FILL);
        self.gl
            .disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        self.gl.disable(WebGl2RenderingContext::SAMPLE_COVERAGE);
        self.gl.disable(WebGl2RenderingContext::SCISSOR_TEST);
        self.gl.disable(WebGl2RenderingContext::STENCIL_TEST);
        self.gl.disable(WebGl2RenderingContext::RASTERIZER_DISCARD);
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear_depth(1.0);
        self.gl.clear_stencil(0);
        self.gl.depth_mask(true);
        self.gl.stencil_func(WebGl2RenderingContext::ALWAYS, 0, 1);
        self.gl.stencil_mask(1);
        self.gl.stencil_op(
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
        );
    }
}

/// [`String`] or [`Uuid`] based key for storing items in pipeline.
/// 2 available types:
///
/// 1. String key for common purpose.
/// 2. Random generated uuid for hard coding purpose.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemKey {
    String(String),
    Uuid(Uuid),
}

impl ItemKey {
    /// Constructs a new item key by [`Uuid`].
    pub fn from_uuid() -> Self {
        Self::Uuid(Uuid::new_v4())
    }

    /// Constructs a new item key by [`String`].
    pub fn from_string(name: impl Into<String>) -> Self {
        Self::String(name.into())
    }
}

/// A graph based pipeline defining the rendering procedure.
pub struct Pipeline<ExecutorError> {
    graph: DirectedGraph<Box<dyn Executor<Error = ExecutorError>>>,
    executor_keys: HashMap<ItemKey, usize>,
    resources: Resources,
}

impl<ExecutorError> Pipeline<ExecutorError> {
    /// Constructs a new pipeline.
    pub fn new() -> Self {
        Self {
            graph: DirectedGraph::new(),
            executor_keys: HashMap::new(),
            resources: Resources::new(),
        }
    }

    pub fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut impl Stuff,
    ) -> Result<bool, ExecutorError> {
        let Ok(iter) = self.graph.iter_mut() else {
            return Ok(false);
        };

        for (_, executor) in iter {
            if executor.before(state, stuff, &mut self.resources)? {
                executor.execute(state, stuff, &mut self.resources)?;
                executor.after(state, stuff, &mut self.resources)?;
            }

            state.reset_gl();
        }

        // clears runtime resources
        self.resources.runtime.clear();

        Ok(true)
    }

    pub fn add_executor<E>(&mut self, key: ItemKey, executor: E)
    where
        E: Executor<Error = ExecutorError> + 'static,
    {
        let index = self.graph.add_vertex(Box::new(executor));
        self.executor_keys.insert(key, index);
    }

    pub fn remove_executor(&mut self, key: &ItemKey) -> Result<(), Error> {
        let Some(index) = self.executor_keys.remove(key) else {
            return Err(self::error::Error::NoSuchExecutor(key.clone()))?;
        };
        self.graph.remove_vertex(index);
        self.executor_keys.iter_mut().for_each(|(_, v)| {
            if *v > index {
                *v -= 1
            }
        });

        Ok(())
    }

    pub fn executor(
        &self,
        key: &ItemKey,
    ) -> Result<Option<&dyn Executor<Error = ExecutorError>>, Error> {
        let Some(index) = self.executor_keys.get(key) else {
            return Err(self::error::Error::NoSuchExecutor(key.clone()))?;
        };

        match self.graph.vertex(*index) {
            Some(executor) => Ok(Some(executor.as_ref())),
            None => Ok(None),
        }
    }

    pub fn executor_mut(
        &mut self,
        key: &ItemKey,
    ) -> Result<Option<&mut dyn Executor<Error = ExecutorError>>, Error> {
        let Some(index) = self.executor_keys.get(key) else {
            return Err(self::error::Error::NoSuchExecutor(key.clone()))?;
        };

        match self.graph.vertex_mut(*index) {
            Some(executor) => Ok(Some(executor.as_mut())),
            None => Ok(None),
        }
    }

    pub fn connect(&mut self, from: &ItemKey, to: &ItemKey) -> Result<(), Error> {
        let from_index = self
            .executor_keys
            .get(from)
            .ok_or(self::error::Error::NoSuchExecutor(from.clone()))?;
        let to_index = self
            .executor_keys
            .get(to)
            .ok_or(self::error::Error::NoSuchExecutor(to.clone()))?;

        self.graph.add_arc(*from_index, *to_index)?;

        Ok(())
    }

    pub fn disconnect(&mut self, from: &ItemKey, to: &ItemKey) -> Result<(), Error> {
        let from_index = self
            .executor_keys
            .get(from)
            .ok_or(self::error::Error::NoSuchExecutor(from.clone()))?;
        let to_index = self
            .executor_keys
            .get(to)
            .ok_or(self::error::Error::NoSuchExecutor(to.clone()))?;

        self.graph.remove_arc(*from_index, *to_index);

        Ok(())
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

/// Resource key based on [`ItemKey`].
/// Distinguish between runtime resource and persist resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceKey<V> {
    Runtime(ItemKey, PhantomData<V>),
    Persist(ItemKey, PhantomData<V>),
}

impl<V> ResourceKey<V> {
    /// Constructs a new string runtime resource key.
    pub fn runtime_str<S: Into<String>>(key: S) -> Self {
        Self::Runtime(ItemKey::from_string(key), PhantomData::<V>)
    }

    /// Constructs a new string persist resource key.
    pub fn persist_str<S: Into<String>>(key: S) -> Self {
        Self::Persist(ItemKey::from_string(key), PhantomData::<V>)
    }

    /// Constructs a new runtime resource key with random uuid.
    pub fn runtime_uuid() -> Self {
        Self::Runtime(ItemKey::from_uuid(), PhantomData::<V>)
    }

    /// Constructs a new persist resource key with random uuid.
    pub fn persist_uuid() -> Self {
        Self::Persist(ItemKey::from_uuid(), PhantomData::<V>)
    }

    pub fn raw(&self) -> &ItemKey {
        match self {
            ResourceKey::Runtime(key, _) => key,
            ResourceKey::Persist(key, _) => key,
        }
    }
}

/// Pipeline resources. 2 kinds of resources are available:
///
/// 1. Runtime Resources, pipeline cleanups data after each execution.
/// 2. Persist Resources, pipeline never cleanups data in persist resources.
pub struct Resources {
    runtime: HashMap<ItemKey, Box<dyn Any>>,
    persist: HashMap<ItemKey, Box<dyn Any>>,
}

impl Resources {
    /// Constructs a new pipeline resources container.
    pub fn new() -> Self {
        Self {
            runtime: HashMap::new(),
            persist: HashMap::new(),
        }
    }

    /// Gets a resource by a specified [`ResourceKey`].
    pub fn get<V: 'static>(&self, key: &ResourceKey<V>) -> Option<&V> {
        let value = match key {
            ResourceKey::Runtime(key, _) => self.runtime.get(key),
            ResourceKey::Persist(key, _) => self.persist.get(key),
        };

        match value {
            Some(value) => value.downcast_ref::<V>(),
            None => None,
        }
    }

    /// Gets a mutable resource by a specified [`ResourceKey`].
    pub fn get_mut<V: 'static>(&mut self, key: &ResourceKey<V>) -> Option<&mut V> {
        let value = match key {
            ResourceKey::Runtime(key, _) => self.runtime.get_mut(key),
            ResourceKey::Persist(key, _) => self.persist.get_mut(key),
        };

        match value {
            Some(value) => value.downcast_mut::<V>(),
            None => None,
        }
    }

    /// Inserts a new resource by a [`ResourceKey`].
    pub fn insert<V: 'static>(&mut self, key: ResourceKey<V>, value: V) {
        match key {
            ResourceKey::Runtime(key, _) => self.runtime.insert(key, Box::new(value)),
            ResourceKey::Persist(key, _) => self.persist.insert(key, Box::new(value)),
        };
    }

    /// Removes a resource by a [`ResourceKey`] and unchecks downcast result.
    pub fn remove_unchecked<V>(&mut self, key: &ResourceKey<V>) -> Option<Box<dyn Any>> {
        match key {
            ResourceKey::Runtime(key, _) => self.runtime.remove(key),
            ResourceKey::Persist(key, _) => self.persist.remove(key),
        }
    }

    /// Removes a resource by a [`ResourceKey`], checks downcast result before removing.
    pub fn remove<V: 'static>(&mut self, key: ResourceKey<V>) -> Option<V> {
        let entry = match key {
            ResourceKey::Runtime(key, _) => self.runtime.entry(key),
            ResourceKey::Persist(key, _) => self.persist.entry(key),
        };

        match entry {
            Entry::Occupied(occupied) => {
                if occupied.get().downcast_ref::<V>().is_some() {
                    let value = occupied.remove().downcast::<V>().unwrap();
                    Some(*value)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => None,
        }
    }

    /// Returns `true` if the resources contains a value for the specified [`ResourceKey`]
    /// and successfully downcast to specified type.
    pub fn contains_key<V: 'static>(&mut self, key: &ResourceKey<V>) -> bool {
        self.get(key).is_some()
    }

    /// Returns `true` if the resources contains a value for the specified [`ResourceKey`].
    pub fn contains_key_unchecked<V>(&mut self, key: &ResourceKey<V>) -> bool {
        match key {
            ResourceKey::Runtime(key, _) => self.runtime.contains_key(key),
            ResourceKey::Persist(key, _) => self.persist.contains_key(key),
        }
    }

    /// Gets the native runtime resources.
    pub fn runtime(&self) -> &HashMap<ItemKey, Box<dyn Any>> {
        &self.runtime
    }

    /// Gets the native persist resources.
    pub fn persist(&self) -> &HashMap<ItemKey, Box<dyn Any>> {
        &self.persist
    }

    /// Gets the mutable native runtime resources.
    pub fn runtime_mut(&mut self) -> &mut HashMap<ItemKey, Box<dyn Any>> {
        &mut self.runtime
    }

    /// Gets the mutable native persist resources.
    pub fn persist_mut(&mut self) -> &mut HashMap<ItemKey, Box<dyn Any>> {
        &mut self.persist
    }
}

/// An execution node for [`Pipeline`].
pub trait Executor {
    type Error;

    /// Actions before execution.
    /// Developer could setup WebGL state here, or return a `false` to skip execution.
    #[allow(unused)]
    fn before(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    /// Main execution procedure.
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error>;

    /// Actions after execution.
    /// Developer should reset WebGL state here to prevent unexpected side effect to other executors.
    #[allow(unused)]
    fn after(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
