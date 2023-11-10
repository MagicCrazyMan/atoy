pub mod webgl2render;

use crate::scene::Scene;

pub trait Render {
    fn render(scene: &Scene);
}
