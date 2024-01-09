use std::any::Any;

use gl_matrix4rust::mat4::Mat4;
use log::info;
use web_sys::js_sys::Float32Array;

use crate::{
    entity::Entity,
    event::EventAgency,
    render::{
        pp::State,
        webgl::{
            attribute::{AttributeBinding, AttributeValue},
            buffer::{
                BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
                BufferUsage,
            },
            program::{ProgramSource, ShaderSource},
            texture::{
                TextureDataType, TextureDescriptor, TextureFormat, TextureInternalFormat,
                TextureMagnificationFilter, TextureMinificationFilter, TextureParameter,
                TexturePixelStorage, TextureUnit, TextureWrapMethod,
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
in mat4 a_InstanceMatrix;
in float a_InstanceIndex;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec2 v_TexCoord;
out float v_InstanceIndex;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_InstanceMatrix * a_Position;
    v_TexCoord = a_TexCoord;
    v_InstanceIndex = a_InstanceIndex;
}
";
const FRAGMENT_SHADER_SOURCE: &'static str = "#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform sampler2D u_Sampler0;
uniform sampler2D u_Sampler1;
uniform sampler2D u_Sampler2;
uniform sampler2D u_Sampler3;
uniform sampler2D u_Sampler4;
uniform sampler2D u_Sampler5;
uniform sampler2D u_Sampler6;
uniform sampler2D u_Sampler7;

in vec2 v_TexCoord;
in float v_InstanceIndex;

out vec4 out_Color;

void main() {
    switch(int(v_InstanceIndex)) {
        case 0:
            out_Color = texture(u_Sampler0, v_TexCoord);
            break;
        case 1:
            out_Color = texture(u_Sampler1, v_TexCoord);
            break;
        case 2:
            out_Color = texture(u_Sampler2, v_TexCoord);
            break;
        case 3:
            out_Color = texture(u_Sampler3, v_TexCoord);
            break;
        case 4:
            out_Color = texture(u_Sampler4, v_TexCoord);
            break;
        case 5:
            out_Color = texture(u_Sampler5, v_TexCoord);
            break;
        case 6:
            out_Color = texture(u_Sampler6, v_TexCoord);
            break;
        case 7:
            out_Color = texture(u_Sampler7, v_TexCoord);
            break;
        default:
            return;
    }
}
";

pub struct MultipleTexturesInstanced {
    count: usize,
    instance_matrices: BufferDescriptor,
    instance_indices: BufferDescriptor,
    textures: [TextureLoader; 8],
    changed_event: EventAgency<()>,
}

impl MultipleTexturesInstanced {
    pub fn new() -> Self {
        let width = 0.25;
        let height = 1.0;

        let count = 8;
        let matrices_bytes_length = (16 * count) as u32;
        let matrices = Float32Array::new_with_length(matrices_bytes_length);
        let indices = Float32Array::new_with_length(8);
        for row in 0..2 {
            for col in 0..4 {
                let index = row * 4 + col;
                let matrix = Mat4::from_translation(&(
                    -1.0 + col as f32 * width,
                    1.0 - row as f32 * height,
                    0.0,
                ));
                info!(
                    "{index} {} {}",
                    -1.0 + col as f32 * width,
                    1.0 - row as f32 * height
                );
                matrices.set_index((index * 16) as u32 + 0, matrix.raw()[0]);
                matrices.set_index((index * 16) as u32 + 1, matrix.raw()[1]);
                matrices.set_index((index * 16) as u32 + 2, matrix.raw()[2]);
                matrices.set_index((index * 16) as u32 + 3, matrix.raw()[3]);
                matrices.set_index((index * 16) as u32 + 4, matrix.raw()[4]);
                matrices.set_index((index * 16) as u32 + 5, matrix.raw()[5]);
                matrices.set_index((index * 16) as u32 + 6, matrix.raw()[6]);
                matrices.set_index((index * 16) as u32 + 7, matrix.raw()[7]);
                matrices.set_index((index * 16) as u32 + 8, matrix.raw()[8]);
                matrices.set_index((index * 16) as u32 + 9, matrix.raw()[9]);
                matrices.set_index((index * 16) as u32 + 10, matrix.raw()[10]);
                matrices.set_index((index * 16) as u32 + 11, matrix.raw()[11]);
                matrices.set_index((index * 16) as u32 + 12, matrix.raw()[12]);
                matrices.set_index((index * 16) as u32 + 13, matrix.raw()[13]);
                matrices.set_index((index * 16) as u32 + 14, matrix.raw()[14]);
                matrices.set_index((index * 16) as u32 + 15, matrix.raw()[15]);
                indices.set_index(index, index as f32);
            }
        }

        Self {
            count,
            instance_matrices: BufferDescriptor::new(
                BufferSource::from_float32_array(matrices, 0, 0),
                BufferUsage::StaticDraw,
            ),
            instance_indices: BufferDescriptor::new(
                BufferSource::from_float32_array(indices, 0, 0),
                BufferUsage::StaticDraw,
            ),
            textures: [
                TextureLoader::from_url("./skybox/skybox_px.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE0,
                }),
                TextureLoader::from_url("./skybox/skybox_py.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE1,
                }),
                TextureLoader::from_url("./skybox/skybox_pz.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE2,
                }),
                TextureLoader::from_url("./skybox/skybox_nx.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE3,
                }),
                TextureLoader::from_url("./skybox/skybox_ny.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE4,
                }),
                TextureLoader::from_url("./skybox/skybox_nz.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE5,
                }),
                TextureLoader::from_url("./skybox/skybox_py.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE6,
                }),
                TextureLoader::from_url("./skybox/skybox_py.jpg", |image| UniformValue::Texture {
                    descriptor: TextureDescriptor::texture_2d_with_html_image_element(
                        image,
                        TextureDataType::UNSIGNED_BYTE,
                        TextureInternalFormat::RGB,
                        TextureFormat::RGB,
                        0,
                        vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
                        true,
                    ),
                    params: vec![
                        TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
                        TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
                        TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
                        TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
                    ],
                    unit: TextureUnit::TEXTURE7,
                }),
            ],
            changed_event: EventAgency::new(),
        }
    }
}

impl ProgramSource for MultipleTexturesInstanced {
    fn name(&self) -> &'static str {
        "TextureInstancedMaterialA"
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
            AttributeBinding::GeometryTextureCoordinate,
            AttributeBinding::FromMaterial("a_InstanceMatrix"),
            AttributeBinding::FromMaterial("a_InstanceIndex"),
        ]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Sampler0"),
            UniformBinding::FromMaterial("u_Sampler1"),
            UniformBinding::FromMaterial("u_Sampler2"),
            UniformBinding::FromMaterial("u_Sampler3"),
            UniformBinding::FromMaterial("u_Sampler4"),
            UniformBinding::FromMaterial("u_Sampler5"),
            UniformBinding::FromMaterial("u_Sampler6"),
            UniformBinding::FromMaterial("u_Sampler7"),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

impl Material for MultipleTexturesInstanced {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn ready(&self) -> bool {
        self.textures.iter().all(|t| t.loaded())
    }

    fn instanced(&self) -> Option<i32> {
        Some(self.count as i32)
    }

    fn attribute_value(&self, name: &str, _: &Entity) -> Option<AttributeValue> {
        match name {
            "a_InstanceMatrix" => Some(AttributeValue::InstancedBuffer {
                descriptor: self.instance_matrices.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Four,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 4,
                divisor: 1,
            }),
            "a_InstanceIndex" => Some(AttributeValue::InstancedBuffer {
                descriptor: self.instance_indices.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::One,
                data_type: BufferDataType::Float,
                normalized: false,
                component_count_per_instance: 1,
                divisor: 1,
            }),
            _ => None,
        }
    }

    fn uniform_value(&self, name: &str, _: &Entity) -> Option<UniformValue> {
        match name {
            "u_Sampler0" => self.textures[0].texture(),
            "u_Sampler1" => self.textures[1].texture(),
            "u_Sampler2" => self.textures[2].texture(),
            "u_Sampler3" => self.textures[3].texture(),
            "u_Sampler4" => self.textures[4].texture(),
            "u_Sampler5" => self.textures[5].texture(),
            "u_Sampler6" => self.textures[6].texture(),
            "u_Sampler7" => self.textures[7].texture(),
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
        self.textures.iter_mut().for_each(|t| t.load());
    }
}
