pub mod entity_collector;

use super::{engine::RenderEngine, scene::Scene};

pub trait Operator<RE>
where
    RE: RenderEngine,
{
    type Output;

    fn execute(&mut self, scene: &mut Scene, engine: &mut RE) -> Self::Output;
}
