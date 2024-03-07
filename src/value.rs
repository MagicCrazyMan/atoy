use std::{
    cell::{Ref, RefCell, RefMut},
    cmp::Ordering,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub enum Readonly<'a, T> {
    Owned(T),
    Borrowed(&'a T),
    Ref(Ref<'a, T>),
}

impl<'a, T> Readonly<'a, T> {
    pub fn value(&self) -> &T {
        match self {
            Readonly::Owned(v) => v,
            Readonly::Borrowed(v) => *v,
            Readonly::Ref(v) => &**v,
        }
    }
}

impl<'a, T> AsRef<T> for Readonly<'a, T> {
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<'a, T> Deref for Readonly<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<'a, T: PartialEq> PartialEq for Readonly<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(other.value())
    }
}

impl<'a, T: Eq> Eq for Readonly<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for Readonly<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<'a, T: Ord> Ord for Readonly<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl<'a, T: Display> Display for Readonly<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.value(), f)
    }
}

impl<'a, T: Debug> Debug for Readonly<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.value(), f)
    }
}

pub enum Writable<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
    RefMut(RefMut<'a, T>),
}

impl<'a, T> Writable<'a, T> {
    pub fn value(&self) -> &T {
        match self {
            Writable::Owned(v) => v,
            Writable::Borrowed(v) => *v,
            Writable::RefMut(v) => &**v,
        }
    }

    pub fn value_mut(&mut self) -> &mut T {
        match self {
            Writable::Owned(v) => v,
            Writable::Borrowed(v) => *v,
            Writable::RefMut(v) => &mut **v,
        }
    }
}

impl<'a, T> AsRef<T> for Writable<'a, T> {
    fn as_ref(&self) -> &T {
        self.value()
    }
}

impl<'a, T> AsMut<T> for Writable<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.value_mut()
    }
}

impl<'a, T> Deref for Writable<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl<'a, T> DerefMut for Writable<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}

impl<'a, T: PartialEq> PartialEq for Writable<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(other.value())
    }
}

impl<'a, T: Eq> Eq for Writable<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for Writable<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<'a, T: Ord> Ord for Writable<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(other.value())
    }
}

impl<'a, T: Display> Display for Writable<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.value(), f)
    }
}

impl<'a, T: Debug> Debug for Writable<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.value(), f)
    }
}

// pub enum ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned,
// {
//     Owned(<T as ToOwned>::Owned),
//     Borrowed(&'a T),
//     Ref(Ref<'a, T>),
// }

// impl<'a, T> ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned,
//     T::Owned: Borrow<T>,
// {
//     pub fn value(&self) -> &T {
//         match &self {
//             ValueUnsized::Owned(ref v) => v.borrow(),
//             ValueUnsized::Borrowed(v) => *v,
//             ValueUnsized::Ref(v) => &**v,
//         }
//     }
// }

// impl<'a, T> AsRef<T> for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned,
//     T::Owned: Borrow<T>,
// {
//     fn as_ref(&self) -> &T {
//         self.value()
//     }
// }

// impl<'a, T> Deref for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned,
//     T::Owned: Borrow<T>,
// {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         self.value()
//     }
// }

// impl<'a, 'b, T, U> PartialEq<ValueUnsized<'b, U>> for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned + PartialEq<U>,
//     U: ?Sized + ToOwned,
// {
//     fn eq(&self, other: &ValueUnsized<'b, U>) -> bool {
//         self.value().eq(other.value())
//     }
// }

// impl<'a, T> Eq for ValueUnsized<'a, T> where T: ?Sized + ToOwned + Eq {}

// impl<'a, T> PartialOrd for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned + PartialOrd,
// {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         self.value().partial_cmp(other.value())
//     }
// }

// impl<'a, T> Ord for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned + Ord,
// {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.value().cmp(other.value())
//     }
// }

// impl<'a, T> Display for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned + Display,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Display::fmt(self.value(), f)
//     }
// }

// impl<'a, T> Debug for ValueUnsized<'a, T>
// where
//     T: ?Sized + ToOwned + Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Debug::fmt(self.value(), f)
//     }
// }

// pub enum ValueMut<'a, T> {
//     Owned(T),
//     Borrowed(&'a mut T),
//     RefMut(RefMut<'a, T>),
// }

// impl<'a, T> ValueMut<'a, T> {
//     pub fn value(&self) -> &T {
//         match &self {
//             ValueMut::Owned(v) => v,
//             ValueMut::Borrowed(v) => *v,
//             ValueMut::RefMut(v) => &**v,
//         }
//     }

//     pub fn value_mut(&mut self) -> &mut T {
//         match self {
//             ValueMut::Owned(v) => v,
//             ValueMut::Borrowed(v) => *v,
//             ValueMut::RefMut(v) => &mut **v,
//         }
//     }
// }

// impl<'a, T> AsRef<T> for ValueMut<'a, T> {
//     fn as_ref(&self) -> &T {
//         self.value()
//     }
// }

// impl<'a, T> Deref for ValueMut<'a, T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         self.value()
//     }
// }

// impl<'a, T> AsMut<T> for ValueMut<'a, T> {
//     fn as_mut(&mut self) -> &mut T {
//         self.value_mut()
//     }
// }

// impl<'a, T> DerefMut for ValueMut<'a, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.value_mut()
//     }
// }

// pub enum ValueMutUnsized<'a, T: ?Sized + ToOwned> {
//     Owned(<T as ToOwned>::Owned),
//     Borrowed(&'a mut T),
//     RefMut(RefMut<'a, T>),
// }

// impl<'a, T: ?Sized + ToOwned> ValueMutUnsized<'a, T>
// where
//     T::Owned: Borrow<T>,
// {
//     pub fn value(&self) -> &T {
//         match &self {
//             ValueMutUnsized::Owned(ref v) => v.borrow(),
//             ValueMutUnsized::Borrowed(v) => *v,
//             ValueMutUnsized::RefMut(v) => &**v,
//         }
//     }

//     pub fn value_mut(&mut self) -> &mut T {
//         match self {
//             ValueMutUnsized::Owned(ref mut v) => v.borrow_mut(),
//             ValueMutUnsized::Borrowed(v) => *v,
//             ValueMutUnsized::RefMut(v) => &mut **v,
//         }
//     }
// }

// impl<'a, T: ?Sized + ToOwned> AsRef<T> for ValueMutUnsized<'a, T>
// where
//     T::Owned: Borrow<T>,
// {
//     fn as_ref(&self) -> &T {
//         self.value()
//     }
// }

// impl<'a, T: ?Sized + ToOwned> Deref for ValueMutUnsized<'a, T>
// where
//     T::Owned: Borrow<T>,
// {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         self.value()
//     }
// }

// impl<'a, T: ?Sized + ToOwned> AsMut<T> for ValueMutUnsized<'a, T>
// where
//     T::Owned: Borrow<T>,
// {
//     fn as_mut(&mut self) -> &mut T {
//         self.value_mut()
//     }
// }

pub enum Value<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
    Rc(Rc<RefCell<T>>),
}

impl<'a, T> Value<'a, T> {
    pub fn value(&self) -> Readonly<T> {
        match self {
            Value::Owned(v) => Readonly::Borrowed(v),
            Value::Borrowed(v) => Readonly::Borrowed(v),
            Value::Rc(v) => Readonly::Ref(v.borrow()),
        }
    }

    pub fn value_mut(&mut self) -> Writable<T> {
        match self {
            Value::Owned(v) => Writable::Borrowed(v),
            Value::Borrowed(v) => Writable::Borrowed(v),
            Value::Rc(v) => Writable::RefMut(v.borrow_mut()),
        }
    }
}

impl<'a, T: PartialEq> PartialEq for Value<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value().eq(&other.value())
    }
}

impl<'a, T: Eq> Eq for Value<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for Value<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value().partial_cmp(&other.value())
    }
}

impl<'a, T: Ord> Ord for Value<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl<'a, T: Display> Display for Value<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.value(), f)
    }
}

impl<'a, T: Debug> Debug for Value<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value(), f)
    }
}