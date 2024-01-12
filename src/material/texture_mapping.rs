use std::{any::Any, borrow::Cow};

use crate::{
    entity::Entity,
    event::EventAgency,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        state::FrameState,
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
            TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
            TexturePixelStorage, TextureUnit, TextureWrapMethod,
        },
        uniform::{UniformBinding, UniformBlockValue, UniformValue},
    },
};

use super::{loader::TextureLoader, StandardMaterial, StandardMaterialSource, Transparency};

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
                        TextureParameter::MIN_FILTER(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MAG_FILTER(TextureMagnificationFilter::Linear),
                        TextureParameter::WRAP_S(TextureWrapMethod::MirroredRepeat),
                        TextureParameter::WRAP_T(TextureWrapMethod::MirroredRepeat),
                    ],
                    unit: TextureUnit::TEXTURE0,
                }
            }),
            changed_event,
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn source(&self) -> StandardMaterialSource {
        StandardMaterialSource::new(
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
                UniformBinding::FromMaterial(Cow::Borrowed("u_DiffuseSampler")),
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

    fn prepare(&mut self, _: &mut FrameState, _: &Entity) {
        self.diffuse_texture.load();
    }
}
