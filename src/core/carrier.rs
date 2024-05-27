use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;
use uuid::Uuid;

type Listeners<D> = Rc<RefCell<HashMap<Uuid, Box<dyn Listener<D>>>>>;

pub struct Carrier<D>
where
    D: ?Sized,
{
    listeners: Listeners<D>,
}

impl<D> Clone for Carrier<D> {
    fn clone(&self) -> Self {
        Self {
            listeners: Rc::clone(&self.listeners),
        }
    }
}

impl<D> Carrier<D>
where
    D: ?Sized,
{
    pub fn new() -> Self {
        Self {
            listeners: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn send(&self, payload: &mut D) {
        let mut listeners = self.listeners.borrow_mut();

        let mut removed = Vec::new();
        for (id, listener) in listeners.iter_mut() {
            listener.execute(payload);
            if listener.abort() {
                removed.push(*id);
            }
        }

        for id in removed {
            listeners.remove(&id);
        }
    }

    pub fn register<L>(&self, listener: L) -> Unregister<D>
    where
        L: Listener<D> + 'static,
    {
        let id = uuid::Uuid::new_v4();
        self.listeners.borrow_mut().insert(id, Box::new(listener));

        Unregister::new(id, Rc::clone(&self.listeners))
    }
}

/// Message receiver receiving data from channel under specified message kind.
pub trait Listener<D>
where
    D: ?Sized,
{
    /// Executes code when receive a message.
    fn execute(&mut self, payload: &mut D);

    /// Returns `true` if this receiver should abort.
    fn abort(&self) -> bool {
        false
    }
}

/// Message unregister removing a receiver from the channel.
pub struct Unregister<D>
where
    D: ?Sized,
{
    id: Uuid,
    listeners: Listeners<D>,
}

impl<D> Unregister<D>
where
    D: ?Sized,
{
    fn new(id: Uuid, listeners: Listeners<D>) -> Self {
        Self { id, listeners }
    }

    /// Removes the associated receiver from the channel.
    pub fn unregister(self) {
        self.listeners.borrow_mut().remove(&self.id);
    }
}
