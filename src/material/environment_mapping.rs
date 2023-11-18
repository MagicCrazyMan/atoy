use std::sync::OnceLock;

use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};
use web_sys::HtmlImageElement;

use crate::{
    document,
    entity::Entity,
    geometry::Geometry,
    ncor::Ncor,
    render::webgl::{
        program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureMagnificationFilter,
            TextureMinificationFilter, TextureParameter, TextureSource,
        },
    },
    scene::Scene,
};

use super::WebGLMaterial;

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

static ATTRIBUTE_BINDINGS: OnceLock<[AttributeBinding; 2]> = OnceLock::new();
static UNIFORM_BINDINGS: OnceLock<[UniformBinding; 5]> = OnceLock::new();

static SHADER_SOURCES: OnceLock<[ShaderSource; 2]> = OnceLock::new();
const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;
in vec4 a_Normal;

uniform mat4 u_ModelViewProjMatrix;
uniform mat4 u_ModelMatrix;
uniform mat4 u_NormalMatrix;

out vec3 v_Position;
out vec3 v_Normal;

void main() {
    gl_Position = u_ModelViewProjMatrix * a_Position;
    v_Normal = vec3(u_NormalMatrix * a_Normal);
    v_Position = vec3(u_ModelMatrix * a_Position);
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es
#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform vec3 u_ActiveCameraPosition;
uniform samplerCube u_Sampler;

in vec3 v_Position;
in vec3 v_Normal;

out vec4 out_Color;

void main() {
    vec3 normal = normalize(v_Normal);
    vec3 incident = normalize(v_Position - u_ActiveCameraPosition);
    vec3 reflection = reflect(incident, normal);

    out_Color = texture(u_Sampler, reflection);
}
";

#[wasm_bindgen]
pub struct EnvironmentMaterial {
    urls: Vec<String>,
    count: usize,
    images: Option<Vec<HtmlImageElement>>,
    onload: Option<Closure<dyn FnMut()>>,
    texture: Option<TextureDescriptor>,
}

#[wasm_bindgen]
impl EnvironmentMaterial {
    #[wasm_bindgen]
    pub fn new_constructor(
        px: String,
        nx: String,
        py: String,
        ny: String,
        pz: String,
        nz: String,
    ) -> Self {
        Self::new(px, nx, py, ny, pz, nz)
    }
}

impl EnvironmentMaterial {
    pub fn new(px: String, nx: String, py: String, ny: String, pz: String, nz: String) -> Self {
        Self {
            count: 0,
            urls: vec![px, nx, py, ny, pz, nz],
            onload: None,
            images: None,
            texture: None,
        }
    }
}

impl WebGLMaterial for EnvironmentMaterial {
    fn name(&self) -> &str {
        "EnvironmentMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        ATTRIBUTE_BINDINGS.get_or_init(|| {
            [
                AttributeBinding::GeometryPosition,
                AttributeBinding::GeometryNormal,
            ]
        })
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        UNIFORM_BINDINGS.get_or_init(|| {
            [
                UniformBinding::ModelMatrix,
                UniformBinding::NormalMatrix,
                UniformBinding::ModelViewProjMatrix,
                UniformBinding::ActiveCameraPosition,
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
                    params: vec![
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                    ],
                    active_unit: 0,
                })),
                None => None,
            },
            _ => None,
        }
    }

    fn prepare(&mut self, _: &Scene, _: &Entity, _: &dyn Geometry) {
        if self.images.is_none() {
            let count_ptr: *mut usize = &mut self.count;
            let images_ptr: *const Option<Vec<HtmlImageElement>> = &self.images;
            let texture_ptr: *mut Option<TextureDescriptor> = &mut self.texture;

            self.onload = Some(Closure::new(move || unsafe {
                *count_ptr += 1;

                if *count_ptr == 6 {
                    let images = &*images_ptr;
                    let images = images.as_ref().unwrap();
                    *texture_ptr =
                        Some(TextureDescriptor::texture_cube_map_with_html_image_element(
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(0).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(1).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(2).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(3).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(4).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UnsignedByte,
                                image: Box::new(images.get(5).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            true,
                        ));
                }
            }));

            let images = self
                .urls
                .iter()
                .map(|url| {
                    let image = document()
                        .create_element("img")
                        .ok()
                        .unwrap()
                        .dyn_into::<HtmlImageElement>()
                        .unwrap();

                    image.set_src(&url);
                    image.set_onload(Some(self.onload.as_ref().unwrap().as_ref().unchecked_ref()));

                    image
                })
                .collect::<Vec<_>>();

            self.images = Some(images);
        }
    }
}
