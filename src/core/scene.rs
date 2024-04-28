use super::entity::Entity;

pub struct Scene<Component> {
    entities: Vec<Box<dyn Entity<Component = Component>>>
}