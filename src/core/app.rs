use super::{
    channel::MessageChannel,
    clock::{Clock, Tick},
    engine::RenderEngine,
    scene::Scene,
};

pub struct App {
    channel: MessageChannel,
    scene: Scene,
    clock: Box<dyn Clock>,
    engine: Box<dyn RenderEngine>,
}

impl App {
    pub fn channel(&self) -> &MessageChannel {
        &self.channel
    }

    pub fn render(&mut self) {
        self.engine.render(&mut self.scene);
    }
}
