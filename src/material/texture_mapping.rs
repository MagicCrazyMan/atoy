use std::{any::Any, borrow::Cow};

use crate::{
    event::EventAgency,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ProgramSource,
        state::FrameState,
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
            TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
            TexturePixelStorage, TextureUnit, TextureWrapMethod,
        },
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
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
    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn ready(&self) -> bool {
        self.diffuse_texture.loaded()
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::Transparency,
            UniformBinding::FromMaterial(Cow::Borrowed("u_DiffuseTexture")),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<UniformValue> {
        match name {
            "u_DiffuseTexture" => self.diffuse_texture.texture(),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue> {
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

    fn prepare(&mut self, _: &mut FrameState) {
        self.diffuse_texture.load();
    }

    fn as_standard_program_source(&self) -> &dyn StandardMaterialSource {
        self
    }

    fn as_program_source(&self) -> &dyn ProgramSource {
        self
    }
}

impl StandardMaterialSource for TextureMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("TextureMaterial")
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/texture_process_frag.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Cow<'static, str>> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Cow<'static, str>> {
        vec![]
    }
}
