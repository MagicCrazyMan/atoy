use std::any::Any;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::HtmlImageElement;

use crate::{
    entity::Entity,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            program::{ProgramSource, ShaderSource},
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
                TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
                TextureSource, TextureUnit,
            },
            uniform::{
                UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
                UniformValue,
            },
        },
    }, event::EventAgency,
};

use super::{Material, Transparency};

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es
in vec4 a_Position;
in vec4 a_Normal;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;
uniform mat4 u_NormalMatrix;

out vec3 v_Position;
out vec3 v_Normal;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
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

uniform vec3 u_CameraPosition;
uniform samplerCube u_Sampler;

in vec3 v_Position;
in vec3 v_Normal;

out vec4 out_Color;

void main() {
    vec3 normal = normalize(v_Normal);
    vec3 incident = normalize(v_Position - u_CameraPosition);
    vec3 reflection = reflect(incident, normal);

    out_Color = texture(u_Sampler, reflection);
}
";

pub struct EnvironmentMaterial {
    urls: Vec<String>,
    count: usize,
    images: Option<Vec<HtmlImageElement>>,
    onload: Option<Closure<dyn FnMut()>>,
    texture: Option<TextureDescriptor>,
    changed_event: EventAgency<()>,
}

impl EnvironmentMaterial {
    pub fn new(px: String, nx: String, py: String, ny: String, pz: String, nz: String) -> Self {
        Self {
            count: 0,
            urls: vec![px, nx, py, ny, pz, nz],
            onload: None,
            images: None,
            texture: None,
            changed_event: EventAgency::new()
        }
    }
}

impl ProgramSource for EnvironmentMaterial {
    fn name(&self) -> &'static str {
        "EnvironmentMaterial"
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(VERTEX_SHADER_SOURCE),
            ShaderSource::FragmentRaw(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
        ]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::CameraPosition,
            UniformBinding::FromMaterial(SAMPLER_UNIFORM),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

impl Material for EnvironmentMaterial {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn ready(&self) -> bool {
        self.texture.is_some()
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &Entity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &Entity) -> Option<UniformValue> {
        match name {
            SAMPLER_UNIFORM => match &self.texture {
                Some(texture) => Some(UniformValue::Texture {
                    descriptor: texture.clone(),
                    params: vec![
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                    ],
                    unit: TextureUnit::TEXTURE0,
                }),
                None => None,
            },
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
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
                                image: Box::new(images.get(0).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
                                image: Box::new(images.get(1).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
                                image: Box::new(images.get(2).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
                                image: Box::new(images.get(3).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
                                image: Box::new(images.get(4).unwrap().clone()),
                                pixel_storages: vec![],
                                x_offset: 0,
                                y_offset: 0,
                            },
                            TextureSource::FromHtmlImageElement {
                                internal_format: TextureInternalFormat::RGB,
                                format: TextureFormat::RGB,
                                data_type: TextureDataType::UNSIGNED_BYTE,
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
                    let image = HtmlImageElement::new().unwrap();
                    image.set_src(&url);
                    image.set_onload(Some(self.onload.as_ref().unwrap().as_ref().unchecked_ref()));

                    image
                })
                .collect::<Vec<_>>();

            self.images = Some(images);
        }
    }
}
