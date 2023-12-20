use std::any::Any;

use gl_matrix4rust::vec4::Vec4;
use palette::{encoding::Srgb, rgb::Rgba};

use crate::{
    entity::BorrowedMut,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::{ProgramSource, ShaderSource},
        shader::{ShaderBuilder, ShaderType, Variable, VariableDataType},
        uniform::{
            UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
            UniformValue,
        },
    },
};

use super::{Material, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
#[derive(Debug, Clone, Copy)]
pub struct SolidColorMaterial {
    color: Rgba<Srgb, f64>,
}

impl SolidColorMaterial {
    pub fn new() -> Self {
        Self {
            color: Rgba::default(),
        }
    }

    pub fn with_color(color: Rgba<Srgb, f64>) -> Self {
        Self { color }
    }

    pub fn color(&self) -> Rgba<Srgb, f64> {
        self.color
    }

    pub fn set_color(&mut self, color: Rgba<Srgb, f64>) {
        self.color = color;
    }
}

impl ProgramSource for SolidColorMaterial {
    fn name(&self) -> &'static str {
        "SolidColorMaterial"
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::Builder(ShaderBuilder::new(
                ShaderType::Vertex,
                [
                    Variable::from_attribute_binding(AttributeBinding::GeometryPosition),
                    Variable::from_attribute_binding(AttributeBinding::GeometryNormal),
                    Variable::from_uniform_binding(UniformBinding::ModelMatrix),
                    Variable::from_uniform_binding(UniformBinding::NormalMatrix),
                    Variable::from_uniform_binding(UniformBinding::ViewProjMatrix),
                    Variable::new_out("v_Normal", VariableDataType::FloatVec3),
                    Variable::new_out("v_Position", VariableDataType::FloatVec3),
                ],
                [],
                "void main() {
                    v_Normal = vec3(u_NormalMatrix * a_Normal);

                    vec4 position = u_ModelMatrix * a_Position;
                    v_Position = vec3(position);
                    gl_Position = u_ViewProjMatrix * position;
                }",
            )),
            ShaderSource::Builder(ShaderBuilder::new(
                ShaderType::Fragment,
                [
                    Variable::from_uniform_binding(UniformBinding::ActiveCameraPosition),
                    Variable::from_uniform_binding(UniformBinding::EnableLighting),
                    Variable::from_uniform_structural_binding(
                        UniformStructuralBinding::AmbientLight,
                    ),
                    Variable::from_uniform_block_binding(UniformBlockBinding::DiffuseLights),
                    Variable::from_uniform_binding(UniformBinding::AmbientReflection),
                    Variable::from_uniform_binding(UniformBinding::DiffuseReflection),
                    Variable::new_in("v_Normal", VariableDataType::FloatVec3),
                    Variable::new_in("v_Position", VariableDataType::FloatVec3),
                    Variable::new_out("o_FragColor", VariableDataType::FloatVec4),
                ],
                [
                    include_str!("../light/shaders/attenuation.glsl"),
                    include_str!("../light/shaders/ambient.glsl"),
                    include_str!("../light/shaders/diffuse.glsl"),
                ],
                "void main() {
                    if (u_EnableLighting) {
                        vec3 ambient_reflection = vec3(u_AmbientReflection);
                        vec3 diffuse_reflection = vec3(u_DiffuseReflection);
                        vec3 surface_normal = normalize(v_Normal);
                        vec3 surface_position = v_Position;
                        vec3 receiver_position = u_ActiveCameraPosition;
                        
                        vec3 ambient = ambient_light(u_AmbientLight, ambient_reflection);
                        vec3 diffuse = diffuse_lights(diffuse_reflection, surface_normal, surface_position, receiver_position);
                        o_FragColor = vec4(ambient + diffuse, u_AmbientReflection.a);
                    } else {
                        o_FragColor = u_AmbientReflection;
                    }
                }",
            )),
        ]
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::ActiveCameraPosition,
            UniformBinding::EnableLighting,
            UniformBinding::AmbientReflection,
            UniformBinding::DiffuseReflection,
        ]
    }

    fn uniform_structural_bindings(&self) -> &[UniformStructuralBinding] {
        &[UniformStructuralBinding::AmbientLight]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[UniformBlockBinding::DiffuseLights]
    }
}

impl Material for SolidColorMaterial {
    fn transparency(&self) -> Transparency {
        if self.color.alpha == 0.0 {
            Transparency::Transparent
        } else if self.color.alpha == 1.0 {
            Transparency::Opaque
        } else {
            Transparency::Translucent(self.color.alpha)
        }
    }

    fn ambient(&self) -> Option<Vec4> {
        Some(Vec4::from_values(
            self.color.red,
            self.color.green,
            self.color.blue,
            self.color.alpha,
        ))
    }

    fn diffuse(&self) -> Option<Vec4> {
        Some(Vec4::from_values(
            self.color.red,
            self.color.green,
            self.color.blue,
            self.color.alpha,
        ))
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformValue> {
        None
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
}
