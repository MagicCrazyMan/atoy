pub mod webgl;

use crate::scene::Scene;

pub trait Render {
    fn render(scene: &Scene);
}
