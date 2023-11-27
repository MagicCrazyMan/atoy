use std::any::Any;

pub enum Ncor<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}
