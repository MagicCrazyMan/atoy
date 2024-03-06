use std::ops::Deref;

pub enum Readonly<'a, T> {
    Borrowed(&'a T),
    Owned(T),
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

pub enum ReadonlyUnsized<'a, T: ?Sized + 'a> {
    Borrowed(&'a T),
    Owned(Box<T>),
}

impl<'a, T: ?Sized + 'a> AsRef<T> for ReadonlyUnsized<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            ReadonlyUnsized::Borrowed(v) => *v,
            ReadonlyUnsized::Owned(v) => v,
        }
    }
}

impl<'a, T: ?Sized + 'a> Deref for ReadonlyUnsized<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadonlyUnsized::Borrowed(v) => *v,
            ReadonlyUnsized::Owned(v) => v,
        }
    }
}
