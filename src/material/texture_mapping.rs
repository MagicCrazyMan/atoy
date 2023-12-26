use std::any::Any;

use crate::{
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::{ProgramSource, ShaderSource},
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
                TextureParameter, TexturePixelStorage, TextureUnit, TextureWrapMethod, TextureMinificationFilter, TextureMagnificationFilter,
            },
            uniform::{
                UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
                UniformValue,
            },
        },
    },
};

use super::{loader::TextureLoader, Material, Transparency};

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;
in vec2 a_TexCoord;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec2 v_TexCoord;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
    v_TexCoord = a_TexCoord;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform sampler2D u_Sampler;

in vec2 v_TexCoord;

out vec4 out_Color;

void main() {
    out_Color = texture(u_Sampler, v_TexCoord);
}
";

pub struct TextureMaterial {
    texture: TextureLoader,
}

impl TextureMaterial {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            texture: TextureLoader::from_url(url, |image| UniformValue::Texture {
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
                texture_unit: TextureUnit::TEXTURE0,
            }),
        }
    }
}

impl ProgramSource for TextureMaterial {
    fn name(&self) -> &'static str {
        "TextureMaterial"
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(VERTEX_SHADER_SOURCE),
            ShaderSource::FragmentRaw(FRAGMENT_SHADER_SOURCE),
        ]
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

    fn uniform_structural_bindings(&self) -> &[UniformStructuralBinding] {
        &[]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }
}

impl Material for TextureMaterial {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn ready(&self) -> bool {
        self.texture.loaded()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_Sampler" => self.texture.texture(),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformBlockValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(&mut self, _: &State, _: &BorrowedMut) {
        self.texture.load()
    }
}
