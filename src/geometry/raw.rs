use std::{any::Any, collections::HashMap};

use crate::{
    bounding::BoundingVolume,
    event::EventAgency,
    render::webgl::{
        attribute::AttributeValue,
        draw::Draw,
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::Geometry;

pub struct RawGeometry {
    draw: Draw,
    vertices: Option<AttributeValue>,
    normals: Option<AttributeValue>,
    texture_coordinates: Option<AttributeValue>,
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, UniformValue>,
    uniform_blocks: HashMap<String, UniformBlockValue>,
    changed_event: EventAgency<()>,
}

impl RawGeometry {
    pub fn new(
        draw: Draw,
        vertices: Option<AttributeValue>,
        normals: Option<AttributeValue>,
        texture_coordinates: Option<AttributeValue>,
        attributes: HashMap<String, AttributeValue>,
        uniforms: HashMap<String, UniformValue>,
        uniform_blocks: HashMap<String, UniformBlockValue>,
    ) -> Self {
        Self {
            draw,
            vertices,
            normals,
            texture_coordinates,
            attributes,
            uniforms,
            uniform_blocks,
            changed_event: EventAgency::new(),
        }
    }
}

impl Geometry for RawGeometry {
    fn draw(&self) -> Draw {
        self.draw.clone()
    }

    fn bounding_volume(&self) -> Option<BoundingVolume> {
        None
    }

    fn vertices(&self) -> Option<AttributeValue> {
        self.vertices.clone()
    }

    fn normals(&self) -> Option<AttributeValue> {
        self.normals.clone()
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        self.texture_coordinates.clone()
    }

    fn attribute_value(&self, name: &str) -> Option<AttributeValue> {
        self.attributes.get(name).map(|v| v.clone())
    }

    fn uniform_value(&self, name: &str) -> Option<UniformValue> {
        self.uniforms.get(name).map(|v| v.clone())
    }

    fn uniform_block_value(&self, name: &str) -> Option<UniformBlockValue> {
        self.uniform_blocks.get(name).map(|v| v.clone())
    }

    fn changed_event(&self) -> &EventAgency<()> {
        &self.changed_event
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
