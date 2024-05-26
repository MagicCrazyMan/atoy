use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

pub trait Command<CTX> {
    fn execute(&mut self, context: CTX);
}

pub struct Commands<CTX>(VecDeque<Box<dyn Command<CTX>>>);

impl<CTX> Commands<CTX> {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
}

impl<CTX> Commands<CTX> {
    pub fn push_front_component<CMD>(&mut self, command: CMD)
    where
        CMD: Command<CTX> + 'static,
    {
        self.0.push_front(Box::new(command));
    }

    pub fn push_back_component<CMD>(&mut self, command: CMD)
    where
        CMD: Command<CTX> + 'static,
    {
        self.0.push_back(Box::new(command));
    }

    pub fn insert_component<CMD>(&mut self, index: usize, command: CMD)
    where
        CMD: Command<CTX> + 'static,
    {
        self.0.insert(index, Box::new(command));
    }
}

impl<CTX> Deref for Commands<CTX> {
    type Target = VecDeque<Box<dyn Command<CTX>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<CTX> DerefMut for Commands<CTX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
