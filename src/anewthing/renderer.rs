use super::app::App;

pub trait Renderer {
    fn render(&mut self, app: &App, timestamp: f64);
}
