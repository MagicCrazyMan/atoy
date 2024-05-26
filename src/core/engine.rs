use super::{app::AppConfig, scene::Scene};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreRender;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PostRender;

pub struct RenderContext<'a, CLK> {
    pub scene: &'a Scene,
    pub clock: &'a CLK,
}

pub trait RenderEngine<CLK> {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized;

    fn render(&mut self, context: &RenderContext<CLK>)
    where
        Self: Sized;
}
