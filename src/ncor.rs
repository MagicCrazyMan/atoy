pub enum Ncor<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> AsRef<T> for Ncor<'a, T> {
    fn as_ref(&self) -> &T {
        match self {
            Ncor::Borrowed(b) => *b,
            Ncor::Owned(o) => o,
        }
    }
}
