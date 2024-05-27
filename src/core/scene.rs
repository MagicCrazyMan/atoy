use super::{app::AppConfig, ecs::manager::EntityManager};

pub struct Scene {
    entity_manager: EntityManager,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
        }
    }

    pub fn entity_manager(&self) -> &EntityManager {
        &self.entity_manager
    }
    
    pub fn entity_manager_mut(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }
}
