use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use hashbrown::HashMap;
use uuid::Uuid;

pub trait Executor {
    type Message;

    /// Executes code when receive a message.
    fn execute(&mut self, msg: &Self::Message);

    /// Returns `true` if this receiver should be removed.
    fn abort(&self) -> bool {
        false
    }
}

/// A aborter to unregister a [`Executor`] from a message channel.
pub struct Aborter<T> {
    id: Uuid,
    executors: Weak<RefCell<HashMap<Uuid, Box<dyn Executor<Message = T>>>>>,
    off_when_dropped: bool,
}

impl<T> Drop for Aborter<T> {
    fn drop(&mut self) {
        if self.off_when_dropped {
            if let Some(executors) = self.executors.upgrade() {
                executors.borrow_mut().remove(&self.id);
            }
        }
    }
}

impl<T> Aborter<T> {
    /// Returns registered key of the associated [`Aborter`].
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Return `true` if [`Executor`] should be unregistered the from channel when this aborter dropped.
    pub fn off_when_dropped(&self) -> bool {
        self.off_when_dropped
    }

    /// Sets whether unregister the [`Executor`] from channel when this aborter dropped.
    pub fn set_off_when_dropped(&mut self, enabled: bool) {
        self.off_when_dropped = enabled;
    }

    /// Unregisters associated [`Executor`] from message channel.
    pub fn off(self) {
        if let Some(executors) = self.executors.upgrade() {
            executors.borrow_mut().remove(&self.id);
        }
    }
}

/// Sender of message channel.
/// Message can be sent through the channel with [`Sender::send`].
pub struct Sender<T> {
    executors: Rc<RefCell<HashMap<Uuid, Box<dyn Executor<Message = T>>>>>,
    aborts: RefCell<Vec<Uuid>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            executors: Rc::clone(&self.executors),
            aborts: RefCell::new(Vec::new()),
        }
    }
}

impl<T> Sender<T> {
    /// Sends a new message to channel.
    pub fn send(&self, msg: T) {
        let mut executors = self.executors.borrow_mut();
        let mut aborts = self.aborts.borrow_mut();

        for (id, executor) in executors.iter_mut() {
            executor.execute(&msg);
            if executor.abort() {
                aborts.push(*id);
            }
        }

        for abort in aborts.drain(..) {
            executors.remove(&abort);
        }
    }
}

/// Receiver of message channel.
/// Message sent to the channel can be receive by a receiver
/// and then receiver will executes all registered executors.
pub struct Receiver<T> {
    executors: Rc<RefCell<HashMap<Uuid, Box<dyn Executor<Message = T>>>>>,
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            executors: Rc::clone(&self.executors),
        }
    }
}

impl<T> Receiver<T> {
    /// Registers a [`Executor`] to channel.
    pub fn on<E>(&self, executor: E) -> Aborter<T>
    where
        E: Executor<Message = T> + 'static,
    {
        let id = Uuid::new_v4();
        self.executors.borrow_mut().insert(id, Box::new(executor));
        Aborter {
            id,
            executors: Rc::downgrade(&self.executors),
            off_when_dropped: false,
        }
    }
}

/// Constructs a new message channel.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let executors = Rc::new(RefCell::new(HashMap::new()));
    let sender = Sender {
        executors: Rc::clone(&executors),
        aborts: RefCell::new(Vec::new()),
    };
    let receiver = Receiver { executors };

    (sender, receiver)
}
