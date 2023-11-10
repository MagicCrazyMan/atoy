use std::{any::Any, collections::HashMap};

use uuid::Uuid;

use crate::material::Material;

use super::Geometry;

pub struct Cube {
    id: Uuid,
    size: f32,
    material: Option<Box<dyn Material>>,
    properties: HashMap<String, Box<dyn Any>>,
}

impl Cube {
    pub fn new() -> Cube {
        Self {
            id: Uuid::new_v4(),
            size: 1.0,
            material: None,
            properties: HashMap::new(),
        }
    }

    pub fn with_size(size: f32) -> Cube {
        Self {
            id: Uuid::new_v4(),
            size,
            material: None,
            properties: HashMap::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
}

#[rustfmt::skip]
impl Geometry for Cube {
    fn vertices(&self) -> Option<Vec<f32>> {
        let s = self.size / 2.0;

        Some(vec![
            -s, s, s,  -s,-s, s,   s, s, s,  s, s, s,  -s,-s, s,  s,-s, s, // front
            -s, s,-s,  -s, s, s,   s, s,-s,  s, s,-s,  -s, s, s,  s, s, s, // up
            -s, s,-s,   s, s,-s,  -s,-s,-s,  s, s,-s,   s,-s,-s, -s,-s,-s, // back
            -s,-s,-s,   s,-s,-s,  -s,-s, s,  s,-s,-s,   s,-s, s, -s,-s, s, // bottom
            -s, s,-s,  -s,-s,-s,  -s, s, s, -s, s, s,  -s,-s,-s, -s,-s, s, // left
             s, s, s,   s,-s, s,   s, s,-s,  s, s,-s,   s,-s, s,  s,-s,-s, // right
        ])
    }

    fn normals(&self) -> Option<Vec<f32>> {
        Some(vec![
            0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 1.0, 0.0,  // front
            0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  // up
            0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  0.0, 0.0,-1.0, 0.0,  // back
            0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  0.0,-1.0, 0.0, 0.0,  // bottom
           -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0,  // left
            1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  1.0, 0.0, 0.0, 0.0,  // right
        ])
    }

    fn textures(&self) -> Option<Vec<f32>> {
        todo!()
    }

    fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut HashMap<String, Box<dyn Any>> {
        &mut self.properties
    }

    fn material(&self) -> Option<&dyn Material> {
        match &self.material {
            Some(material) => Some(&**material),
            None => None,
        }
    }

    // fn set_material<M: Material>(&mut self, material: Option<M>) {
    //     self.material = material
    // }
}
