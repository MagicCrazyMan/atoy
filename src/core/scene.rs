use std::{cell::RefCell, rc::Rc};

use super::{camera::Camera, channel::Sender};

pub struct Scene {
    sender: Sender<SceneMessage>,

    camera: Box<dyn Camera>,

    // entities: Vec<Rc<RefCell<dyn Entity>>>,
}

impl Scene {
    pub fn camera(&self) -> &Box<dyn Camera> {
        &self.camera
    }

    // pub fn entity_collection(&self) -> &Collection {
    //     &self.entity_collection
    // }
}

#[derive(Debug)]
pub enum SceneMessage {}
