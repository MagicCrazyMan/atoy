use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type Share<T> = Rc<RefCell<T>>;

pub type WeakShare<T> = Weak<RefCell<T>>;
