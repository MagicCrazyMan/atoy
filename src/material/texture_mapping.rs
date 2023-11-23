use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};
use web_sys::HtmlImageElement;

use crate::{
    document,
    entity::Entity,
    geometry::Geometry,
    render::webgl::{
        program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureMagnificationFilter,
            TextureMinificationFilter, TextureParameter, TexturePixelStorage, TextureWrapMethod, TextureUnit,
        },
    },
    scene::Scene,
};

use super::Material;

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;
in vec2 a_TexCoord;

uniform mat4 u_ModelViewProjMatrix;

out vec2 v_TexCoord;

void main() {
    gl_Position = u_ModelViewProjMatrix * a_Position;
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

impl Material for TextureMaterial {
    fn name(&self) -> &'static str {
        "TextureMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelViewProjMatrix,
            UniformBinding::FromMaterial(SAMPLER_UNIFORM),
        ]
    }

    fn sources(&self) -> &[ShaderSource] {
        &[
            ShaderSource::Vertex(VERTEX_SHADER_SOURCE),
            ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn ready(&self) -> bool {
        self.texture.is_some()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<UniformValue> {
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

    fn prepare(&mut self, _: &mut Scene, _: &mut Entity, _: &mut dyn Geometry) {
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
                    TextureDataType::UnsignedByte,
                    TextureFormat::RGB,
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
