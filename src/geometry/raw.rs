use std::{any::Any, collections::HashMap};

use crate::{
    bounding::BoundingVolume, notify::Notifier, render::webgl::{
        attribute::AttributeValue,
        draw::{CullFace, Draw},
        uniform::{UniformBlockValue, UniformValue},
    }
};

use super::Geometry;

pub struct RawGeometry {
    draw: Draw,
    cull_face: Option<CullFace>,
    positions: Option<AttributeValue>,
    normals: Option<AttributeValue>,
    texture_coordinates: Option<AttributeValue>,
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, UniformValue>,
    uniform_blocks: HashMap<String, UniformBlockValue>,
    notifier: Notifier<()>,
}

impl RawGeometry {
    pub fn new(
        draw: Draw,
        cull_face: Option<CullFace>,
        positions: Option<AttributeValue>,
        normals: Option<AttributeValue>,
        texture_coordinates: Option<AttributeValue>,
        attributes: HashMap<String, AttributeValue>,
        uniforms: HashMap<String, UniformValue>,
        uniform_blocks: HashMap<String, UniformBlockValue>,
    ) -> Self {
        Self {
            draw,
            cull_face,
            positions,
            normals,
            texture_coordinates,
            attributes,
            uniforms,
            uniform_blocks,
            notifier: Notifier::new()
        }
    }
}

impl Geometry for RawGeometry {
    fn draw(&self) -> Draw {
        self.draw.clone()
    }

    fn cull_face(&self) -> Option<CullFace> {
        self.cull_face
    }

    fn bounding_volume(&self) -> Option<BoundingVolume> {
        None
    }

    fn positions(&self) -> Option<AttributeValue> {
        self.positions.clone()
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

    fn notifier(&mut self) -> &mut Notifier<()> {
        &mut self.notifier
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
