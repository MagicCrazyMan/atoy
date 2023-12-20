use std::any::Any;

use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};
use web_sys::HtmlImageElement;

use crate::{
    document,
    entity::BorrowedMut,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::{ShaderSource, ProgramSource},
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureMagnificationFilter,
                TextureMinificationFilter, TextureParameter, TexturePixelStorage, TextureUnit,
                TextureWrapMethod, TextureInternalFormat,
            },
            uniform::{UniformBinding, UniformValue, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding},
        },
    },
};

use super::{Material, Transparency};

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

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

#[wasm_bindgen]
pub struct TextureMaterial {
    url: String,
    texture: Option<TextureDescriptor>,
    image: Option<HtmlImageElement>,
    onload: Option<Closure<dyn FnMut()>>,
}

#[wasm_bindgen]
impl TextureMaterial {
    #[wasm_bindgen]
    pub fn new_constructor(url: String) -> Self {
        Self::new(url)
    }
}

impl TextureMaterial {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            texture: None,
            image: None,
            onload: None,
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
            UniformBinding::FromMaterial(SAMPLER_UNIFORM),
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
        self.texture.is_some()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            SAMPLER_UNIFORM => match &self.texture {
                Some(texture) => Some(UniformValue::Texture {
                    descriptor: texture.clone(),
                    params: vec![
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    texture_unit: TextureUnit::TEXTURE0,
                }),
                None => None,
            },
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
        if self.image.is_none() {
            let image = document()
                .create_element("img")
                .ok()
                .unwrap()
                .dyn_into::<HtmlImageElement>()
                .unwrap();

            image.set_src(&self.url);

            let texture_cloned: *mut Option<TextureDescriptor> = &mut self.texture;
            let image_cloned = image.clone();
            self.onload = Some(Closure::new(move || {
                let texture = Some(TextureDescriptor::texture_2d_with_html_image_element(
                    image_cloned.clone(),
                    TextureDataType::UNSIGNED_BYTE,
                    TextureInternalFormat::RGB,
                    TextureFormat::RGB,
                    0,
                    vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                    true,
                ));
                unsafe {
                    *texture_cloned = texture;
                }
            }));
            image.set_onload(Some(self.onload.as_ref().unwrap().as_ref().unchecked_ref()));

            self.image = Some(image);
        }
    }
}
