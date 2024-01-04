pub mod error;
pub mod graph;

use std::{
    any::Any,
    collections::{hash_map::Entry, HashMap},
    marker::PhantomData,
};

use gl_matrix4rust::vec3::Vec3;
use uuid::Uuid;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    camera::Camera,
    entity::collection::EntityCollection,
    light::{
        ambient_light::AmbientLight, area_light::AreaLight, directional_light::DirectionalLight,
        point_light::PointLight, spot_light::SpotLight,
    },
};

use self::{graph::DirectedGraph, error::Error};

use super::webgl::{
    buffer::{BufferDescriptor, BufferStore},
    program::ProgramStore,
    texture::TextureStore,
};

/// Pipeline rendering stuff.
pub trait Stuff {
    /// Returns an entity collection that should be draw on current frame.
    fn entity_collection(&self) -> &EntityCollection;

    /// Returns an mutable entity collection that should be draw on current frame.
    fn entity_collection_mut(&mut self) -> &mut EntityCollection;

    /// Returns the main camera for current frame.
    fn camera(&self) -> &dyn Camera;

    /// Returns the mutable main camera for current frame.
    fn camera_mut(&mut self) -> &mut dyn Camera;

    /// Returns `true` if enable lighting.
    fn lighting_enabled(&self) -> bool;

    /// Returns light attenuations.
    fn light_attenuations(&self) -> Option<Vec3>;

    /// Returns ambient light.
    fn ambient_light(&self) -> Option<&AmbientLight>;

    /// Returns point lights.
    fn point_lights(&self) -> &[PointLight];

    /// Returns directional lights.
    fn directional_lights(&self) -> &[DirectionalLight];

    /// Returns spot lights.
    fn spot_lights(&self) -> &[SpotLight];

    /// Returns area lights.
    fn area_lights(&self) -> &[AreaLight];
}

/// Pipeline rendering state.
pub struct State<'a> {
    timestamp: f64,
    canvas: &'a HtmlCanvasElement,
    gl: &'a WebGl2RenderingContext,
    universal_ubo: &'a BufferDescriptor,
    lights_ubo: &'a BufferDescriptor,
    program_store: &'a mut ProgramStore,
    buffer_store: &'a mut BufferStore,
    texture_store: &'a mut TextureStore,
}

impl<'a> State<'a> {
    /// Constructs a new rendering state.
    pub(crate) fn new(
        timestamp: f64,
        gl: &'a WebGl2RenderingContext,
        canvas: &'a HtmlCanvasElement,
        universal_ubo: &'a BufferDescriptor,
        lights_ubo: &'a BufferDescriptor,
        program_store: &'a mut ProgramStore,
        buffer_store: &'a mut BufferStore,
        texture_store: &'a mut TextureStore,
    ) -> Self {
        Self {
            gl,
            canvas,
            timestamp,
            universal_ubo,
            lights_ubo,
            program_store,
            buffer_store,
            texture_store,
        }
    }

    /// Returns the [`WebGl2RenderingContext`] associated with the canvas.
    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Returns the [`HtmlCanvasElement`] to be drawn to.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Returns the `requestAnimationFrame` timestamp.
    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Returns the [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store(&self) -> &ProgramStore {
        &self.program_store
    }

    /// Returns the mutable [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        &mut self.program_store
    }

    /// Returns the [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    /// Returns the mutable [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }

    /// Returns the [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store(&self) -> &TextureStore {
        &self.texture_store
    }

    /// Returns the mutable [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store_mut(&mut self) -> &mut TextureStore {
        &mut self.texture_store
    }

    /// Returns uniform buffer object for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
    pub fn universal_ubo(&self) -> BufferDescriptor {
        self.universal_ubo.clone()
    }

    /// Returns uniform buffer object for `atoy_Lights`.
    pub fn lights_ubo(&self) -> BufferDescriptor {
        self.lights_ubo.clone()
    }

    /// Resets WebGl state
    fn reset_gl(&self) {
        self.gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
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
        self.gl.read_buffer(WebGl2RenderingContext::NONE);
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

/// A rendering pipeline.
pub trait Pipeline {
    /// Error that could be thrown during execution.
    type Error;

    /// Executes this rendering pipeline with specified [`State`] and rendering [`Stuff`].
    fn execute<S>(&mut self, state: &mut State, stuff: &mut S) -> Result<(), Self::Error>
    where
        S: Stuff;
}

/// An execution node for [`Pipeline`].
pub trait Executor {
    /// Error that could be thrown during execution.
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

/// A standard rendering pipeline container based on [`DirectedGraph`].
pub struct GraphPipeline<E> {
    graph: DirectedGraph<Box<dyn Executor<Error = E>>>,
    executor_keys: HashMap<ItemKey, usize>,
    resources: Resources,
}

impl<E> GraphPipeline<E> {
    /// Constructs a new standard pipeline.
    pub fn new() -> Self {
        Self {
            graph: DirectedGraph::new(),
            executor_keys: HashMap::new(),
            resources: Resources::new(),
        }
    }

    /// Adds a new executor with a [`ItemKey`].
    pub fn add_executor<T>(&mut self, key: ItemKey, executor: T)
    where
        T: Executor<Error = E> + 'static,
    {
        let index = self.graph.add_vertex(Box::new(executor));
        self.executor_keys.insert(key, index);
    }

    /// Removes an executor by a [`ItemKey`].
    pub fn remove_executor(&mut self, key: &ItemKey) {
        if let Some(index) = self.executor_keys.remove(key) {
            self.graph.remove_vertex(index);
            self.executor_keys.iter_mut().for_each(|(_, v)| {
                if *v > index {
                    *v -= 1
                }
            });
        };
    }

    /// Returns an executor by a [`ItemKey`].
    pub fn executor(&self, key: &ItemKey) -> Option<&dyn Executor<Error = E>> {
        self.executor_keys
            .get(key)
            .and_then(|index| self.graph.vertex(*index))
            .map(|vertex| vertex.as_ref())
    }

    pub fn executor_mut(&mut self, key: &ItemKey) -> Option<&mut dyn Executor<Error = E>> {
        let Some(index) = self.executor_keys.get(key) else {
            return None;
        };

        match self.graph.vertex_mut(*index) {
            Some(executor) => Some(executor.as_mut()),
            None => None,
        }
    }

    /// Connects two executors by [`ItemKey`].
    ///
    /// # Errors
    ///
    /// - [`Error::NoSuchExecutor`] thrown if `from` or `to` index does not exist.
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

    /// Disconnects two executors by [`ItemKey`].
    ///
    /// # Errors
    ///
    /// - [`Error::NoSuchExecutor`] thrown if `from` or `to` index does not exist.
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

    /// Returns [`Resources`] associated with this pipeline.
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Returns mutable [`Resources`] associated with this pipeline.
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

impl<E> Pipeline for GraphPipeline<E> {
    type Error = E;

    fn execute<S>(&mut self, state: &mut State, stuff: &mut S) -> Result<(), Self::Error>
    where
        S: Stuff,
    {
        for (_, executor) in self.graph.iter_mut().unwrap() {
            state.reset_gl();

            if executor.before(state, stuff, &mut self.resources)? {
                executor.execute(state, stuff, &mut self.resources)?;
                executor.after(state, stuff, &mut self.resources)?;
            }
        }

        // clears runtime resources
        self.resources.runtime.clear();
        Ok(())
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
    pub fn new_uuid() -> Self {
        Self::Uuid(Uuid::new_v4())
    }

    /// Constructs a new item key by [`String`].
    pub fn new_str<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self::String(name.into())
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
    pub fn new_runtime_str<S>(key: S) -> Self
    where
        S: Into<String>,
    {
        Self::Runtime(ItemKey::new_str(key), PhantomData::<V>)
    }

    /// Constructs a new string persist resource key.
    pub fn new_persist_str<S>(key: S) -> Self
    where
        S: Into<String>,
    {
        Self::Persist(ItemKey::new_str(key), PhantomData::<V>)
    }

    /// Constructs a new runtime resource key with random uuid.
    pub fn new_runtime_uuid() -> Self {
        Self::Runtime(ItemKey::new_uuid(), PhantomData::<V>)
    }

    /// Constructs a new persist resource key with random uuid.
    pub fn new_persist_uuid() -> Self {
        Self::Persist(ItemKey::new_uuid(), PhantomData::<V>)
    }

    /// Returns a raw [`ItemKey`] associated with this resource key.
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
