use super::{channel::MessageChannel, clock::Clock, engine::RenderEngine, scene::Scene};

pub struct App {
    channel: MessageChannel,
    scene: Scene,
    clock: Box<dyn Clock>,
    engine: Box<dyn RenderEngine>,
}

impl App {
    fn init(&mut self) {
        
    }


    pub fn render(&mut self) {
        self.engine.render(&mut self.scene);
    }
}
