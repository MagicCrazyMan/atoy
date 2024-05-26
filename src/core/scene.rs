use super::{app::AppConfig, ecs::manager::EntityManager};

pub struct Scene {
    entity_manager: EntityManager,
}

impl Scene {
    pub fn new(app_config: &AppConfig) -> Self {
        Self {
            entity_manager: EntityManager::new(app_config),
        }
    }

    pub fn entity_manager(&self) -> &EntityManager {
        &self.entity_manager
    }
    
    pub fn entity_manager_mut(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }
}
