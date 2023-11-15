use std::sync::OnceLock;

use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};
use wasm_bindgen_test::console_log;
use web_sys::HtmlImageElement;

use crate::{
    document,
    entity::Entity,
    geometry::Geometry,
    ncor::Ncor,
    render::webgl::{
        program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureTarget,
            TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
            TexturePixelStorage, TextureWrapMethod,
        },
    },
    scene::Scene,
};

use super::WebGLMaterial;

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

static ATTRIBUTE_BINDINGS: OnceLock<[AttributeBinding; 2]> = OnceLock::new();
static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 2]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "
attribute vec4 a_Position;
attribute vec2 a_TexCoord;

uniform mat4 u_ModelViewProjMatrix;

varying vec2 v_TexCoord;

void main() {
    gl_Position = u_ModelViewProjMatrix * a_Position;
    v_TexCoord = a_TexCoord;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

varying vec2 v_TexCoord;

uniform sampler2D u_Sampler;

void main() {
    gl_FragColor = texture2D(u_Sampler, v_TexCoord);
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

impl WebGLMaterial for TextureMaterial {
    fn name(&self) -> &str {
        "TextureMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        ATTRIBUTE_BINDINGS.get_or_init(|| {
            [
                AttributeBinding::GeometryPosition,
                AttributeBinding::GeometryTextureCoordinate,
            ]
        })
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        UNIFORM_BINDINGS.get_or_init(|| {
            [
                UniformBinding::ModelViewProjMatrix,
                UniformBinding::FromMaterial(SAMPLER_UNIFORM.to_string()),
            ]
        })
    }

    fn sources(&self) -> &[ShaderSource] {
        SHADER_SOURCES.get_or_init(|| {
            [
                ShaderSource::Vertex(VERTEX_SHADER_SOURCE.to_string()),
                ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE.to_string()),
            ]
        })
    }

    fn ready(&self) -> bool {
        self.texture.is_some()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value<'a>(&'a self, _name: &str) -> Option<Ncor<'a, AttributeValue>> {
        None
    }

    fn uniform_value<'a>(&'a self, name: &str) -> Option<Ncor<'a, UniformValue>> {
        match name {
            SAMPLER_UNIFORM => match &self.texture {
                Some(texture) => Some(Ncor::Owned(UniformValue::Texture {
                    descriptor: Ncor::Borrowed(texture),
                    target: TextureTarget::Texture2D,
                    params: vec![
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    active_unit: 0,
                })),
                None => None,
            },
            _ => None,
        }
    }

    fn pre_render(&mut self, _: &Scene, _: &Entity, _: &dyn Geometry) {
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
                let texture = Some(TextureDescriptor::with_html_image_element(
                    image_cloned.clone(),
                    TextureDataType::UnsignedByte,
                    TextureFormat::RGB,
                    TextureFormat::RGB,
                    0,
                    vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                    false,
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
