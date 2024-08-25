use crate::anewthing::{app::App, renderer::Renderer};

pub struct WebGlRenderer {}

impl WebGlRenderer {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Renderer for WebGlRenderer {
    fn render(&mut self, app: &App, timestamp: f64) {
        todo!()
    }
}
