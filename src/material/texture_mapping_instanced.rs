use gl_matrix4rust::mat4::{AsMat4, Mat4};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::HtmlImageElement;

use crate::{
    document,
    entity::Entity,
    geometry::Geometry,
    render::webgl::{
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferUsage,
        },
        program::{AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue},
        texture::{
            TextureDataType, TextureDescriptor, TextureFormat, TextureMagnificationFilter,
            TextureMinificationFilter, TextureParameter, TexturePixelStorage, TextureWrapMethod, TextureUnit,
        },
    },
    scene::Scene,
};

use super::Material;

const INSTANCE_MODEL_MATRIX_ATTRIBUTE: &'static str = "a_InstanceMatrix";

const SAMPLER_UNIFORM: &'static str = "u_Sampler";

const VERTEX_SHADER_SOURCE: &'static str = "#version 300 es

in vec4 a_Position;
in vec2 a_TexCoord;
in mat4 a_InstanceMatrix;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec2 v_TexCoord;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_InstanceMatrix * a_Position;
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

pub struct TextureInstancedMaterial {
    count: i32,
    instance_matrices: BufferDescriptor,
    url: String,
    texture: Option<TextureDescriptor>,
    image: Option<HtmlImageElement>,
    onload: Option<Closure<dyn FnMut()>>,
}

impl TextureInstancedMaterial {
    pub fn new(url: String, count: i32, grid: i32, width: f32, height: f32) -> Self {
        let cell_width = width / (grid as f32);
        let cell_height = height / (grid as f32);
        let start_x = width / 2.0 - cell_width / 2.0;
        let start_z = height / 2.0 - cell_height / 2.0;

        let matrices_bytes_length = (16 * 4 * count) as usize;
        let mut matrices_data = Vec::with_capacity(matrices_bytes_length);
        for index in 0..count {
            let row = index / grid;
            let col = index % grid;

            let center_x = start_x - col as f32 * cell_width;
            let center_z = start_z - row as f32 * cell_height;
            matrices_data.extend(Mat4::from_translation(&(center_x, 0.0, center_z)).to_gl_binary());
        }

        Self {
            count,
            instance_matrices: BufferDescriptor::from_binary(
                matrices_data,
                0,
                matrices_bytes_length as u32,
                BufferUsage::StaticDraw,
            ),
            url,
            texture: None,
            image: None,
            onload: None,
        }
    }
}

impl Material for TextureInstancedMaterial {
    fn name(&self) -> &'static str {
        "TextureInstancedMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
            AttributeBinding::FromMaterial(INSTANCE_MODEL_MATRIX_ATTRIBUTE),
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
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
        Some(self.count)
    }

    fn attribute_value(&self, name: &str) -> Option<AttributeValue> {
        match name {
            INSTANCE_MODEL_MATRIX_ATTRIBUTE => Some(AttributeValue::InstancedBuffer {
                descriptor: self.instance_matrices.clone(),
                target: BufferTarget::Buffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 4,
                divisor: 1,
            }),
            _ => None,
        }
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
