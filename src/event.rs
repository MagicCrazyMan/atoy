use std::{cell::RefCell, rc::Rc};

use uuid::Uuid;

struct Listener<T: ?Sized> {
    id: Uuid,
    execution_count: usize,
    max_execution_count: Option<usize>,
    func: Box<dyn FnMut(&mut T)>,
}

/// A common event listener registration and dispatch agency.
///
/// Registers a listener to `EventTarget` using [`EventTarget::on()`],
/// [`EventTarget::once()`] or [`EventTarget::on_count()`] methods and removes a listener using [`EventTarget::off()`].
/// Invokes [`EventTarget::raise()`] for raising an event.
pub struct EventAgency<T: ?Sized>(Rc<RefCell<Vec<Listener<T>>>>);

impl<T> EventAgency<T> {
    /// Constructs a new event target agency.
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Vec::new())))
    }

    /// Adds a listener to event target.
    pub fn on<F>(&self, f: F) -> Uuid
    where
        F: FnMut(&mut T) + 'static,
    {
        let id = Uuid::new_v4();
        self.0.borrow_mut().push(Listener {
            id,
            execution_count: 0,
            max_execution_count: None,
            func: Box::new(f),
        });
        id
    }

    /// Adds a listener to event target and execute it until it reaches the specified execution count.
    pub fn on_count<F>(&self, f: F, count: usize) -> Uuid
    where
        F: FnMut(&mut T) + 'static,
    {
        let id = Uuid::new_v4();
        self.0.borrow_mut().push(Listener {
            id,
            execution_count: 0,
            max_execution_count: Some(count),
            func: Box::new(f),
        });
        id
    }

    /// Adds a listener to event target and execute it only once.
    pub fn once<F>(&self, f: F) -> Uuid
    where
        F: FnMut(&mut T) + 'static,
    {
        let id = Uuid::new_v4();
        self.0.borrow_mut().push(Listener {
            id,
            execution_count: 0,
            max_execution_count: Some(1),
            func: Box::new(f),
        });
        id
    }

    /// Removes a listener with a specified [`Uuid`] from event target.
    pub fn off(&self, id: &Uuid) {
        let Some(index) = self
            .0
            .borrow_mut()
            .iter()
            .position(|listener| &listener.id == id)
        else {
            return;
        };
        self.0.borrow_mut().remove(index);
    }

    /// Raises an event, notifies and invokes all registered listeners.
    pub fn raise(&self, mut event: T) {
        let mut listeners = self.0.borrow_mut();
        let mut len = listeners.len();
        let mut i = 0;
        while i < len {
            let listener = listeners.get_mut(i).unwrap();
            let func = listener.func.as_mut();
            func(&mut event);

            listener.execution_count += 1;

            if let Some(max_count) = listener.max_execution_count {
                // removes listener if it reaches the max executions.
                if listener.execution_count >= max_count {
                    listeners.remove(i);
                    len -= 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

impl<T> Clone for EventAgency<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
