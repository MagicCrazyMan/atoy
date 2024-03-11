use std::ops::{Deref, DerefMut};

use uuid::Uuid;

pub struct Property<T> {
    id: Uuid,
    dirty_count: usize,
    value: T,
}

impl<T> Property<T> {
    pub fn new(value: T) -> Self {
        Self::with_counter(value, 1)
    }

    pub fn with_counter(value: T, dirty_count: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            dirty_count,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn take(self) -> T {
        self.value
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn set_value(&mut self, value: T) {
        self.mark_dirty();
        self.value = value;
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty_count
    }

    pub fn mark_dirty(&mut self) {
        self.dirty_count = self.dirty_count.saturating_add(1);
    }

    pub fn is_dirty(&self, stamp: &PropertyStamp) -> bool {
        self.dirty_count != stamp.dirty_count || self.id != stamp.id
    }

    pub fn stamp(&self) -> PropertyStamp {
        PropertyStamp {
            id: self.id,
            dirty_count: self.dirty_count,
        }
    }

    pub fn stamp_to(&self, stamp: &mut PropertyStamp) {
        stamp.id = self.id;
        stamp.dirty_count = self.dirty_count;
    }
}

impl<T> Deref for Property<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Property<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> AsRef<T> for Property<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> AsMut<T> for Property<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyStamp {
    id: Uuid,
    dirty_count: usize,
}

impl PropertyStamp {
    pub fn new(id: Uuid, dirty_count: usize) -> Self {
        Self { id, dirty_count }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty_count
    }
}
