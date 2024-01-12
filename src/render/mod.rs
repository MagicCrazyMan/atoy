use std::{collections::{HashMap, hash_map::Entry}, any::Any, marker::PhantomData};

use uuid::Uuid;

use crate::{camera::Camera, scene::Scene};

use self::{graph::DirectedGraph, error::Error};

pub mod webgl;
pub mod graph;
pub mod error;

pub trait Render {
    type State;

    type Error;

    fn render(
        &mut self,
        pipeline: &mut (dyn Pipeline<State = Self::State, Error = Self::Error> + 'static),
        camera: &mut (dyn Camera + 'static),
        scene: &mut Scene,
        timestamp: f64,
    ) -> Result<(), Self::Error>;
}

/// A rendering pipeline.
pub trait Pipeline {
    /// Runtime state.
    type State;

    /// Error that could be thrown during execution.
    type Error;

    /// Executes this rendering pipeline with specified [`State`] and rendering [`Stuff`].
    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error>;
}

/// An execution node for [`Pipeline`].
pub trait Executor {
    /// Runtime state.
    type State;

    /// Error that could be thrown during execution.
    type Error;

    /// Actions before execution.
    /// Developer could setup WebGL state here, or return a `false` to skip execution.
    #[allow(unused)]
    fn before(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }

    /// Main execution procedure.
    fn execute(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error>;

    /// Actions after execution.
    /// Developer should reset WebGL state here to prevent unexpected side effect to other executors.
    #[allow(unused)]
    fn after(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A standard rendering pipeline container based on [`DirectedGraph`].
pub struct GraphPipeline<S, E> {
    graph: DirectedGraph<Box<dyn Executor<State = S, Error = E>>>,
    executor_keys: HashMap<ItemKey, usize>,
    resources: Resources,
}

impl<S, E> GraphPipeline<S, E> {
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
        T: Executor<State = S, Error = E> + 'static,
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
    pub fn executor(&self, key: &ItemKey) -> Option<&dyn Executor<State = S, Error = E>> {
        self.executor_keys
            .get(key)
            .and_then(|index| self.graph.vertex(*index))
            .map(|vertex| vertex.as_ref())
    }

    pub fn executor_mut(
        &mut self,
        key: &ItemKey,
    ) -> Option<&mut dyn Executor<State = S, Error = E>> {
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

impl<S, E> Pipeline for GraphPipeline<S, E> {
    type State = S;

    type Error = E;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
        for (_, executor) in self.graph.iter_mut().unwrap() {
            if executor.before(state, scene, &mut self.resources)? {
                executor.execute(state, scene, &mut self.resources)?;
                executor.after(state, scene, &mut self.resources)?;
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
    pub fn contains_resource<V: 'static>(&mut self, key: &ResourceKey<V>) -> bool {
        self.get(key).is_some()
    }

    /// Returns `true` if the resources contains a value for the specified [`ResourceKey`].
    pub fn contains_resource_unchecked<V>(&mut self, key: &ResourceKey<V>) -> bool {
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
