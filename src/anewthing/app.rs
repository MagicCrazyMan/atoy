use std::any::{Any, TypeId};

use hashbrown::HashMap;

use super::{channel::Channel, ecs::manager::EntityManager, plugin::Plugin, renderer::Renderer};

pub struct App {
    channel: Channel,
    entity_manager: EntityManager,
    renderer: Box<dyn Renderer>,
    plugins: HashMap<TypeId, Box<dyn Any>>,
}

impl App {
    pub fn new<S>(renderer: S) -> Self
    where
        S: Renderer + 'static,
    {
        let channel = Channel::new();

        Self {
            entity_manager: EntityManager::new(channel.clone()),
            plugins: HashMap::new(),
            renderer: Box::new(renderer),
            channel,
        }
    }

    pub fn channel(&self) -> &Channel {
        &self.channel
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

    pub fn add_plugin<P>(&mut self, mut plugin: P) -> Result<(), P>
    where
        P: Plugin + 'static,
    {
        let id = TypeId::of::<P>();
        if self.plugins.contains_key(&id) {
            return Err(plugin);
        }

        plugin.plugin(self);
        self.plugins.insert_unique_unchecked(id, Box::new(plugin));
        Ok(())
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
