use super::{scene::Scene, AsAny};

pub trait RenderEngine: AsAny {
    type RenderType;

    fn render(&self, scene: &Scene<Self::RenderType>);
}
