use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use hashbrown::HashMap;

pub struct Notifying<T> {
    key: usize,
    notifiee: *mut dyn Notifiee<T>,
    inner: Weak<RefCell<Inner<T>>>,
}

impl<T> Notifying<T> {
    pub fn key(&self) -> usize {
        self.key
    }

    pub fn unregister(self) {
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        let Some(notifiee) = inner
            .borrow()
            .notifiees
            .get(&self.key)
            .map(|notifiee| *notifiee)
        else {
            return;
        };

        if notifiee == self.notifiee {
            let Some(notifiee) = inner.borrow_mut().notifiees.remove(&self.key) else {
                return;
            };
            unsafe { drop(Box::from_raw(notifiee)) }
        }
    }
}

struct Inner<T> {
    counter: usize,
    notifiees: HashMap<usize, *mut dyn Notifiee<T>>,
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        unsafe {
            for (_, notifiee) in self.notifiees.drain() {
                drop(Box::from_raw(notifiee));
            }
        }
    }
}

impl<T> Inner<T> {
    fn next(&mut self) -> usize {
        if self.notifiees.len() == usize::MAX {
            panic!("too many notifiees, only {} are accepted", usize::MAX);
        }

        self.counter = self.counter.wrapping_add(1);
        while self.notifiees.contains_key(&self.counter) {
            self.counter = self.counter.wrapping_add(1);
        }
        self.counter
    }
}

pub struct Notifier<T> {
    inner: Rc<RefCell<Inner<T>>>,
    aborts: Rc<RefCell<Vec<usize>>>,
}

impl<T> Clone for Notifier<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            aborts: Rc::clone(&self.aborts),
        }
    }
}

impl<T> Notifier<T> {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                counter: 0,
                notifiees: HashMap::new(),
            })),
            aborts: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn register<N>(&self, notifiee: N) -> Notifying<T>
    where
        N: Notifiee<T> + 'static,
    {
        let mut inner = self.inner.borrow_mut();

        let key = inner.next();
        let notifiee = Box::leak(Box::new(notifiee));
        inner.notifiees.insert(key, notifiee);
        Notifying {
            key,
            notifiee,
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub fn unregister(&self, key: usize) {
        let Some(notifiee) = self.inner.borrow_mut().notifiees.remove(&key) else {
            return;
        };
        unsafe { drop(Box::from_raw(notifiee)) }
    }

    pub fn notify(&self, msg: &T) {
        unsafe {
            let mut inner = self.inner.borrow_mut();
            let mut aborts = self.aborts.borrow_mut();

            for (key, notifiee) in inner.notifiees.iter_mut() {
                let notifiee = &mut **notifiee;
                notifiee.notify(msg);
                if notifiee.abort() {
                    aborts.push(*key);
                }
            }

            for abort in aborts.drain(..) {
                let Some(notifiee) = inner.notifiees.remove(&abort) else {
                    continue;
                };
                drop(Box::from_raw(notifiee));
            }
        }
    }
}

pub trait Notifiee<T> {
    fn notify(&mut self, msg: &T);

    fn abort(&self) -> bool {
        false
    }
}
