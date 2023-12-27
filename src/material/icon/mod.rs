use std::any::Any;

use crate::{
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::{ProgramSource, ShaderSource},
            uniform::{
                UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
                UniformValue,
            },
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

impl ProgramSource for IconMaterial {
    fn name(&self) -> &'static str {
        "IconMaterial"
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(include_str!("./icon.vert")),
            ShaderSource::FragmentRaw(include_str!("./icon.frag")),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Sampler"),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

impl Material for IconMaterial {
    fn transparency(&self) -> Transparency {
        self.transparency
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

    fn uniform_block_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformBlockValue> {
        None
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
