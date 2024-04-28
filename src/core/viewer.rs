use super::{engine::RenderEngine, scene::Scene, webgl::WebGl};

pub struct Viewer<RenderType> {
    scene: Scene<RenderType>,
    engine: Box<dyn RenderEngine<RenderType = RenderType>>
}

impl Viewer<WebGl> {
    pub fn render(&self) {
        self.engine.render(&self.scene);
    }
}