use std::{
    cell::{Ref, RefCell, RefMut},
    collections::VecDeque,
    rc::Rc,
};

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Scheduler {
    async fn start(&mut self);

    async fn stop(&mut self);

    fn running(&self) -> bool;

    fn tasks(&self) -> &SchedulingTasks;
}

#[derive(Clone)]
pub struct SchedulingTasks {
    tasks: Rc<RefCell<VecDeque<Box<dyn Task>>>>,
}

impl SchedulingTasks {
    pub fn new() -> Self {
        Self {
            tasks: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn raw(&self) -> Ref<VecDeque<Box<dyn Task>>> {
        self.tasks.borrow()
    }

    pub fn raw_mut(&self) -> RefMut<VecDeque<Box<dyn Task>>> {
        self.tasks.borrow_mut()
    }

    pub fn push_back<T>(&self, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.borrow_mut().push_back(Box::new(task));
    }

    pub fn push_front<T>(&self, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.borrow_mut().push_front(Box::new(task));
    }

    pub fn insert<T>(&self, index: usize, task: T)
    where
        T: Task + 'static,
    {
        self.tasks.borrow_mut().insert(index, Box::new(task));
    }

    pub fn pop_back(&self) -> Option<Box<dyn Task>> {
        self.tasks.borrow_mut().pop_back()
    }

    pub fn pop_front(&self) -> Option<Box<dyn Task>> {
        self.tasks.borrow_mut().pop_front()
    }
}

pub trait Task {
    fn exec(&mut self, tasks: &SchedulingTasks);
}
