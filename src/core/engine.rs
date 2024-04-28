use super::scene::Scene;

pub trait RenderEngine {
    type Component;

    fn render(&self, scene: &Scene<Self::Component>);
}
