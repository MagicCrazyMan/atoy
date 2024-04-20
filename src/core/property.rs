use std::any::Any;

use uuid::Uuid;

pub struct Property {
    id: Uuid,
    version: usize,
    value: Box<dyn Any>,
}

impl Property {
    pub fn new<V>(value: V) -> Self
    where
        V: 'static,
    {
        Self {
            id: Uuid::new_v4(),
            version: usize::MIN,
            value: Box::new(value),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn version(&self) -> usize {
        self.version
    }

    pub fn next_version(&mut self) {
        self.version = self.version.saturating_add(1);
    }

    pub fn value(&self) -> &dyn Any {
        &*self.value
    }

    pub fn value_downcast<V>(&self) -> Option<&V>
    where
        V: 'static,
    {
        let value = self.value.as_ref();
        value.downcast_ref()
    }

    pub fn set_value<V>(&mut self, value: V)
    where
        V: 'static,
    {
        self.value = Box::new(value);
    }
}
