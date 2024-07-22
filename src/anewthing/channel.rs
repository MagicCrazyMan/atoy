use std::{
    any::{Any, TypeId},
    cell::RefCell,
    rc::Rc,
};

use hashbrown::HashMap;

type Handlers = HashMap<TypeId, HashMap<TypeId, Box<dyn Any>>>;

#[derive(Clone)]
pub struct Channel {
    handlers: Rc<RefCell<Handlers>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            handlers: Rc::new(RefCell::new(HashMap::new())),
        }
    }

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

    pub fn send<M>(&self, msg: M)
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
            let mut ctx = Context::new();
            handler.as_mut().handle(&msg, &mut ctx);

            if ctx.aborted {
                aborted.push(*handler_type_id);
            }

            if ctx.terminated {
                break;
            }
        }

        for handler_type_id in aborted {
            handlers.remove(&handler_type_id);
        }
    }
}

pub struct Context {
    aborted: bool,
    terminated: bool,
}

impl Context {
    fn new() -> Self {
        Self {
            aborted: false,
            terminated: false,
        }
    }

    pub fn abort(&mut self) {
        self.aborted = true;
    }

    pub fn terminate(&mut self) {
        self.terminated = true;
    }
}

pub trait Handler<M> {
    fn handle(&mut self, msg: &M, ctx: &mut Context);
}
