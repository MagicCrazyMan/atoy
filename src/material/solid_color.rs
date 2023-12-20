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
                    Variable::from_uniform_binding(UniformBinding::ModelMatrix),
                    Variable::from_uniform_binding(UniformBinding::ViewProjMatrix),
                ],
                [],
                "void main() {
                    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
                }",
            )),
            ShaderSource::Builder(ShaderBuilder::new(
                ShaderType::Fragment,
                [
                    Variable::from_uniform_structural_binding(UniformStructuralBinding::AmbientLight),
                    Variable::from_uniform_binding(UniformBinding::AmbientReflection),
                    Variable::new_out("o_FragColor", VariableDataType::FloatVec4),
                ],
                [include_str!("../light/shaders/ambient.glsl")],
                "void main() {
                    vec4 ambient = vec4(ambient_light(u_AmbientLight, vec3(u_AmbientReflection)), u_AmbientReflection.a);

                    o_FragColor = ambient;
                }",
            )),
        ]
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::AmbientReflection,
        ]
    }

    fn uniform_structural_bindings(&self) -> &[UniformStructuralBinding] {
        &[UniformStructuralBinding::AmbientLight]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
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

    fn ambient_reflection(&self) -> Option<Vec4> {
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
