use super::{engine::RenderEngine, scene::Scene};

pub trait Operator {
    type RenderType;

    type RenderEngine: RenderEngine<RenderType = Self::RenderType>;

    fn execute(&mut self, scene: &Scene<Self::RenderType>, engine: &Self::RenderEngine);
}
