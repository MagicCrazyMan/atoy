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
}
