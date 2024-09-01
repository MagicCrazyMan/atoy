use std::{
    any::{Any, TypeId},
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use hashbrown::HashMap;
use uuid::Uuid;

type Handlers = HashMap<TypeId, HashMap<TypeId, Box<dyn Any>>>;

#[derive(Clone)]
pub struct Channel {
    id: Uuid,
    handlers: Rc<RefCell<Handlers>>,
}

impl Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Channel").field("id", &self.id).finish()
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Channel {}

impl Hash for Channel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Channel {
    /// Constructs a new message channel.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            handlers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Returns the id of the channel.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Registers a handler for a message type.
    pub fn on<M, H>(&self, handler: H) -> bool
    where
        M: 'static,
        H: Handler<M> + 'static,
    {
        let msg_type_id = TypeId::of::<M>();
        let handler_type_id = TypeId::of::<H>();

        let mut handlers = self.handlers.borrow_mut();
        let handlers = handlers
            .entry(msg_type_id)
            .or_insert_with(|| HashMap::new());
        if handlers.contains_key(&handler_type_id) {
            return false;
        }

        handlers.insert_unique_unchecked(
            handler_type_id,
            Box::new(Box::new(handler) as Box<dyn Handler<M>>),
        );

        true
    }

    /// Unregisters a handler for a message type.
    pub fn off<M, H>(&self)
    where
        M: 'static,
        H: Handler<M> + 'static,
    {
        let msg_type_id = TypeId::of::<M>();
        let handler_type_id = TypeId::of::<H>();

        let mut handlers = self.handlers.borrow_mut();
        let Some(handlers) = handlers.get_mut(&msg_type_id) else {
            return;
        };
        handlers.remove(&handler_type_id);
    }

    /// Sends a message to the channel, and invokes all associated handlers.
    pub fn send<M>(&self, mut msg: M)
    where
        M: 'static,
    {
        let msg_type_id = TypeId::of::<M>();

        let mut handlers = self.handlers.borrow_mut();
        let Some(handlers) = handlers.get_mut(&msg_type_id) else {
            return;
        };

        let mut aborted = Vec::new();
        for (handler_type_id, handler) in handlers.iter_mut() {
            let handler = handler.downcast_mut::<Box<dyn Handler<M>>>().unwrap();
            let mut event = Event::new(&mut msg);
            handler.as_mut().handle(&mut event);

            if event.aborted {
                aborted.push(*handler_type_id);
            }

            if event.terminated {
                break;
            }
        }

        for handler_type_id in aborted {
            handlers.remove(&handler_type_id);
        }
    }
}

pub struct Event<'a, M> {
    message: &'a mut M,
    aborted: bool,
    terminated: bool,
}

impl<'a, M> Event<'a, M> {
    fn new(message: &'a mut M) -> Self {
        Self {
            message,
            aborted: false,
            terminated: false,
        }
    }

    /// Returns the immutable message.
    pub fn message(&self) -> &M {
        self.message
    }

    /// Returns the mutable message.
    pub fn message_mut(&mut self) -> &mut M {
        self.message
    }

    /// Removes the handler after finishing invoking.
    pub fn abort(&mut self) {
        self.aborted = true;
    }

    /// Terminates message propagation.
    /// Ignores all remaining handlers after finishing invoking this handler.
    pub fn terminate(&mut self) {
        self.terminated = true;
    }
}

impl<'a, M> Deref for Event<'a, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        self.message
    }
}

impl<'a, M> DerefMut for Event<'a, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.message
    }
}

pub trait Handler<M> {
    fn handle<'a>(&'a mut self, msg: &mut Event<'_, M>);
}
