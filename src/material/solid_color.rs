use std::{any::Any, borrow::Cow};

use gl_matrix4rust::vec3::{AsVec3, Vec3};

use crate::{
    entity::Entity,
    event::EventAgency,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        state::FrameState,
        uniform::{
            UniformBinding, UniformBlockBinding, UniformBlockValue, UniformStructuralBinding,
            UniformValue,
        }, program::ProgramSource,
    },
};

use super::{StandardMaterial, StandardMaterialSource, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
#[derive(Clone)]
pub struct SolidColorMaterial {
    color: Vec3,
    transparency: Transparency,
    changed_event: EventAgency<()>,
}

impl SolidColorMaterial {
    /// Constructs a solid color material with `(1.0, 0.0, 0.0, 1.0)``.
    pub fn new() -> Self {
        Self::with_color(Vec3::from_values(1.0, 0.0, 0.0), Transparency::Opaque)
    }

    /// Constructs a solid color material with specified color and transparency.
    pub fn with_color(color: Vec3, transparency: Transparency) -> Self {
        Self {
            color,
            transparency,
            changed_event: EventAgency::new(),
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
        self.changed_event.raise(());
    }
}

impl StandardMaterial for SolidColorMaterial {
    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn ready(&self) -> bool {
        true
    }

    fn attribute_value(&self, _: &str, _: &Entity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &Entity) -> Option<UniformValue> {
        match name {
            "u_Color" => Some(UniformValue::FloatVector3(self.color.to_gl())),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &Entity) -> Option<UniformBlockValue> {
        None
    }

    fn prepare(&mut self, _: &mut FrameState, _: &Entity) {}

    fn changed_event(&self) -> &EventAgency<()> {
        &self.changed_event
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_standard_program_source(&self) -> &dyn StandardMaterialSource {
        self
    }

    fn as_program_source(&self) -> &dyn ProgramSource {
        self
    }
}

impl StandardMaterialSource for SolidColorMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("SolidColorMaterial")
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/solid_color_process_frag.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Cow<'static, str>> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Cow<'static, str>> {
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
            UniformBinding::FromMaterial(Cow::Borrowed("u_Color")),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}
