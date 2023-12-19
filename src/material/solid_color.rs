use std::any::Any;

use palette::rgb::Rgba;

use crate::{
    entity::BorrowedMut,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::{ProgramSource, ShaderSource},
        shader::{ShaderBuilder, ShaderType, Variable, VariableDataType},
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::{Material, Transparency};

#[derive(Debug, Clone, Copy)]
pub struct SolidColorMaterial {
    color: Rgba,
}

impl SolidColorMaterial {
    pub fn new() -> Self {
        Self {
            color: Rgba::default(),
        }
    }

    pub fn with_color(color: Rgba) -> Self {
        Self { color }
    }

    pub fn color(&self) -> Rgba {
        self.color
    }

    pub fn set_color(&mut self, color: Rgba) {
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
                    Variable::new_uniform("u_Color", VariableDataType::FloatVec4),
                    Variable::new_out("o_FragColor", VariableDataType::FloatVec4),
                ],
                [],
                "void main() {
                    o_FragColor = u_Color;
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
            UniformBinding::FromMaterial("u_Color"),
        ]
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

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_Color" => Some(UniformValue::Float4(
                self.color.red,
                self.color.green,
                self.color.blue,
                self.color.alpha,
            )),
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
}
