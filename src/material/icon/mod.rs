use std::any::Any;

use crate::{
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::ShaderSource,
            uniform::{UniformBinding, UniformValue},
        },
    },
};

use super::{loader::TextureLoader, Material, Transparency};

pub struct IconMaterial {
    transparency: Transparency,
    loader: TextureLoader,
}

impl IconMaterial {
    pub fn new(loader: TextureLoader, transparency: Transparency) -> Self {
        Self {
            transparency,
            loader,
        }
    }
}

impl Material for IconMaterial {
    fn name(&self) -> &'static str {
        "IconMaterial"
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Sampler"),
        ]
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(include_str!("./icon.vert")),
            ShaderSource::Fragment(include_str!("./icon.frag")),
        ]
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_Sampler" => self.loader.texture(),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        self.loader.loaded()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn prepare(&mut self, _: &State, _: &BorrowedMut) {
        self.loader.load();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
