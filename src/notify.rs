use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;

use crate::share::{Share, WeakShare};

pub struct Notifying<T> {
    key: usize,
    notifiees: WeakShare<HashMap<usize, *mut dyn Notifiee<T>>>,
}

impl<T> Notifying<T> {
    pub fn key(&self) -> usize {
        self.key
    }

    pub fn unregister(self) {
        let Some(notifiees) = self.notifiees.upgrade() else {
            return;
        };
        let Some(notifiee) = notifiees.borrow_mut().remove(&self.key) else {
            return;
        };

        unsafe { drop(Box::from_raw(notifiee)) }
    }
}

pub struct Notifier<T> {
    counter: usize,
    notifiees: Share<HashMap<usize, *mut dyn Notifiee<T>>>,
    aborts: Vec<usize>,
}

impl<T> Drop for Notifier<T> {
    fn drop(&mut self) {
        unsafe {
            let mut notifiees = self.notifiees.borrow_mut();
            notifiees
                .iter_mut()
                .for_each(|(_, notifiee)| drop(Box::from_raw(notifiee)));
        }
    }
}

impl<T> Notifier<T> {
    pub fn new() -> Self {
        Self {
            counter: 0,
            notifiees: Rc::new(RefCell::new(HashMap::new())),
            aborts: Vec::new(),
        }
    }

    fn next(&mut self) -> usize {
        let notifiees = self.notifiees.borrow();
        if notifiees.len() == usize::MAX {
            panic!("too many notifiees, only {} are accepted", usize::MAX);
        }

        self.counter = self.counter.wrapping_add(1);
        while notifiees.contains_key(&self.counter) {
            self.counter = self.counter.wrapping_add(1);
        }
        self.counter
    }

    pub fn register<N>(&mut self, notifiee: N) -> Notifying<T>
    where
        N: Notifiee<T> + 'static,
    {
        let key = self.next();
        let notifiee = Box::leak(Box::new(notifiee));
        self.notifiees.borrow_mut().insert(key, notifiee);
        Notifying {
            key,
            notifiees: Rc::downgrade(&self.notifiees),
        }
    }

    pub fn unregister(&mut self, key: usize) {
        let Some(notifiee) = self.notifiees.borrow_mut().remove(&key) else {
            return;
        };
        unsafe {
            drop(Box::from_raw(notifiee));
        }
    }

    pub fn notify(&mut self, msg: &T) {
        unsafe {
            let mut notifiees = self.notifiees.borrow_mut();

            for (key, notifiee) in notifiees.iter_mut() {
                let notifiee = &mut **notifiee;
                notifiee.notify(msg);
                if notifiee.abort() {
                    self.aborts.push(*key);
                }
            }

            for abort in self.aborts.drain(..) {
                let Some(notifiee) = notifiees.remove(&abort) else {
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
