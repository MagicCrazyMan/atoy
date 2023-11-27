use std::any::Any;

pub enum Ncor<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

pub enum BoxedNcor<'a> {
    Borrowed(&'a dyn Any),
    Owned(Box<dyn Any>),
}
