pub mod webgl;
pub mod pp;

use crate::scene::Scene;

pub trait Render {
    fn render(scene: &Scene);
}
