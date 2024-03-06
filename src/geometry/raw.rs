use std::{any::Any, collections::HashMap};

use crate::{
    bounding::BoundingVolume,
    clock::Tick,
    readonly::{Readonly, ReadonlyUnsized},
    renderer::webgl::{
        attribute::AttributeValue,
        draw::{CullFace, Draw},
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::Geometry;

pub struct RawGeometry {
    draw: Draw,
    cull_face: Option<CullFace>,
    positions: AttributeValue,
    normals: Option<AttributeValue>,
    tangents: Option<AttributeValue>,
    bitangents: Option<AttributeValue>,
    texture_coordinates: Option<AttributeValue>,
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, Box<dyn UniformValue>>,
    uniform_blocks: HashMap<String, UniformBlockValue>,
}

impl RawGeometry {
    pub fn new(
        draw: Draw,
        cull_face: Option<CullFace>,
        positions: AttributeValue,
        normals: Option<AttributeValue>,
        tangents: Option<AttributeValue>,
        bitangents: Option<AttributeValue>,
        texture_coordinates: Option<AttributeValue>,
        attributes: HashMap<String, AttributeValue>,
        uniforms: HashMap<String, Box<dyn UniformValue>>,
        uniform_blocks: HashMap<String, UniformBlockValue>,
    ) -> Self {
        Self {
            draw,
            cull_face,
            positions,
            normals,
            tangents,
            bitangents,
            texture_coordinates,
            attributes,
            uniforms,
            uniform_blocks,
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

    fn bounding_volume(&self) -> Option<Readonly<'_, BoundingVolume>> {
        None
    }

    fn positions(&self) -> Readonly<'_, AttributeValue> {
        Readonly::Borrowed(&self.positions)
    }

    fn normals(&self) -> Option<Readonly<'_, AttributeValue>> {
        self.normals.as_ref().map(|v| Readonly::Borrowed(v))
    }

    fn tangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        self.tangents.as_ref().map(|v| Readonly::Borrowed(v))
    }

    fn bitangents(&self) -> Option<Readonly<'_, AttributeValue>> {
        self.bitangents.as_ref().map(|v| Readonly::Borrowed(v))
    }

    fn texture_coordinates(&self) -> Option<Readonly<'_, AttributeValue>> {
        self.texture_coordinates
            .as_ref()
            .map(|v| Readonly::Borrowed(v))
    }

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>> {
        self.attributes.get(name).map(|v| Readonly::Borrowed(v))
    }

    fn uniform_value(&self, name: &str) -> Option<ReadonlyUnsized<'_, dyn UniformValue>> {
        self.uniforms.get(name).map(|v| ReadonlyUnsized::Borrowed(v.as_ref()))
    }

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        self.uniform_blocks.get(name).map(|v| Readonly::Borrowed(v))
    }

    fn tick(&mut self, _: &Tick) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
