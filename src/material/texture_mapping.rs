use std::{any::Any, ptr::NonNull};

use crate::{
    entity::Entity,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            shader::Variable,
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
                TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
                TexturePixelStorage, TextureUnit, TextureWrapMethod,
            },
            uniform::{
                UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
                UniformValue,
            },
        },
    },
};

use super::{loader::TextureLoader, Material, StandardMaterialSource, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,
    diffuse_texture: TextureLoader,
}

impl TextureMaterial {
    pub fn new<S: Into<String>>(url: S, transparency: Transparency) -> Self {
        Self {
            transparency,
            diffuse_texture: TextureLoader::from_url(url, |image| UniformValue::Texture {
                descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                    image,
                    TextureDataType::UNSIGNED_BYTE,
                    TextureInternalFormat::SRGB8,
                    TextureFormat::RGB,
                    0,
                    vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                    false,
                ),
                params: vec![
                    TextureParameter::MinFilter(TextureMinificationFilter::Linear),
                    TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                    TextureParameter::WrapS(TextureWrapMethod::MirroredRepeat),
                    TextureParameter::WrapT(TextureWrapMethod::MirroredRepeat),
                ],
                unit: TextureUnit::TEXTURE0,
            }),
        }
    }
}

impl StandardMaterialSource for TextureMaterial {
    fn name(&self) -> &'static str {
        "TextureMaterial"
    }

    fn vertex_variables(&self) -> Vec<Variable> {
        vec![]
    }

    fn fragment_variables(&self) -> Vec<Variable> {
        vec![]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::Transparency,
            UniformBinding::FromMaterial("u_DiffuseSampler"),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }

    fn fragment_process(&self) -> &'static str {
        include_str!("./standard/texture_process_frag.glsl")
    }
}

impl Material for TextureMaterial {
    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn ready(&self) -> bool {
        self.diffuse_texture.loaded()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &Entity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &Entity) -> Option<UniformValue> {
        match name {
            "u_DiffuseSampler" => self.diffuse_texture.texture(),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &Entity) -> Option<UniformBlockValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(&mut self, _: &mut State, _: &Entity) {
        self.diffuse_texture.load();
    }
}
