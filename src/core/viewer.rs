use super::{
    channel::MessageChannel, clock::Clock, engine::RenderEngine, scene::Scene, webgl::WebGl,
};

pub struct Viewer<RenderType> {
    channel: MessageChannel,
    clock: Box<dyn Clock>,
    scene: Scene<RenderType>,
    engine: Box<dyn RenderEngine<RenderType = RenderType>>,
}

impl Viewer<WebGl> {
    pub fn render(&self) {
        self.engine.render(&self.scene);
    }
}
