use std::{any::Any, collections::HashMap};

use crate::render::webgl::{attribute::AttributeValue, draw::Draw, uniform::UniformValue, EntityRenderState};

use super::Geometry;

pub struct RawGeometry {
    draw: Draw,
    vertices: Option<AttributeValue>,
    normals: Option<AttributeValue>,
    texture_coordinates: Option<AttributeValue>,
    attributes: HashMap<String, AttributeValue>,
    uniforms: HashMap<String, UniformValue>,
}

impl RawGeometry {
    pub fn new(
        draw: Draw,
        vertices: Option<AttributeValue>,
        normals: Option<AttributeValue>,
        texture_coordinates: Option<AttributeValue>,
        attributes: HashMap<String, AttributeValue>,
        uniforms: HashMap<String, UniformValue>,
    ) -> Self {
        Self {
            draw,
            vertices,
            normals,
            texture_coordinates,
            attributes,
            uniforms,
        }
    }
}

impl Geometry for RawGeometry {
    fn draw(&self) -> Draw {
        self.draw.clone()
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

    fn attribute_value(&self, name: &str, _: &EntityRenderState) -> Option<AttributeValue> {
        self.attributes.get(name).map(|v| v.clone())
    }

    fn uniform_value(&self, name: &str, _: &EntityRenderState) -> Option<UniformValue> {
        self.uniforms.get(name).map(|v| v.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
