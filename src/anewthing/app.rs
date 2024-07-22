use std::any::{Any, TypeId};

use hashbrown::HashMap;

use super::{channel::Channel, plugin::Plugin};

pub struct App {
    channel: Channel,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl App {
    pub fn new() -> Self {
        let channel = Channel::new();

        Self {
            channel,
            plugins: HashMap::new(),
        }
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn plugin<P>(&self) -> Option<&P>
    where
        P: Plugin + 'static,
    {
        self.plugins
            .get(&TypeId::of::<P>())
            .map(|p| p.downcast_ref::<P>().unwrap())
    }

    pub fn plugin_mut<P>(&mut self) -> Option<&mut P>
    where
        P: Plugin + 'static,
    {
        self.plugins
            .get_mut(&TypeId::of::<P>())
            .map(|p| p.downcast_mut::<P>().unwrap())
    }

    pub fn add_plugin<P>(&mut self, mut plugin: P) -> bool
    where
        P: Plugin + 'static,
    {
        let id = TypeId::of::<P>();
        if self.plugins.contains_key(&id) {
            return false;
        }

        plugin.plugin(self);
        self.plugins.insert_unique_unchecked(id, Box::new(plugin));
        true
    }

    pub fn remove_plugin<P>(&mut self) -> Option<P>
    where
        P: Plugin + 'static,
    {
        let id = TypeId::of::<P>();
        let plugin = self.plugins.remove(&id)?;
        Some(*plugin.downcast::<P>().unwrap())
    }
}
