pub mod error;
pub mod graph;
pub mod picking;
pub mod standard;
pub mod outlining;

use std::{any::Any, collections::HashMap};

use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{camera::Camera, entity::collection::EntityCollection};

use self::graph::DirectedGraph;

use super::webgl::{
    buffer::BufferStore, error::Error, program::ProgramStore, texture::TextureStore,
};

pub trait Stuff {
    /// Gets entity collection that should be draw on current frame.
    fn entity_collection(&self) -> &EntityCollection;

    /// Gets mutable entity collection that should be draw on current frame.
    fn entity_collection_mut(&mut self) -> &mut EntityCollection;

    /// Gets the main camera for current frame.
    fn camera(&self) -> &dyn Camera;

    /// Gets mutable the main camera for current frame.
    fn camera_mut(&mut self) -> &mut dyn Camera;
}

pub struct State<'a> {
    program_store: &'a mut ProgramStore,
    buffer_store: &'a mut BufferStore,
    texture_store: &'a mut TextureStore,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    timestamp: f64,
}

impl<'a> State<'a> {
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

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn program_store(&self) -> &ProgramStore {
        &self.program_store
    }

    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        &mut self.program_store
    }

    pub fn buffer_store(&self) -> &&'a mut BufferStore {
        &self.buffer_store
    }

    pub fn buffer_store_mut(&mut self) -> &mut &'a mut BufferStore {
        &mut self.buffer_store
    }

    pub fn texture_store(&self) -> &&'a mut TextureStore {
        &self.texture_store
    }

    pub fn texture_store_mut(&mut self) -> &mut &'a mut TextureStore {
        &mut self.texture_store
    }
}

pub struct Pipeline {
    graph: DirectedGraph<Box<dyn Executor>>,
    name_to_index: HashMap<String, usize>,
    runtime_resources: HashMap<String, Box<dyn Any>>,
    persist_resources: HashMap<String, Box<dyn Any>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            graph: DirectedGraph::new(),
            name_to_index: HashMap::new(),
            runtime_resources: HashMap::new(),
            persist_resources: HashMap::new(),
        }
    }

    pub fn execute<S: Stuff>(&mut self, state: &mut State, stuff: &mut S) -> Result<(), Error> {
        for (_, executor) in self.graph.iter_mut()? {
            executor.execute(
                state,
                stuff,
                &mut self.runtime_resources,
                &mut self.persist_resources,
            )?;
        }

        // clears runtime resources
        self.runtime_resources.clear();

        Ok(())
    }

    pub fn add_executor<E: Executor + 'static>(&mut self, name: impl Into<String>, executor: E) {
        let name = name.into();
        let index = self.graph.add_vertex(Box::new(executor));
        self.name_to_index.insert(name, index);
    }

    pub fn remove_executor(&mut self, name: &str) -> Result<(), Error> {
        let Some(index) = self.name_to_index.remove(name) else {
            return Err(self::error::Error::NoSuchExecutor(name.to_string()))?;
        };
        self.graph.remove_vertex(index);
        self.name_to_index.iter_mut().for_each(|(_, v)| {
            if *v > index {
                *v -= 1
            }
        });

        Ok(())
    }

    pub fn executor(&self, name: &str) -> Result<Option<&dyn Executor>, Error> {
        let Some(index) = self.name_to_index.get(name) else {
            return Err(self::error::Error::NoSuchExecutor(name.to_string()))?;
        };

        match self.graph.vertex(*index) {
            Some(executor) => Ok(Some(executor.as_ref())),
            None => Ok(None),
        }
    }

    pub fn executor_mut(&mut self, name: &str) -> Result<Option<&mut dyn Executor>, Error> {
        let Some(index) = self.name_to_index.get(name) else {
            return Err(self::error::Error::NoSuchExecutor(name.to_string()))?;
        };

        match self.graph.vertex_mut(*index) {
            Some(executor) => Ok(Some(executor.as_mut())),
            None => Ok(None),
        }
    }

    pub fn connect(&mut self, from: &str, to: &str) -> Result<(), Error> {
        let from_index = self
            .name_to_index
            .get(from)
            .ok_or(self::error::Error::NoSuchExecutor(from.to_string()))?;
        let to_index = self
            .name_to_index
            .get(to)
            .ok_or(self::error::Error::NoSuchExecutor(to.to_string()))?;

        self.graph.add_arc(*from_index, *to_index)?;

        Ok(())
    }

    pub fn disconnect(&mut self, from: &str, to: &str) -> Result<(), Error> {
        let from_index = self
            .name_to_index
            .get(from)
            .ok_or(self::error::Error::NoSuchExecutor(from.to_string()))?;
        let to_index = self
            .name_to_index
            .get(to)
            .ok_or(self::error::Error::NoSuchExecutor(to.to_string()))?;

        self.graph.remove_arc(*from_index, *to_index);

        Ok(())
    }

    pub fn persist_resources(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.persist_resources
    }

    pub fn persist_resources_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.persist_resources
    }

    pub fn runtime_resources(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.runtime_resources
    }

    pub fn runtime_resources_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.runtime_resources
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceSource {
    Runtime(String),
    Persist(String),
}

impl ResourceSource {
    pub fn runtime(key: impl Into<String>) -> Self {
        Self::Runtime(key.into())
    }

    pub fn persist(key: impl Into<String>) -> Self {
        Self::Persist(key.into())
    }
}

pub trait Executor {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn Any>>,
        persist_resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error>;
}
