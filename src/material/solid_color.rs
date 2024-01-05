use std::{any::Any, ptr::NonNull};

use gl_matrix4rust::vec3::{AsVec3, Vec3};

use crate::{
    entity::Entity,
    render::{webgl::{
        attribute::{AttributeBinding, AttributeValue},
        shader::Variable,
        uniform::{
            UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
            UniformValue,
        },
    }, pp::State},
};

use super::{Material, StandardMaterialSource, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
#[derive(Clone, Copy)]
pub struct SolidColorMaterial {
    color: Vec3,
    transparency: Transparency,
}

impl SolidColorMaterial {
    /// Constructs a solid color material with `(1.0, 0.0, 0.0, 1.0)``.
    pub fn new() -> Self {
        Self {
            color: Vec3::from_values(1.0, 0.0, 0.0),
            transparency: Transparency::Opaque,
        }
    }

    /// Constructs a solid color material with specified color and transparency.
    pub fn with_color(color: Vec3, transparency: Transparency) -> Self {
        Self {
            color,
            transparency,
        }
    }

    /// Returns color.
    pub fn color(&self) -> Vec3 {
        self.color
    }

    /// Sets color,
    pub fn set_color(&mut self, color: Vec3, transparency: Transparency) {
        self.color = color;
        self.transparency = transparency;
    }
}

impl StandardMaterialSource for SolidColorMaterial {
    fn name(&self) -> &'static str {
        "SolidColorMaterial"
    }

    fn vertex_variables(&self) -> Vec<Variable> {
        vec![]
    }

    fn fragment_variables(&self) -> Vec<Variable> {
        vec![]
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
            UniformBinding::Transparency,
            UniformBinding::FromMaterial("u_Color"),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }

    fn fragment_process(&self) -> &'static str {
        include_str!("./standard/solid_color_process_frag.glsl")
    }
}

impl Material for SolidColorMaterial {
    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn attribute_value(&self, _: &str, _: NonNull<Entity>) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: NonNull<Entity>) -> Option<UniformValue> {
        match name {
            "u_Color" => Some(UniformValue::FloatVector3(self.color.to_gl())),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: NonNull<Entity>) -> Option<UniformBlockValue> {
        None
    }

    fn prepare(&mut self, state: &mut State, entity: NonNull<Entity>) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
