use super::{scene::Scene, AsAny};

pub trait RenderEngine: AsAny {
    fn render(&mut self, scene: &mut Scene);
}
