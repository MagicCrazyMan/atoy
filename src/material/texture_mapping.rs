use std::{any::Any, borrow::Cow};

use crate::{
    entity::Entity,
    event::EventAgency,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
                TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
                TexturePixelStorage, TextureUnit, TextureWrapMethod,
            },
            uniform::{UniformBinding, UniformBlockValue, UniformValue},
        },
    },
};

use super::{loader::TextureLoader, Material, MaterialSource, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,
    diffuse_texture: TextureLoader,
    changed_event: EventAgency<()>,
}

impl TextureMaterial {
    pub fn new<S: Into<String>>(url: S, transparency: Transparency) -> Self {
        let changed_event = EventAgency::new();
        let changed_event_cloned = changed_event.clone();
        Self {
            transparency,
            diffuse_texture: TextureLoader::from_url(url, move |image| {
                changed_event_cloned.raise(());
                UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::SRGB8_ALPHA8,
                        TextureFormat::RGBA,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::MirroredRepeat),
                        TextureParameter::WrapT(TextureWrapMethod::MirroredRepeat),
                    ],
                    unit: TextureUnit::TEXTURE0,
                }
            }),
            changed_event,
        }
    }
}

impl Material for TextureMaterial {
    fn source(&self) -> MaterialSource {
        MaterialSource::new(
            Cow::Borrowed("TextureMaterial"),
            None,
            Cow::Borrowed(include_str!("./shaders/texture_process_frag.glsl")),
            vec![],
            vec![],
            vec![
                AttributeBinding::GeometryPosition,
                AttributeBinding::GeometryNormal,
                AttributeBinding::GeometryTextureCoordinate,
            ],
            vec![
                UniformBinding::ModelMatrix,
                UniformBinding::NormalMatrix,
                UniformBinding::Transparency,
                UniformBinding::FromMaterial("u_DiffuseSampler"),
            ],
            vec![],
            vec![],
        )
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn ready(&self) -> bool {
        self.diffuse_texture.loaded()
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

    fn changed_event(&self) -> &EventAgency<()> {
        &self.changed_event
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
