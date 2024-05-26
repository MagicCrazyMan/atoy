use std::{collections::VecDeque, ops::{Deref, DerefMut}};

use super::{
    channel::MessageChannel, clock::Clock, engine::RenderEngine, resource::Resources, scene::Scene,
};

pub struct Context<'a, CLK, RE> {
    pub scene: &'a mut Scene,
    pub clock: &'a mut CLK,
    pub engine: &'a mut RE,
    pub channel: &'a MessageChannel,
    pub resources: &'a mut Resources,
    pub temp_resources: &'a mut Resources,

    pub current_commands: &'a mut Commands<CLK, RE>,
    pub next_commands: &'a mut Commands<CLK, RE>,
}

pub trait Command {
    type Clock: Clock;

    type RenderEngine: RenderEngine;

    fn execute(&mut self, context: Context<'_, Self::Clock, Self::RenderEngine>);
}

pub struct Commands<CLK, RE>(VecDeque<Box<dyn Command<Clock = CLK, RenderEngine = RE>>>);

impl<Clock, RenderEngine> Commands<Clock, RenderEngine> {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
}

impl<CLK, RE> Commands<CLK, RE>
where
    CLK: Clock,
    RE: RenderEngine,
{
    pub fn push_front_component<CMD>(&mut self, command: CMD)
    where
        CMD: Command<Clock = CLK, RenderEngine = RE> + 'static,
    {
        self.0.push_front(Box::new(command));
    }

    pub fn push_back_component<CMD>(&mut self, command: CMD)
    where
        CMD: Command<Clock = CLK, RenderEngine = RE> + 'static,
    {
        self.0.push_back(Box::new(command));
    }

    pub fn insert_component<CMD>(&mut self, index: usize, command: CMD)
    where
        CMD: Command<Clock = CLK, RenderEngine = RE> + 'static,
    {
        self.0.insert(index, Box::new(command));
    }
}

impl<CLK, RE> Deref for Commands<CLK, RE> {
    type Target = VecDeque<Box<dyn Command<Clock = CLK, RenderEngine = RE>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<CLK, RE> DerefMut for Commands<CLK, RE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
