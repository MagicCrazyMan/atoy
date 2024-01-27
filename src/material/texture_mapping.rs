use std::{any::Any, borrow::Cow};

use crate::{
    loader::texture::TextureLoader,
    notify::Notifier,
    readonly::Readonly,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ProgramSource,
        shader::Define,
        state::FrameState,
        texture::{
            MemoryPolicy, Texture2D, TextureDataType, TextureDescriptor, TextureFormat,
            TextureInternalFormat, TextureMagnificationFilter, TextureMinificationFilter,
            TextureParameter, TexturePixelStorage, TextureSource, TextureUnit, TextureWrapMethod,
        },
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::{StandardMaterial, StandardMaterialSource, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,
    diffuse: TextureLoader,
    notifier: Notifier<()>,
}

impl TextureMaterial {
    pub fn new<S: Into<String>>(url: S, transparency: Transparency) -> Self {
        let notifier = Notifier::new();
        let mut notifier_cloned = notifier.clone();
        Self {
            transparency,
            diffuse: TextureLoader::from_url(url, move |image| {
                notifier_cloned.notify(&mut ());
                UniformValue::Texture2D {
                    descriptor: TextureDescriptor::<Texture2D>::with_source(
                        TextureSource::HtmlImageElement {
                            image,
                            format: TextureFormat::RGBA,
                            data_type: TextureDataType::UNSIGNED_BYTE,
                            pixel_storages: vec![TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
                            custom_size: None,
                        },
                        TextureInternalFormat::SRGB8_ALPHA8,
                        true,
                        MemoryPolicy::default(),
                    ),
                    params: vec![
                        TextureParameter::MIN_FILTER(
                            TextureMinificationFilter::LINEAR_MIPMAP_LINEAR,
                        ),
                        TextureParameter::MAG_FILTER(TextureMagnificationFilter::LINEAR),
                        TextureParameter::WRAP_S(TextureWrapMethod::MIRRORED_REPEAT),
                        TextureParameter::WRAP_T(TextureWrapMethod::MIRRORED_REPEAT),
                    ],
                    unit: TextureUnit::TEXTURE0,
                }
            }),
            notifier,
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn ready(&self) -> bool {
        self.diffuse.loaded()
    }

    fn prepare(&mut self, _: &mut FrameState) {
        self.diffuse.load();
    }

    fn transparency(&self) -> Transparency {
        self.transparency
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
            UniformBinding::FromMaterial(Cow::Borrowed("u_DiffuseTexture")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>> {
        match name {
            "u_DiffuseTexture" => self
                .diffuse
                .texture()
                .map(|texture| Readonly::Borrowed(texture)),
            "u_Transparency" => Some(Readonly::Owned(UniformValue::Float1(self.transparency.alpha()))),
            "u_SpecularShininess" => Some(Readonly::Owned(UniformValue::Float1(128.0))),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue> {
        None
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

    fn as_standard_material_source(&self) -> &dyn StandardMaterialSource {
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
        Cow::Borrowed(include_str!("./shaders/texture_build_material.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Define> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Define> {
        vec![]
    }
}
