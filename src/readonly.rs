use std::ops::Deref;

pub enum Readonly<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T: Clone> Clone for Readonly<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(v) => Self::Borrowed(*v),
            Self::Owned(v) => Self::Owned(v.clone()),
        }
    }
}

impl<'a, T> AsRef<T> for Readonly<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Readonly::Borrowed(v) => *v,
            Readonly::Owned(v) => v,
        }
    }
}

impl<'a, T> Deref for Readonly<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Readonly::Borrowed(v) => *v,
            Readonly::Owned(v) => v,
        }
    }
}
