pub mod error;
pub mod graph;

use std::{
    any::Any,
    collections::{HashMap, VecDeque},
};

use web_sys::WebGl2RenderingContext;

use self::{error::Error, graph::DirectedGraph};

#[derive(Clone)]
pub struct State {
    gl: WebGl2RenderingContext,
    timestamp: f64,
}

pub struct Pipeline {
    graph: DirectedGraph<Box<dyn Executor>>,
    name_to_index: HashMap<String, usize>,
    runtime_data: HashMap<usize, Box<dyn Any>>,
}

impl Pipeline {
    pub fn new() -> Self {
        let mut instance = Self {
            graph: DirectedGraph::new(),
            name_to_index: HashMap::new(),
            runtime_data: HashMap::new(),
        };
        instance.add_executor(StartExecutor);
        instance
    }

    pub fn execute(&mut self, state: &State) -> Result<(), super::webgl::error::Error> {
        let mut executors = VecDeque::from([self.graph.vertex_mut(0).unwrap()]);
        while let Some(executor) = executors.pop_front() {
            executor.execute(state)?;
        }

        self.clear();

        Ok(())
    }

    fn clear(&mut self) {
        self.runtime_data.clear();
    }

    pub fn add_executor<E: Executor + 'static>(&mut self, executor: E) {
        let name = executor.name().to_string();
        let index = self.graph.add_vertex(Box::new(executor));
        self.name_to_index.insert(name, index);
    }

    pub fn remove_executor(&mut self, name: &str) -> Result<(), Error> {
        let Some(index) = self.name_to_index.remove(name) else {
            return Err(Error::NoSuchExecutor(name.to_string()));
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
            return Err(Error::NoSuchExecutor(name.to_string()));
        };

        match self.graph.vertex(*index) {
            Some(executor) => Ok(Some(executor.as_ref())),
            None => Ok(None),
        }
    }

    pub fn executor_mut(&mut self, name: &str) -> Result<Option<&mut dyn Executor>, Error> {
        let Some(index) = self.name_to_index.get(name) else {
            return Err(Error::NoSuchExecutor(name.to_string()));
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
            .ok_or(Error::NoSuchExecutor(from.to_string()))?;
        let to_index = self
            .name_to_index
            .get(to)
            .ok_or(Error::NoSuchExecutor(to.to_string()))?;

        self.graph.add_arc(*from_index, *to_index)?;

        Ok(())
    }

    pub fn disconnect(&mut self, from: &str, to: &str) -> Result<(), Error> {
        let from_index = self
            .name_to_index
            .get(from)
            .ok_or(Error::NoSuchExecutor(from.to_string()))?;
        let to_index = self
            .name_to_index
            .get(to)
            .ok_or(Error::NoSuchExecutor(to.to_string()))?;

        self.graph.remove_arc(*from_index, *to_index);

        Ok(())
    }
}

struct RuntimeData {
    data: Box<dyn Any>,
}

pub const START_EXECUTOR_NAME: &'static str = "__StartExecutor__";

/// An start executor doing nothing.
/// `StartExecutor` is always placed as the first vertex of the graph.
struct StartExecutor;

impl Executor for StartExecutor {
    #[inline]
    fn name(&self) -> &str {
        START_EXECUTOR_NAME
    }

    #[inline]
    fn execute(&mut self, _: &State) -> Result<(), super::webgl::error::Error> {
        Ok(())
    }
}

pub trait Executor {
    fn name(&self) -> &str;

    fn execute(&mut self, state: &State) -> Result<(), super::webgl::error::Error>;
}
