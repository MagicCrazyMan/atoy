use super::{app::AppConfig, clock::Clock, resource::Resources, scene::Scene, AsAny, Rrc};

pub struct RenderContext {
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub resources: Rrc<Resources>,
}

pub trait RenderEngine: AsAny {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized;

    fn render(&mut self, context: RenderContext);
}
